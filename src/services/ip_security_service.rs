use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use ipnet::IpNet;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::db::Database;
use crate::error::{AppError, Result};
use crate::models::audit::{AuditEventType, AuditMetadata, CreateAuditLogRequest};
use crate::models::security::{
    BatchIpListRequest, BatchIpListResponse, CreateIpListRequest, IpCheckRequest, IpCheckResponse,
    IpCheckResult, IpListEntry, IpListEntryResponse, IpListQuery, IpListType, IpSecurityStats,
    UpdateIpListRequest,
};
use crate::services::audit_service::AuditService;

/// IP 列表缓存项
#[derive(Debug, Clone)]
enum IpCacheEntry {
    /// 单个 IP 地址
    Single(IpAddr),
    /// CIDR 范围
    Cidr(IpNet),
}

impl IpCacheEntry {
    /// 检查 IP 是否匹配此条目
    fn contains(&self, ip: IpAddr) -> bool {
        match self {
            IpCacheEntry::Single(entry_ip) => *entry_ip == ip,
            IpCacheEntry::Cidr(net) => net.contains(&ip),
        }
    }
}

/// IP 安全服务
/// 负责 IP 白名单/黑名单的管理和检查
/// 支持内存缓存提高查询性能
pub struct IpSecurityService {
    db: Database,
    audit_service: Arc<AuditService>,
    /// 白名单缓存（包含单个 IP 和 CIDR 范围）
    whitelist_cache: Arc<RwLock<Vec<IpCacheEntry>>>,
    /// 黑名单缓存（包含单个 IP 和 CIDR 范围）
    blacklist_cache: Arc<RwLock<Vec<IpCacheEntry>>>,
    /// 是否启用白名单模式（默认关闭）
    whitelist_mode_enabled: Arc<RwLock<bool>>,
    /// 缓存刷新间隔
    cache_refresh_interval: Duration,
}

impl std::fmt::Debug for IpSecurityService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpSecurityService")
            .field("db", &self.db)
            .field("cache_refresh_interval", &self.cache_refresh_interval)
            .finish_non_exhaustive()
    }
}

impl IpSecurityService {
    /// 创建新的 IP 安全服务
    ///
    /// # 参数
    /// - `db`: 数据库连接
    /// - `audit_service`: 审计服务
    pub async fn new(db: Database, audit_service: Arc<AuditService>) -> Self {
        let whitelist_cache = Arc::new(RwLock::new(Vec::new()));
        let blacklist_cache = Arc::new(RwLock::new(Vec::new()));
        let whitelist_mode_enabled = Arc::new(RwLock::new(false));
        let cache_refresh_interval = Duration::from_secs(60); // 每分钟刷新缓存

        let service = Self {
            db,
            audit_service,
            whitelist_cache,
            blacklist_cache,
            whitelist_mode_enabled,
            cache_refresh_interval,
        };

        // 初始化缓存
        if let Err(e) = service.refresh_cache().await {
            error!("Failed to initialize IP security cache: {}", e);
        }

        // 启动后台刷新任务
        service.start_cache_refresh_task();

        info!("IpSecurityService initialized");
        service
    }

    /// 启动缓存刷新后台任务
    fn start_cache_refresh_task(&self) {
        let whitelist_cache = Arc::clone(&self.whitelist_cache);
        let blacklist_cache = Arc::clone(&self.blacklist_cache);
        let db = self.db.clone();
        let interval_duration = self.cache_refresh_interval;

        tokio::spawn(async move {
            let mut ticker = interval(interval_duration);
            loop {
                ticker.tick().await;
                debug!("Refreshing IP security cache...");

                // 刷新白名单缓存
                match Self::load_ip_list_to_cache(&db, IpListType::Whitelist, &whitelist_cache)
                    .await
                {
                    Ok(count) => debug!("Whitelist cache refreshed: {} entries", count),
                    Err(e) => error!("Failed to refresh whitelist cache: {}", e),
                }

                // 刷新黑名单缓存
                match Self::load_ip_list_to_cache(&db, IpListType::Blacklist, &blacklist_cache)
                    .await
                {
                    Ok(count) => debug!("Blacklist cache refreshed: {} entries", count),
                    Err(e) => error!("Failed to refresh blacklist cache: {}", e),
                }
            }
        });
    }

