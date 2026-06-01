use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Datelike, Timelike, Utc};
use serde_json::json;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::{ConfigChangeEvent, ConfigManager};
use crate::db::Database;
use crate::error::{AppError, Result};
use crate::models::audit::{
    AlertQuery, AlertRule, AlertStatus, AuditAlert, AuditAlertResponse, AuditEventType, AuditLog,
    AuditLogQuery, AuditMetadata, AuditSeverity, AuditStats, CreateAlertRequest,
    CreateAuditLogRequest, DailyCount, EventTypeCount, SeverityCount as AuditSeverityCount, UpdateAlertRuleRequest,
};
use crate::models::user::{UserInfo, UserRole};
use crate::redis::{AuditLogStreamMessage, StreamManager};
use crate::services::alert_engine::AlertEngine;
use crate::services::alert_handler::AlertHandler;
use crate::services::notification_service::NotificationService;

/// 审计服务
/// 负责审计日志的记录、查询、导出和告警管理
/// 支持 Redis Stream 异步写入和本地 Buffer 降级
pub struct AuditService {
    db: Database,
    notification_service: Arc<NotificationService>,
    config_manager: Arc<ConfigManager>,
    log_buffer: Arc<RwLock<Vec<CreateAuditLogRequest>>>,
    buffer_size: usize,
    flush_interval: Duration,
    alert_engine: Arc<AlertEngine>,
    alert_handler: Arc<AlertHandler>,
    /// Redis Stream 管理器（可选）
    stream_manager: Option<Arc<StreamManager>>,
    /// 是否使用 Redis Stream
    use_stream: bool,
    /// 节点 ID
    node_id: String,
}

impl std::fmt::Debug for AuditService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuditService")
            .field("db", &self.db)
            .field("buffer_size", &self.buffer_size)
            .field("flush_interval", &self.flush_interval)
            .finish_non_exhaustive()
    }
}

impl AuditService {
    /// 创建新的审计服务
    ///
    /// # 参数
    /// - `db`: 数据库连接
    /// - `notification_service`: 通知服务
    /// - `config_manager`: 配置管理器
    /// - `stream_manager`: Redis Stream 管理器（可选）
    pub async fn new(
        db: Database,
        notification_service: Arc<NotificationService>,
        config_manager: Arc<ConfigManager>,
        stream_manager: Option<Arc<StreamManager>>,
    ) -> Self {
        let log_buffer = Arc::new(RwLock::new(Vec::new()));
        let alert_engine = Arc::new(AlertEngine::new(db.clone()));
        let alert_handler = Arc::new(AlertHandler::new(db.clone(), notification_service.clone()));

        // 从配置管理器加载审计配置
        let config = config_manager.get_config().await;
        let audit_config = &config.audit;
        let buffer_size = audit_config.buffer_size;
        let flush_interval = Duration::from_secs(audit_config.flush_interval_seconds);

        // 确定是否使用 Redis Stream
        let use_stream = stream_manager.is_some();
        let node_id = stream_manager
            .as_ref()
            .map(|_s| {
                // 尝试从 StreamManager 获取 node_id
                "node-audit".to_string()
            })
            .unwrap_or_else(|| format!("node-{}", Uuid::new_v4()));

        if use_stream {
            info!("AuditService initialized with Redis Stream support");
        } else {
            info!("AuditService initialized with direct database writes");
        }

        let service = Self {
            db,
            notification_service,
            config_manager,
            log_buffer,
            buffer_size,
            flush_interval,
            alert_engine,
            alert_handler,
            stream_manager,
            use_stream,
            node_id,
        };

        service.start_background_tasks();
        service
    }

    /// 启动后台任务
    fn start_background_tasks(&self) {
        // 启动日志刷新任务
        let buffer = Arc::clone(&self.log_buffer);
        let db = self.db.clone();
        let interval_duration = self.flush_interval;

        tokio::spawn(async move {
            let mut ticker = interval(interval_duration);
            loop {
                ticker.tick().await;
                if let Err(e) = flush_logs(&db, &buffer).await {
                    error!("Failed to flush audit logs: {}", e);
                }
            }
        });

        // 启动配置监听任务
        self.start_config_listener();

        // 启动自动清理任务
        self.start_cleanup_task();
    }

    /// 启动配置监听器，支持热更新
    fn start_config_listener(&self) {
        let config_manager = self.config_manager.clone();

        tokio::spawn(async move {
            // 监听配置变更事件
            let mut rx = config_manager.subscribe_config_changes();
            while let Ok(event) = rx.recv().await {
                if let ConfigChangeEvent::ConfigUpdated { key, new_value, .. } = event {
                    if key.starts_with("audit.") {
                        info!("Audit configuration changed: {}", key);

                        // 如果缓冲区大小变更，需要重新初始化（此处简化处理，仅记录日志）
                        if key == "audit.buffer_size" {
                            if let Ok(new_size) = new_value.parse::<usize>() {
                                info!("Audit buffer size changed to: {}", new_size);
                            }
                        }

                        // 如果刷新间隔变更，需要重启刷新任务（此处简化处理）
                        if key == "audit.flush_interval_seconds" {
                            if let Ok(new_interval) = new_value.parse::<u64>() {
                                info!("Audit flush interval changed to: {} seconds", new_interval);
                            }
                        }
                    }
                }
            }
        });
    }

    /// 重新加载配置（用于手动触发）
    pub async fn reload_config(&self) -> Result<()> {
        let config = self.config_manager.get_config().await;
        let audit_config = &config.audit;

        info!(
            "Audit configuration reloaded: enabled={}, buffer_size={}, flush_interval={}s, alert_enabled={}",
            audit_config.enabled,
            audit_config.buffer_size,
            audit_config.flush_interval_seconds,
            audit_config.alert_enabled
        );

        Ok(())
    }

    /// 启动自动清理任务
    fn start_cleanup_task(&self) {
        let service = Arc::new(self.clone_for_task());
        let config_manager = self.config_manager.clone();

        tokio::spawn(async move {
            // 每天检查一次
            let mut ticker = interval(Duration::from_secs(86400)); // 24 小时

            loop {
                ticker.tick().await;

                // 从配置中读取保留天数
                let config = config_manager.get_config().await;
                let retention_days = config.audit.log_retention_days;
                let auto_archive = config.audit.auto_archive_enabled;
                let archive_hour = config.audit.archive_hour;
                drop(config);

                if retention_days <= 0 {
                    debug!("Audit log retention is disabled (0 days)");
                    continue;
                }

                // 检查当前小时是否为归档小时
                let now = Utc::now();
                if auto_archive && now.hour() == archive_hour as u32 {
                    // 执行归档
                    let before = now - chrono::Duration::days(retention_days as i64);
                    if let Err(e) = service.archive_old_logs(before, None).await {
                        error!("Failed to archive old audit logs: {}", e);
                    }
                } else if !auto_archive {
                    // 直接清理
                    let before = now - chrono::Duration::days(retention_days as i64);
                    if let Err(e) = service.cleanup_old_logs(before).await {
                        error!("Failed to cleanup old audit logs: {}", e);
                    }
                }
            }
        });
    }

    /// 为任务克隆创建轻量级引用
    fn clone_for_task(&self) -> Self {
        Self {
            db: self.db.clone(),
            notification_service: self.notification_service.clone(),
            config_manager: self.config_manager.clone(),
            log_buffer: self.log_buffer.clone(),
            buffer_size: self.buffer_size,
            flush_interval: self.flush_interval,
            alert_engine: self.alert_engine.clone(),
            alert_handler: self.alert_handler.clone(),
            stream_manager: self.stream_manager.clone(),
            use_stream: self.use_stream,
            node_id: self.node_id.clone(),
        }
    }

    // ==================== 审计日志记录 ====================

    /// 记录审计日志（异步批量写入）
    /// 优先使用 Redis Stream，不可用时降级到本地 Buffer
    /// 同时触发告警规则引擎分析
    pub async fn log_event(&self, log: CreateAuditLogRequest) -> Result<()> {
        // 如果启用了 Redis Stream，先尝试写入 Stream
        if self.use_stream {
            match self.send_to_stream(&log).await {
                Ok(_) => {
                    debug!("Audit log sent to Redis Stream");
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to send audit log to Redis Stream: {}, falling back to buffer",
                        e
                    );
                    // 降级到本地 Buffer
                }
            }
        }

        // 使用本地 Buffer（降级方案或直接写入模式）
        let mut buffer = self.log_buffer.write().await;
        buffer.push(log);

        if buffer.len() >= self.buffer_size {
            drop(buffer);
            self.flush_buffer().await?;
        }