    /// 从数据库加载 IP 列表到缓存
    async fn load_ip_list_to_cache(
        db: &Database,
        list_type: IpListType,
        cache: &RwLock<Vec<IpCacheEntry>>,
    ) -> Result<usize> {
        let list_type_str = match list_type {
            IpListType::Whitelist => "whitelist",
            IpListType::Blacklist => "blacklist",
        };

        // 查询所有条目（包括 CIDR 范围）
        let rows = sqlx::query_as::<_, (String, Option<String>)>(
            r#"
            SELECT ip_address, ip_range_cidr FROM ip_lists
            WHERE list_type = $1
            AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(list_type_str)
        .fetch_all(db.pool())
        .await?;

        let mut cache_guard = cache.write().await;
        cache_guard.clear();

        for (ip_str, cidr_opt) in rows {
            // 如果有 CIDR 范围，优先使用 CIDR
            if let Some(cidr) = cidr_opt {
                if let Ok(net) = cidr.parse::<IpNet>() {
                    cache_guard.push(IpCacheEntry::Cidr(net));
                }
            } else if let Ok(ip) = ip_str.parse::<IpAddr>() {
                // 否则使用单个 IP
                cache_guard.push(IpCacheEntry::Single(ip));
            }
        }

        Ok(cache_guard.len())
    }

    /// 刷新缓存（手动触发）
    pub async fn refresh_cache(&self) -> Result<()> {
        Self::load_ip_list_to_cache(&self.db, IpListType::Whitelist, &self.whitelist_cache).await?;
        Self::load_ip_list_to_cache(&self.db, IpListType::Blacklist, &self.blacklist_cache).await?;
        info!("IP security cache refreshed manually");
        Ok(())
    }

    /// 检查 IP 是否允许连接
    ///
    /// # 检查逻辑
    /// 1. 先检查黑名单（优先级最高）
    /// 2. 如果启用了白名单模式，检查白名单
    /// 3. 默认允许连接
    pub async fn check_ip(&self, ip: IpAddr) -> Result<IpCheckResult> {
        // 1. 检查黑名单（内存缓存检查，支持 CIDR 范围）
        {
            let blacklist = self.blacklist_cache.read().await;
            for entry in blacklist.iter() {
                if entry.contains(ip) {
                    return Ok(IpCheckResult::BlockedByBlacklist {
                        reason: "IP address is in blacklist".to_string(),
                    });
                }
            }
        }

        // 2. 检查白名单模式
        let whitelist_enabled = *self.whitelist_mode_enabled.read().await;
        if whitelist_enabled {
            let whitelist = self.whitelist_cache.read().await;
            let mut found = false;
            for entry in whitelist.iter() {
                if entry.contains(ip) {
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(IpCheckResult::NotInWhitelist);
            }
        }

        Ok(IpCheckResult::Allowed)
    }

    /// 记录 IP 检查事件到审计系统
    pub async fn log_ip_check(&self, ip: IpAddr, result: &IpCheckResult, user_agent: Option<&str>) {
        if matches!(result, IpCheckResult::Allowed) {
            return; // 正常连接不记录
        }

        let event_type = match result {
            IpCheckResult::BlockedByBlacklist { .. } => AuditEventType::IpBlocked,
            IpCheckResult::NotInWhitelist => AuditEventType::IpWhitelistDenied,
            IpCheckResult::RateLimited => AuditEventType::IpRateLimited,
            IpCheckResult::Allowed => return,
        };

        let description = format!(
            "IP {} access denied: {}",
            ip,
            result.rejection_reason().unwrap_or_default()
        );

        let mut metadata = AuditMetadata::new().with_ip(ip);
        if let Some(ua) = user_agent {
            metadata = metadata.with_user_agent(ua.to_string());
        }

        let severity = event_type.default_severity();
        let log = CreateAuditLogRequest::new(event_type, "ip_security_check", description)
            .with_metadata(metadata)
            .with_severity(severity);

        if let Err(e) = self.audit_service.log_event(log).await {
            error!("Failed to log IP security check: {}", e);
        }
    }

    /// 添加 IP 到列表
    pub async fn add_ip_to_list(
        &self,
        request: CreateIpListRequest,
        created_by: Uuid,
    ) -> Result<IpListEntry> {
        // 验证 CIDR 格式（如果提供）
        if let Some(ref cidr) = request.ip_range_cidr {
            Self::validate_cidr(cidr)?;
        }

        let list_type_str = match request.list_type {
            IpListType::Whitelist => "whitelist",
            IpListType::Blacklist => "blacklist",
        };

        let ip_address_str = request.ip_address.to_string();

        let entry = sqlx::query_as::<_, IpListEntry>(
            r#"
            INSERT INTO ip_lists (ip_address, ip_range_cidr, list_type, description, created_by, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&ip_address_str)
        .bind(request.ip_range_cidr)
        .bind(list_type_str)
        .bind(request.description)
        .bind(created_by)
        .bind(request.expires_at)
        .fetch_one(self.db.pool())
        .await?;

        // 刷新缓存
        self.refresh_cache().await?;

        // 记录审计日志
        let log = CreateAuditLogRequest::new(
            AuditEventType::IpListAdded,
            "add_ip_to_list",
            format!("Added IP {} to {:?}", request.ip_address, request.list_type),
        )
        .with_metadata(AuditMetadata::new().with_ip(request.ip_address));

        if let Err(e) = self.audit_service.log_event(log).await {
            error!("Failed to log IP list addition: {}", e);
        }

        info!("Added IP {} to {:?}", request.ip_address, request.list_type);
        Ok(entry)
    }

    /// 批量添加 IP 到列表
    pub async fn batch_add_to_list(
        &self,
        request: BatchIpListRequest,
        created_by: Uuid,
    ) -> Result<BatchIpListResponse> {
        let mut success_count = 0;
        let mut failed_ips = Vec::new();

        for entry in request.entries {
            match self.add_ip_to_list(entry.clone(), created_by).await {
                Ok(_) => success_count += 1,
                Err(_) => failed_ips.push(entry.ip_address.to_string()),
            }
        }

        Ok(BatchIpListResponse {
            success_count,
            failed_count: failed_ips.len(),
            failed_ips,
        })
    }

    /// 从列表中移除 IP
    pub async fn remove_from_list(&self, id: Uuid, _removed_by: Uuid) -> Result<()> {
        // 先获取条目信息用于审计日志
        let entry = sqlx::query_as::<_, IpListEntry>("SELECT * FROM ip_lists WHERE id = $1")
            .bind(id)
            .fetch_optional(self.db.pool())
            .await?;

        let result = sqlx::query("DELETE FROM ip_lists WHERE id = $1")
            .bind(id)
            .execute(self.db.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        // 刷新缓存
        self.refresh_cache().await?;

        // 记录审计日志
        if let Some(entry) = entry {
            let ip_addr = entry.ip().unwrap_or_else(|| "0.0.0.0".parse().unwrap());
            let log = CreateAuditLogRequest::new(
                AuditEventType::IpListRemoved,
                "remove_from_list",
                format!("Removed IP {} from {:?}", entry.ip_address, entry.list_type),
            )
            .with_metadata(AuditMetadata::new().with_ip(ip_addr));

            if let Err(e) = self.audit_service.log_event(log).await {
                error!("Failed to log IP list removal: {}", e);
            }
        }

        info!("Removed IP list entry {}", id);
        Ok(())
    }

    /// 更新 IP 列表项
    pub async fn update_ip_entry(
        &self,
        id: Uuid,
        request: UpdateIpListRequest,
        _updated_by: Uuid,
    ) -> Result<IpListEntry> {
        let entry = sqlx::query_as::<_, IpListEntry>(
            r#"
            UPDATE ip_lists
            SET description = COALESCE($1, description),
                expires_at = COALESCE($2, expires_at),
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(request.description)
        .bind(request.expires_at)
        .bind(id)
        .fetch_one(self.db.pool())
        .await
        .map_err(|_| AppError::NotFound)?;

        // 刷新缓存
        self.refresh_cache().await?;

        // 记录审计日志
        let ip_addr = entry.ip().unwrap_or_else(|| "0.0.0.0".parse().unwrap());
        let log = CreateAuditLogRequest::new(
            AuditEventType::IpListUpdated,
            "update_ip_entry",
            format!("Updated IP list entry {}", id),
        )
        .with_metadata(AuditMetadata::new().with_ip(ip_addr));

        if let Err(e) = self.audit_service.log_event(log).await {
            error!("Failed to log IP list update: {}", e);
        }

        info!("Updated IP list entry {}", id);
        Ok(entry)
    }

    /// 查询 IP 列表
    pub async fn query_ip_list(
        &self,
        query: IpListQuery,
    ) -> Result<(Vec<IpListEntryResponse>, i64)> {
        let mut sql = String::from(
            r#"
            SELECT 
                i.*,
                u.username as created_by_username
            FROM ip_lists i
            LEFT JOIN users u ON i.created_by = u.id
            WHERE 1=1
            "#,
        );

        if let Some(ref list_type) = query.list_type {
            sql.push_str(&format!(" AND i.list_type = '{}'", list_type));
        }

        if let Some(ip) = query.ip_address {
            sql.push_str(&format!(" AND i.ip_address::text = '{}'", ip));
        }

        sql.push_str(" ORDER BY i.created_at DESC");

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let entries = sqlx::query_as::<_, IpListEntryResponse>(&sql)
            .fetch_all(self.db.pool())
            .await?;

        // 获取总数
        let count_sql = r#"
            SELECT COUNT(*) FROM ip_lists i
            WHERE 1=1
        "#;
        let mut count_sql = count_sql.to_string();
        if let Some(ref list_type) = query.list_type {
            count_sql.push_str(&format!(" AND i.list_type = '{}'", list_type));
        }
        if let Some(ip) = query.ip_address {
            count_sql.push_str(&format!(" AND i.ip_address::text = '{}'", ip));
        }

        let total = sqlx::query_scalar::<_, i64>(&count_sql)
            .fetch_one(self.db.pool())
            .await?;

        Ok((entries, total))
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<IpSecurityStats> {
        let total_whitelist = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ip_lists WHERE list_type = 'whitelist'",
        )
        .fetch_one(self.db.pool())
        .await?;

        let total_blacklist = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ip_lists WHERE list_type = 'blacklist'",
        )
        .fetch_one(self.db.pool())
        .await?;

        let expired_entries = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ip_lists WHERE expires_at IS NOT NULL AND expires_at <= NOW()",
        )
        .fetch_one(self.db.pool())
        .await?;

        let active_entries = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ip_lists WHERE expires_at IS NULL OR expires_at > NOW()",
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(IpSecurityStats {
            total_whitelist,
            total_blacklist,
            expired_entries,
            active_entries,
        })
    }

    /// 检查 IP 状态（管理 API 使用）
    pub async fn check_ip_status(&self, request: IpCheckRequest) -> Result<IpCheckResponse> {
        let result = self.check_ip(request.ip_address).await?;

        // 查询 IP 在哪个列表中
        let ip_str = request.ip_address.to_string();

        let list_type = sqlx::query_scalar::<_, String>(
            r#"
            SELECT list_type FROM ip_lists
            WHERE ip_address::text = $1
            AND (expires_at IS NULL OR expires_at > NOW())
            LIMIT 1
            "#,
        )
        .bind(ip_str)
        .fetch_optional(self.db.pool())
        .await?;

        let list_type = list_type.and_then(|s| match s.as_str() {
            "whitelist" => Some(IpListType::Whitelist),
            "blacklist" => Some(IpListType::Blacklist),
            _ => None,
        });

        Ok(IpCheckResponse {
            ip_address: request.ip_address,
            allowed: result.is_allowed(),
            reason: result.rejection_reason(),
            list_type,
        })
    }

    /// 设置白名单模式
    pub async fn set_whitelist_mode(&self, enabled: bool) {
        let mut guard = self.whitelist_mode_enabled.write().await;
        *guard = enabled;
        info!(
            "Whitelist mode {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// 获取白名单模式状态
    pub async fn is_whitelist_mode_enabled(&self) -> bool {
        *self.whitelist_mode_enabled.read().await
    }

    /// 清理过期的 IP 条目
    pub async fn cleanup_expired_entries(&self) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM ip_lists WHERE expires_at IS NOT NULL AND expires_at <= NOW()",
        )
        .execute(self.db.pool())
        .await?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            info!("Cleaned up {} expired IP list entries", deleted);
            self.refresh_cache().await?;
        }

        Ok(deleted)
    }

    /// 验证 CIDR 格式
    fn validate_cidr(cidr: &str) -> Result<()> {
        // 使用 ipnet 验证 CIDR 格式
        match cidr.parse::<IpNet>() {
            Ok(_) => Ok(()),
            Err(e) => Err(AppError::Validation(format!("Invalid CIDR format: {}", e))),
        }
    }
}