        Ok(())
    }

    /// 将审计日志发送到 Redis Stream
    ///
    /// # 参数
    /// - `log`: 审计日志请求
    ///
    /// # 返回
    /// - 发送成功返回 Ok(())
    /// - 发送失败返回 Err
    async fn send_to_stream(&self, log: &CreateAuditLogRequest) -> anyhow::Result<()> {
        if let Some(ref manager) = self.stream_manager {
            let mut producer = manager
                .get_producer()
                .await
                .ok_or_else(|| anyhow::anyhow!("Stream producer not available"))?;

            let stream_msg = AuditLogStreamMessage {
                id: Uuid::new_v4(),
                event_type: log.event_type.to_string(),
                severity: log.severity().to_string(),
                actor_id: log.actor_id,
                actor_role: log.actor_role.as_ref().map(|r| r.to_string()),
                target_type: log.target_type.clone(),
                target_id: log.target_id,
                action: log.action.clone(),
                description: log.description.clone(),
                metadata: log
                    .metadata
                    .as_ref()
                    .and_then(|m| serde_json::to_value(m).ok()),
                status: log.status.clone().unwrap_or_else(|| "success".to_string()),
                error_message: log.error_message.clone(),
                node_id: self.node_id.clone(),
                timestamp: Utc::now(),
            };

            producer.send(&stream_msg).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Stream manager not available"))
        }
    }

    /// 记录审计日志并触发告警分析（用于需要实时告警的场景）
    pub async fn log_event_with_alert(&self, log: CreateAuditLogRequest) -> Result<()> {
        // 先保存日志
        self.log_event(log.clone()).await?;

        // 转换为 AuditLog 用于告警分析
        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            event_type: log.event_type.clone(),
            severity: log.severity(),
            actor_id: log.actor_id,
            actor_name: None,
            actor_role: log.actor_role,
            target_type: log.target_type.clone(),
            target_id: log.target_id,
            action: log.action.clone(),
            description: log.description.clone(),
            metadata: log
                .metadata
                .as_ref()
                .and_then(|m| serde_json::to_value(m).ok()),
            status: log.status.unwrap_or_else(|| "success".to_string()),
            error_message: log.error_message.clone(),
            created_at: Utc::now(),
        };

        // 触发告警分析
        match self.alert_engine.analyze_log(&audit_log).await {
            Ok(alerts) => {
                for alert in alerts {
                    info!("Alert generated: {} - {}", alert.id, alert.title);
                }
            }
            Err(e) => {
                error!("Failed to analyze log for alerts: {}", e);
            }
        }

        Ok(())
    }

    /// 立即刷新缓冲区到数据库
    pub async fn flush_buffer(&self) -> Result<()> {
        flush_logs(&self.db, &self.log_buffer).await
    }

    /// 记录用户登录事件
    pub async fn log_user_login(
        &self,
        user_id: Uuid,
        role: UserRole,
        ip: IpAddr,
        user_agent: Option<String>,
        success: bool,
    ) -> Result<()> {
        let mut log = CreateAuditLogRequest::new(
            AuditEventType::UserLogin,
            "login",
            if success {
                format!("用户 {} 登录成功", user_id)
            } else {
                format!("用户 {} 登录失败", user_id)
            },
        )
        .with_actor(user_id, role);

        let metadata = AuditMetadata::new().with_ip(ip);
        let metadata = if let Some(ua) = user_agent {
            metadata.with_user_agent(ua)
        } else {
            metadata
        };
        log = log.with_metadata(metadata);

        if !success {
            log = log.with_error("登录失败");
        }

        self.log_event(log).await
    }

    /// 记录用户登出事件
    pub async fn log_user_logout(&self, user_id: Uuid, role: UserRole, ip: IpAddr) -> Result<()> {
        let log = CreateAuditLogRequest::new(
            AuditEventType::UserLogout,
            "logout",
            format!("用户 {} 登出", user_id),
        )
        .with_actor(user_id, role)
        .with_metadata(AuditMetadata::new().with_ip(ip));

        self.log_event(log).await
    }

    /// 记录用户注册事件
    pub async fn log_user_register(&self, user_id: Uuid, ip: IpAddr) -> Result<()> {
        let log = CreateAuditLogRequest::new(
            AuditEventType::UserRegister,
            "register",
            format!("新用户注册: {}", user_id),
        )
        .with_actor(user_id, UserRole::User)
        .with_metadata(AuditMetadata::new().with_ip(ip));

        self.log_event(log).await
    }

    /// 记录密码修改事件
    pub async fn log_password_change(
        &self,
        user_id: Uuid,
        role: UserRole,
        ip: IpAddr,
    ) -> Result<()> {
        let log = CreateAuditLogRequest::new(
            AuditEventType::UserPasswordChange,
            "password_change",
            format!("用户 {} 修改密码", user_id),
        )
        .with_actor(user_id, role)
        .with_metadata(AuditMetadata::new().with_ip(ip));

        self.log_event(log).await
    }

    /// 记录管理员操作
    pub async fn log_admin_action(
        &self,
        admin_id: Uuid,
        action: &str,
        target: &str,
        target_id: Uuid,
        ip: IpAddr,
    ) -> Result<()> {
        let event_type = match action {
            "user_disable" => AuditEventType::AdminUserDisable,
            "user_role_change" => AuditEventType::AdminUserRoleChange,
            "user_delete" => AuditEventType::AdminUserDelete,
            "room_delete" => AuditEventType::AdminRoomDelete,
            "message_delete" => AuditEventType::AdminMessageDelete,
            "config_update" => AuditEventType::AdminConfigUpdate,
            _ => AuditEventType::AdminConfigUpdate,
        };

        let log = CreateAuditLogRequest::new(
            event_type,
            action,
            format!(
                "管理员 {} 执行 {} 操作，目标: {}({})",
                admin_id, action, target, target_id
            ),
        )
        .with_actor(admin_id, UserRole::Admin)
        .with_target(target, target_id)
        .with_metadata(AuditMetadata::new().with_ip(ip));

        self.log_event(log).await
    }

    /// 记录用户相关操作
    pub async fn log_user_action(
        &self,
        user_id: Uuid,
        role: UserRole,
        action: &str,
        ip: IpAddr,
    ) -> Result<()> {
        let event_type = match action {
            "friend_request_send" => AuditEventType::UserFriendRequestSend,
            "friend_request_accept" => AuditEventType::UserFriendRequestAccept,
            "friend_request_reject" => AuditEventType::UserFriendRequestReject,
            "friend_request_cancel" => AuditEventType::UserFriendRequestCancel,
            "friend_remove" => AuditEventType::UserFriendRemove,
            _ => AuditEventType::UserProfileUpdate,
        };

        let log = CreateAuditLogRequest::new(
            event_type,
            action,
            format!("用户 {} 执行 {} 操作", user_id, action),
        )
        .with_actor(user_id, role)
        .with_metadata(AuditMetadata::new().with_ip(ip));

        self.log_event(log).await
    }

    /// 记录房间操作
    pub async fn log_room_action(
        &self,
        user_id: Uuid,
        role: UserRole,
        room_id: Uuid,
        action: &str,
        ip: IpAddr,
    ) -> Result<()> {
        let event_type = match action {
            "create" => AuditEventType::RoomCreate,
            "delete" => AuditEventType::RoomDelete,
            "member_add" => AuditEventType::RoomMemberAdd,
            "member_remove" => AuditEventType::RoomMemberRemove,
            "member_role_change" => AuditEventType::RoomMemberRoleChange,
            _ => AuditEventType::RoomCreate,
        };

        let log = CreateAuditLogRequest::new(
            event_type,
            action,
            format!("用户 {} 在房间 {} 执行 {} 操作", user_id, room_id, action),
        )
        .with_actor(user_id, role)
        .with_target("room", room_id)
        .with_metadata(AuditMetadata::new().with_ip(ip));

        self.log_event(log).await
    }

    /// 记录消息操作
    pub async fn log_message_action(
        &self,
        user_id: Uuid,
        role: UserRole,
        message_id: Uuid,
        action: &str,
        ip: IpAddr,
    ) -> Result<()> {
        let event_type = match action {
            "send" => AuditEventType::MessageSend,
            "edit" => AuditEventType::MessageEdit,
            "delete" => AuditEventType::MessageDelete,
            "report" => AuditEventType::MessageReport,
            _ => AuditEventType::MessageSend,
        };

        let log = CreateAuditLogRequest::new(
            event_type,
            action,
            format!(
                "用户 {} 对消息 {} 执行 {} 操作",
                user_id, message_id, action
            ),
        )
        .with_actor(user_id, role)
        .with_target("message", message_id)
        .with_metadata(AuditMetadata::new().with_ip(ip));

        self.log_event(log).await
    }

    /// 记录登录失败事件
    pub async fn log_login_failure(&self, email: &str, ip: IpAddr, reason: &str) -> Result<()> {
        let log = CreateAuditLogRequest::new(
            AuditEventType::SystemLoginFailure,
            "login_failure",
            format!("邮箱 {} 登录失败: {}", email, reason),
        )
        .with_metadata(
            AuditMetadata::new()
                .with_ip(ip)
                .with_extra(json!({"email": email, "reason": reason })),
        );

        self.log_event(log).await
    }

    /// 记录未授权访问事件
    pub async fn log_unauthorized_access(
        &self,
        ip: IpAddr,
        path: &str,
        method: &str,
    ) -> Result<()> {
        let log = CreateAuditLogRequest::new(
            AuditEventType::SystemUnauthorizedAccess,
            "unauthorized_access",
            format!("未授权访问: {} {}", method, path),
        )
        .with_metadata(
            AuditMetadata::new()
                .with_ip(ip)
                .with_request(method, path, None)
                .with_extra(json!({"path": path, "method": method })),
        );

        self.log_event(log).await
    }

    /// 记录频率限制触发事件
    pub async fn log_rate_limit_triggered(
        &self,
        ip: IpAddr,
        path: &str,
        user_id: Option<Uuid>,
    ) -> Result<()> {
        let mut log = CreateAuditLogRequest::new(
            AuditEventType::SystemRateLimitTriggered,
            "rate_limit_triggered",
            format!("IP {} 触发频率限制: {}", ip, path),
        )
        .with_metadata(
            AuditMetadata::new()
                .with_ip(ip)
                .with_request("", path, None),
        );

        if let Some(uid) = user_id {
            log.actor_id = Some(uid);
        }

        self.log_event(log).await
    }

    // ==================== 审计日志查询 ====================

    /// 查询审计日志（支持过滤、分页、排序）
    pub async fn query_logs(&self, query: AuditLogQuery) -> Result<(Vec<AuditLog>, i64)> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let pool = self.db.pool();

        // 使用动态查询构建，LEFT JOIN 获取用户名
        let mut logs_query = sqlx::QueryBuilder::new(
            "SELECT al.id, al.event_type, al.severity, al.actor_id, u.username as actor_name, 
             al.actor_role, al.target_type, al.target_id, al.action, al.description, 
             al.metadata, al.status, al.error_message, al.created_at 
             FROM audit_logs al 
             LEFT JOIN users u ON al.actor_id = u.id 
             WHERE 1=1",
        );

        let mut count_query = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM audit_logs WHERE 1=1");

        if let Some(event_type) = &query.event_type {
            logs_query
                .push(" AND event_type = ")
                .push_bind(event_type.clone());
            count_query
                .push(" AND event_type = ")
                .push_bind(event_type.clone());
        }

        if let Some(severity) = &query.severity {
            logs_query
                .push(" AND severity = ")
                .push_bind(severity.clone());
            count_query
                .push(" AND severity = ")
                .push_bind(severity.clone());
        }

        if let Some(actor_id) = query.actor_id {
            logs_query.push(" AND actor_id = ").push_bind(actor_id);
            count_query.push(" AND actor_id = ").push_bind(actor_id);
        }

        if let Some(target_id) = query.target_id {
            logs_query.push(" AND target_id = ").push_bind(target_id);
            count_query.push(" AND target_id = ").push_bind(target_id);
        }

        if let Some(target_type) = &query.target_type {
            logs_query
                .push(" AND target_type = ")
                .push_bind(target_type.clone());
            count_query
                .push(" AND target_type = ")
                .push_bind(target_type.clone());
        }

        if let Some(status) = &query.status {
            logs_query.push(" AND status = ").push_bind(status.clone());
            count_query.push(" AND status = ").push_bind(status.clone());
        }

        if let Some(start_time) = query.start_time {
            logs_query.push(" AND created_at >= ").push_bind(start_time);
            count_query
                .push(" AND created_at >= ")
                .push_bind(start_time);
        }

        if let Some(end_time) = query.end_time {
            logs_query.push(" AND created_at <= ").push_bind(end_time);
            count_query.push(" AND created_at <= ").push_bind(end_time);
        }

        logs_query.push(" ORDER BY created_at DESC");
        logs_query.push(" LIMIT ").push_bind(limit);
        logs_query.push(" OFFSET ").push_bind(offset);

        let logs: Vec<AuditLog> = logs_query
            .build_query_as()
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        Ok((logs, total))
    }

    /// 获取单条审计日志详情
    pub async fn get_log_by_id(&self, log_id: Uuid) -> Result<Option<AuditLog>> {
        let pool = self.db.pool();

        let log: Option<AuditLog> = sqlx::query_as(
            "SELECT id, event_type, severity, actor_id, actor_name, actor_role, target_type, target_id,
             action, description, metadata, status, error_message, created_at
             FROM audit_logs WHERE id = $1",
        )
        .bind(log_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(log)
    }

    // ==================== 审计统计 ====================

    /// 获取审计统计信息
    pub async fn get_audit_stats(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<AuditStats> {
        let pool = self.db.pool();

        let start = start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
        let end = end_time.unwrap_or_else(Utc::now);

        let total_logs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs WHERE created_at >= $1 AND created_at <= $2",
        )
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        let severity_counts: Vec<(AuditSeverity, i64)> = sqlx::query_as(
            "SELECT severity, COUNT(*) as count FROM audit_logs 
             WHERE created_at >= $1 AND created_at <= $2 
             GROUP BY severity ORDER BY severity",
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        let logs_by_severity: Vec<AuditSeverityCount> = severity_counts
            .into_iter()
            .map(|(severity, count)| AuditSeverityCount { severity, count })
            .collect();

        let event_type_counts: Vec<(AuditEventType, i64)> = sqlx::query_as(
            "SELECT event_type, COUNT(*) as count FROM audit_logs 
             WHERE created_at >= $1 AND created_at <= $2 
             GROUP BY event_type ORDER BY count DESC",
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        let logs_by_event_type: Vec<EventTypeCount> = event_type_counts
            .into_iter()
            .map(|(event_type, count)| EventTypeCount { event_type, count })
            .collect();

        let daily_counts: Vec<(String, i64)> = sqlx::query_as(
            "SELECT DATE(created_at)::text as date, COUNT(*) as count FROM audit_logs 
             WHERE created_at >= $1 AND created_at <= $2 
             GROUP BY DATE(created_at) ORDER BY date",
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        let logs_by_day: Vec<DailyCount> = daily_counts
            .into_iter()
            .map(|(date, count)| DailyCount { date, count })
            .collect();

        // 计算今日开始时间（当天 00:00:00）
        let today_start = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();
        let today_logs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE created_at >= $1")
                .bind(today_start)
                .fetch_one(pool)
                .await
                .map_err(AppError::Database)?;

        // 计算本周开始时间（7 天前）
        let week_start = Utc::now() - chrono::Duration::days(7);
        let week_logs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE created_at >= $1")
                .bind(week_start)
                .fetch_one(pool)
                .await
                .map_err(AppError::Database)?;

        // 计算本月开始时间（当月 1 号 00:00:00）
        let now = Utc::now();
        let month_start = now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();
        let month_logs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE created_at >= $1")
                .bind(month_start)
                .fetch_one(pool)
                .await
                .map_err(AppError::Database)?;

        let alerts_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_alerts")
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        let new_alerts_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_alerts WHERE status = 'new'")
                .fetch_one(pool)
                .await
                .map_err(AppError::Database)?;

        Ok(AuditStats {
            total_logs,
            today_logs,
            week_logs,
            month_logs,
            logs_by_severity,
            logs_by_event_type,
            logs_by_day,
            alerts_count,
            new_alerts_count,
        })
    }

    // ==================== 日志导出 ====================

    /// 导出审计日志为 JSON
    pub async fn export_logs_json(&self, query: AuditLogQuery) -> Result<String> {
        let (logs, _) = self.query_logs(query).await?;
        serde_json::to_string_pretty(&logs).map_err(|_| AppError::Internal)
    }

    /// 导出审计日志为 CSV
    pub async fn export_logs_csv(&self, query: AuditLogQuery) -> Result<String> {
        let (logs, _) = self.query_logs(query).await?;

        let mut csv = String::from("id,event_type,severity,actor_id,actor_role,target_type,target_id,action,description,status,created_at\n");

        for log in logs {
            csv.push_str(&format!(
                "{},{:?},{:?},{},{},{},{},{},{},{},{}\n",
                log.id,
                log.event_type,
                log.severity,
                log.actor_id.map(|id| id.to_string()).unwrap_or_default(),
                log.actor_role
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_default(),
                log.target_type.unwrap_or_default(),
                log.target_id.map(|id| id.to_string()).unwrap_or_default(),
                log.action,
                log.description.replace(',', ";"),
                log.status,
                log.created_at.to_rfc3339()
            ));
        }

        Ok(csv)
    }

    // ==================== 告警管理 ====================

    /// 创建安全告警
    pub async fn create_alert(&self, alert: CreateAlertRequest) -> Result<AuditAlert> {
        let pool = self.db.pool();

        let alert: AuditAlert = sqlx::query_as(
            "INSERT INTO audit_alerts 
             (rule_id, alert_type, severity, title, description, related_logs, source_ip, affected_user_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, rule_id, alert_type, severity, title, description, related_logs, 
                       source_ip, affected_user_id, status, acknowledged_by, acknowledged_at, 
                       resolved_by, resolved_at, created_at, updated_at"
        )
        .bind(alert.rule_id)
        .bind(&alert.alert_type)
        .bind(&alert.severity)
        .bind(&alert.title)
        .bind(&alert.description)
        .bind(&alert.related_logs)
        .bind(alert.source_ip)
        .bind(alert.affected_user_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        info!("Created security alert: {} - {}", alert.id, alert.title);

        // 使用告警处理器处理通知
        if let Err(e) = self.alert_handler.handle_alert(&alert).await {
            error!("Failed to handle alert notification: {}", e);
        }

        Ok(alert)
    }

    /// 查询告警列表
    pub async fn query_alerts(&self, query: AlertQuery) -> Result<(Vec<AuditAlertResponse>, i64)> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut sql = String::from(
            "SELECT id, rule_id, alert_type, severity, title, description, related_logs,
             source_ip, affected_user_id, status, acknowledged_by, acknowledged_at,
             resolved_by, resolved_at, created_at, updated_at
             FROM audit_alerts WHERE 1=1",
        );
        let mut count_sql = String::from("SELECT COUNT(*) FROM audit_alerts WHERE 1=1");

        if let Some(status) = &query.status {
            sql.push_str(&format!(
                " AND status = '{}'",
                format!("{:?}", status).to_lowercase()
            ));
            count_sql.push_str(&format!(
                " AND status = '{}'",
                format!("{:?}", status).to_lowercase()
            ));
        }

        if let Some(severity) = &query.severity {
            sql.push_str(&format!(
                " AND severity = '{}'",
                format!("{:?}", severity).to_lowercase()
            ));
            count_sql.push_str(&format!(
                " AND severity = '{}'",
                format!("{:?}", severity).to_lowercase()
            ));
        }

        if let Some(alert_type) = &query.alert_type {
            sql.push_str(&format!(" AND alert_type = '{}'", alert_type));
            count_sql.push_str(&format!(" AND alert_type = '{}'", alert_type));
        }

        if let Some(user_id) = query.affected_user_id {
            sql.push_str(&format!(" AND affected_user_id = '{}'", user_id));
            count_sql.push_str(&format!(" AND affected_user_id = '{}'", user_id));
        }

        if let Some(start_time) = query.start_time {
            sql.push_str(&format!(" AND created_at >= '{}'", start_time));
            count_sql.push_str(&format!(" AND created_at >= '{}'", start_time));
        }

        if let Some(end_time) = query.end_time {
            sql.push_str(&format!(" AND created_at <= '{}'", end_time));
            count_sql.push_str(&format!(" AND created_at <= '{}'", end_time));
        }

        sql.push_str(" ORDER BY created_at DESC");
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let pool = self.db.pool();

        let alerts: Vec<AuditAlert> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        // 收集所有用户ID
        let mut user_ids: Vec<Uuid> = alerts.iter().filter_map(|a| a.affected_user_id).collect();
        user_ids.extend(alerts.iter().filter_map(|a| a.acknowledged_by));
        user_ids.extend(alerts.iter().filter_map(|a| a.resolved_by));
        user_ids.sort_unstable();
        user_ids.dedup();

        // 批量查询用户信息
        let user_infos = self.get_user_infos(&user_ids).await?;

        // 转换为响应
        let responses: Vec<AuditAlertResponse> = alerts
            .into_iter()
            .map(|alert| {
                let affected_user = alert
                    .affected_user_id
                    .and_then(|id| user_infos.get(&id).cloned());
                let acknowledged_by = alert
                    .acknowledged_by
                    .and_then(|id| user_infos.get(&id).cloned());
                let resolved_by = alert
                    .resolved_by
                    .and_then(|id| user_infos.get(&id).cloned());
                alert.to_response(affected_user, acknowledged_by, resolved_by)
            })
            .collect();

        Ok((responses, total))
    }

    /// 批量获取用户信息
    async fn get_user_infos(
        &self,
        user_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, UserInfo>> {
        if user_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let rows: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, username, avatar_url FROM users WHERE id = ANY($1)
            "#,
        )
        .bind(user_ids)
        .fetch_all(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        let mut map = std::collections::HashMap::new();
        for (id, username, avatar_url) in rows {
            map.insert(id, UserInfo::new(id, username, avatar_url));
        }

        Ok(map)
    }

    /// 更新告警状态
    pub async fn update_alert_status(
        &self,
        alert_id: Uuid,
        status: AlertStatus,
        user_id: Uuid,
    ) -> Result<AuditAlert> {
        let pool = self.db.pool();

        let (acknowledged_by, acknowledged_at, resolved_by, resolved_at) = match status {
            AlertStatus::Acknowledged => (Some(user_id), Some(Utc::now()), None, None),
            AlertStatus::Resolved => {
                let existing: Option<AuditAlert> =
                    sqlx::query_as("SELECT * FROM audit_alerts WHERE id = $1")
                        .bind(alert_id)
                        .fetch_optional(pool)
                        .await
                        .map_err(AppError::Database)?;

                if let Some(alert) = existing {
                    (
                        alert.acknowledged_by,
                        alert.acknowledged_at,
                        Some(user_id),
                        Some(Utc::now()),
                    )
                } else {
                    (None, None, Some(user_id), Some(Utc::now()))
                }
            }
            _ => (None, None, None, None),
        };

        let alert: AuditAlert = sqlx::query_as(
            "UPDATE audit_alerts 
             SET status = $1, acknowledged_by = $2, acknowledged_at = $3, 
                 resolved_by = $4, resolved_at = $5, updated_at = NOW()
             WHERE id = $6
             RETURNING id, rule_id, alert_type, severity, title, description, related_logs, 
                       source_ip, affected_user_id, status, acknowledged_by, acknowledged_at, 
                       resolved_by, resolved_at, created_at, updated_at",
        )
        .bind(&status)
        .bind(acknowledged_by)
        .bind(acknowledged_at)
        .bind(resolved_by)
        .bind(resolved_at)
        .bind(alert_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(alert)
    }

    /// 获取告警规则列表
    pub async fn get_alert_rules(&self) -> Result<Vec<AlertRule>> {
        let pool = self.db.pool();

        let rules: Vec<AlertRule> = sqlx::query_as(
            "SELECT id, name, description, event_type, condition, severity, enabled, 
             cooldown_minutes, notify_admins, created_at, updated_at 
             FROM audit_alert_rules ORDER BY created_at",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rules)
    }

    /// 更新告警规则
    pub async fn update_alert_rule(
        &self,
        rule_id: Uuid,
        updates: UpdateAlertRuleRequest,
    ) -> Result<AlertRule> {
        let pool = self.db.pool();

        let existing: AlertRule = sqlx::query_as("SELECT * FROM audit_alert_rules WHERE id = $1")
            .bind(rule_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        let name = updates.name.unwrap_or(existing.name);
        let description = updates.description.or(existing.description);
        let condition = updates.condition.unwrap_or(existing.condition);
        let severity = updates.severity.unwrap_or(existing.severity);
        let enabled = updates.enabled.unwrap_or(existing.enabled);
        let cooldown_minutes = updates
            .cooldown_minutes
            .unwrap_or(existing.cooldown_minutes);
        let notify_admins = updates.notify_admins.unwrap_or(existing.notify_admins);

        let rule: AlertRule = sqlx::query_as(
            "UPDATE audit_alert_rules 
             SET name = $1, description = $2, condition = $3, severity = $4, 
                 enabled = $5, cooldown_minutes = $6, notify_admins = $7, updated_at = NOW()
             WHERE id = $8
             RETURNING id, name, description, event_type, condition, severity, enabled, 
                       cooldown_minutes, notify_admins, created_at, updated_at",
        )
        .bind(&name)
        .bind(&description)
        .bind(&condition)
        .bind(&severity)
        .bind(enabled)
        .bind(cooldown_minutes)
        .bind(notify_admins)
        .bind(rule_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rule)
    }

    // ==================== 日志归档与清理 ====================

    /// 归档过期日志到冷存储
    /// 将过期日志导出为JSON格式并存储到指定目录，然后从主表中删除
    pub async fn archive_old_logs(
        &self,
        before: DateTime<Utc>,
        archive_dir: Option<&str>,
    ) -> Result<u64> {
        let pool = self.db.pool();
        let archive_dir = archive_dir.unwrap_or("./archives/audit_logs");

        // 确保归档目录存在
        tokio::fs::create_dir_all(archive_dir)
            .await
            .map_err(|e| AppError::Config(format!("创建归档目录失败: {}", e)))?;

        // 查询需要归档的日志
        let logs: Vec<AuditLog> = sqlx::query_as(
            "SELECT id, event_type, severity, actor_id, actor_role, target_type, target_id, 
             action, description, metadata, status, error_message, created_at 
             FROM audit_logs WHERE created_at < $1 ORDER BY created_at",
        )
        .bind(before)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        if logs.is_empty() {
            info!("No audit logs to archive before {}", before);
            return Ok(0);
        }

        // 按日期分组归档
        use std::collections::HashMap;
        let mut logs_by_date: HashMap<String, Vec<&AuditLog>> = HashMap::new();

        for log in &logs {
            let date = log.created_at.format("%Y-%m").to_string();
            logs_by_date.entry(date).or_default().push(log);
        }

        // 写入归档文件
        for (date, date_logs) in logs_by_date {
            let filename = format!("{}/audit_logs_{}.json", archive_dir, date);
            let json_data =
                serde_json::to_string_pretty(&date_logs).map_err(|_| AppError::Internal)?;

            tokio::fs::write(&filename, json_data)
                .await
                .map_err(|e| AppError::Config(format!("写入归档文件失败: {}", e)))?;

            info!("Archived {} audit logs to {}", date_logs.len(), filename);
        }

        // 从主表删除已归档的日志
        let result = sqlx::query("DELETE FROM audit_logs WHERE created_at < $1")
            .bind(before)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        let deleted = result.rows_affected();
        info!("Archived and deleted {} old audit logs", deleted);

        Ok(deleted)
    }

    /// 清理过期日志（直接删除，不归档）
    pub async fn cleanup_old_logs(&self, before: DateTime<Utc>) -> Result<u64> {
        let pool = self.db.pool();

        let result = sqlx::query("DELETE FROM audit_logs WHERE created_at < $1")
            .bind(before)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        let deleted = result.rows_affected();
        info!("Cleaned up {} old audit logs", deleted);

        Ok(deleted)
    }

    // ==================== 告警通知 ====================

    #[allow(dead_code)]
    async fn should_notify_admins(&self, alert: &AuditAlert) -> bool {
        if alert.severity == AuditSeverity::Critical || alert.severity == AuditSeverity::Error {
            return true;
        }

        let pool = self.db.pool();

        if let Ok(Some(rule)) =
            sqlx::query_as::<_, AlertRule>("SELECT * FROM audit_alert_rules WHERE id = $1")
                .bind(alert.rule_id)
                .fetch_optional(pool)
                .await
        {
            return rule.notify_admins;
        }

        true
    }

    /// 通知管理员关于告警的信息
    /// 通过通知服务发送告警信息给所有管理员
    #[allow(dead_code)]
    async fn notify_admins_about_alert(&self, alert: &AuditAlert) -> Result<()> {
        debug!("Notifying admins about alert: {}", alert.id);

        // 获取所有管理员用户
        let pool = self.db.pool();
        let admin_ids: Vec<uuid::Uuid> =
            sqlx::query_scalar("SELECT id FROM users WHERE role IN ('admin', 'super_admin')")
                .fetch_all(pool)
                .await
                .map_err(AppError::Database)?;

        if admin_ids.is_empty() {
            warn!("No admins found to notify about alert {}", alert.id);
            return Ok(());
        }

        // 构建通知内容
        let severity_emoji = match alert.severity {
            AuditSeverity::Critical => "🚨",
            AuditSeverity::Error => "❌",
            AuditSeverity::Warning => "⚠️",
            AuditSeverity::Info => "ℹ️",
        };

        let notification_content = format!(
            "{} 安全告警: {}\n\n类型: {}\n严重程度: {:?}\n描述: {}\n时间: {}\n\n请尽快处理！",
            severity_emoji,
            alert.title,
            alert.alert_type,
            alert.severity,
            alert.description,
            alert.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );

        // 构建系统通知信息
        use crate::services::notification_service::SystemNotificationInfo;
        use crate::websocket::protocol::NotificationType;

        let notification_info = SystemNotificationInfo {
            notification_type: NotificationType::Warning,
            title: format!("安全告警: {}", alert.title),
            content: notification_content,
            data: Some(json!({
                "alert_id": alert.id,
                "alert_type": alert.alert_type,
                "severity": format!("{:?}", alert.severity),
            })),
            created_at: Utc::now(),
        };

        // 发送通知给所有管理员
        if let Err(e) = self
            .notification_service
            .send_system_notification(notification_info, Some(admin_ids))
            .await
        {
            error!("Failed to send alert notification to admins: {}", e);
        }

        info!(
            "Alert notification sent to admins for alert {}: {}",
            alert.id, alert.title
        );

        Ok(())
    }

    /// 统计最近事件数量（用于告警规则引擎）
    pub async fn count_recent_events(
        &self,
        event_type: &AuditEventType,
        actor_id: Option<Uuid>,
        time_window: chrono::Duration,
    ) -> Result<i64> {
        let pool = self.db.pool();
        let since = Utc::now() - time_window;

        let count: i64 = if let Some(user_id) = actor_id {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM audit_logs 
                 WHERE event_type = $1 AND actor_id = $2 AND created_at >= $3",
            )
            .bind(event_type)
            .bind(user_id)
            .bind(since)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM audit_logs 
                 WHERE event_type = $1 AND created_at >= $2",
            )
            .bind(event_type)
            .bind(since)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?
        };

        Ok(count)
    }

    /// 获取告警引擎引用
    pub fn alert_engine(&self) -> &Arc<AlertEngine> {
        &self.alert_engine
    }

    /// 获取安全告警统计
    pub async fn get_security_stats(&self) -> Result<SecurityStats> {
        let pool = self.db.pool();

        // 今日登录失败数
        let failed_logins_today: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM audit_logs
            WHERE event_type = 'system_login_failure'
            AND created_at > NOW() - INTERVAL '1 day'
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        // 待处理告警数
        let pending_alerts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_alerts WHERE status = 'new'",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        // 今日告警数
        let alerts_today: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM audit_alerts
            WHERE created_at > NOW() - INTERVAL '1 day'
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        // 按严重级别统计告警
        let severity_stats: Vec<(AuditSeverity, i64)> = sqlx::query_as(
            r#"
            SELECT severity, COUNT(*) as count
            FROM audit_alerts
            WHERE created_at > NOW() - INTERVAL '7 days'
            GROUP BY severity
            ORDER BY severity
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        let alerts_by_severity: Vec<AuditSeverityCount> = severity_stats
            .into_iter()
            .map(|(severity, count)| AuditSeverityCount { severity, count })
            .collect();

        // 本周审计日志数
        let audit_logs_this_week: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM audit_logs
            WHERE created_at > NOW() - INTERVAL '7 days'
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(SecurityStats {
            failed_logins_today,
            pending_alerts,
            alerts_today,
            alerts_by_severity,
            audit_logs_this_week,
        })
    }
}

/// 安全统计
#[derive(Debug, serde::Serialize)]
pub struct SecurityStats {
    pub failed_logins_today: i64,
    pub pending_alerts: i64,
    pub alerts_today: i64,
    pub alerts_by_severity: Vec<AuditSeverityCount>,
    pub audit_logs_this_week: i64,
}

/// 刷新日志缓冲区到数据库
async fn flush_logs(db: &Database, buffer: &Arc<RwLock<Vec<CreateAuditLogRequest>>>) -> Result<()> {
    let logs_to_flush = {
        let mut buffer = buffer.write().await;
        if buffer.is_empty() {
            return Ok(());
        }
        std::mem::take(&mut *buffer)
    };

    if logs_to_flush.is_empty() {
        return Ok(());
    }

    let pool = db.pool();
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    for log in logs_to_flush {
        let metadata_json = log
            .metadata
            .as_ref()
            .and_then(|m| serde_json::to_value(m).ok());

        if let Err(e) = sqlx::query(
            "INSERT INTO audit_logs 
             (event_type, severity, actor_id, actor_role, target_type, target_id, 
              action, description, metadata, status, error_message)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(&log.event_type)
        .bind(log.severity())
        .bind(log.actor_id)
        .bind(log.actor_role)
        .bind(&log.target_type)
        .bind(log.target_id)
        .bind(&log.action)
        .bind(&log.description)
        .bind(metadata_json)
        .bind(log.status.unwrap_or_else(|| "success".to_string()))
        .bind(&log.error_message)
        .execute(&mut *tx)
        .await
        {
            error!("Failed to insert audit log: {}", e);
        }
    }

    tx.commit().await.map_err(AppError::Database)?;

    Ok(())
}
