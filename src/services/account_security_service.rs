use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use std::net::IpAddr;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::account_security::{
    AccountSecurityOverview, CreateSessionRequest, DeviceInfo, LoginHistory, LoginHistoryEntry,
    RecordLoginRequest, RiskLevel, UserSession,
};
use crate::models::user_settings::NotificationSettings;
use crate::services::notification_service::NotificationService;

/// 账号安全服务
/// 负责管理用户登录设备、登录历史、异地登录检测等功能
#[derive(Debug, Clone)]
pub struct AccountSecurityService {
    pool: PgPool,
}

impl AccountSecurityService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============================================================
    // 会话（设备）管理
    // ============================================================

    /// 创建新的设备会话
    /// 用户登录成功后调用，记录当前设备信息
    pub async fn create_session(
        &self,
        user_id: Uuid,
        request: CreateSessionRequest,
    ) -> Result<UserSession> {
        // 先将该用户的所有会话标记为非当前会话
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_current = false
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        // 创建新会话
        let ip_network = ipnetwork::IpNetwork::from(request.ip_address);

        let session = sqlx::query_as::<_, UserSession>(
            r#"
            INSERT INTO user_sessions (
                user_id, session_token_hash, device_name, device_type,
                ip_address, user_agent, location_info, is_current, is_active, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, true, $8)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&request.session_token_hash)
        .bind(&request.device_name)
        .bind(&request.device_type)
        .bind(ip_network)
        .bind(&request.user_agent)
        .bind(request.location_info)
        .bind(request.expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(session)
    }

    /// 获取用户的所有活跃设备会话
    pub async fn list_user_devices(&self, user_id: Uuid) -> Result<Vec<DeviceInfo>> {
        let sessions = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT *
            FROM user_sessions
            WHERE user_id = $1 AND is_active = true AND expires_at > NOW()
            ORDER BY is_current DESC, last_active_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(sessions.into_iter().map(DeviceInfo::from).collect())
    }

    /// 获取指定设备会话详情
    pub async fn get_session(&self, user_id: Uuid, session_id: Uuid) -> Result<UserSession> {
        let session = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT *
            FROM user_sessions
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        session.ok_or(AppError::NotFound)
    }

    /// 终止指定设备会话（远程登出）
    /// 用户不能终止当前会话
    pub async fn terminate_session(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        // 先检查会话是否存在且不是当前会话
        let session = self.get_session(user_id, session_id).await?;

        if session.is_current {
            return Err(AppError::Validation(
                "不能终止当前会话，如需登出请使用登出功能".to_string(),
            ));
        }

        // 将会话标记为非活跃
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false, is_current = false
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 禁用指定设备
    /// 被禁用的设备无法使用旧 Token 登录，需要重新认证
    /// 用户不能禁用当前会话
    pub async fn block_device(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        // 先检查会话是否存在且不是当前会话
        let session = self.get_session(user_id, session_id).await?;

        if session.is_current {
            return Err(AppError::Validation("不能禁用当前设备".to_string()));
        }

        // 将设备标记为禁用
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_blocked = true, is_active = false, is_current = false
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 启用被禁用的设备
    /// 将设备从禁用状态恢复，但用户需要重新登录
    pub async fn unblock_device(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        // 检查会话是否存在
        let _ = self.get_session(user_id, session_id).await?;

        // 将设备标记为未禁用（但保持非活跃状态，需要重新登录）
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_blocked = false
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 终止除当前会话外的所有会话
    pub async fn terminate_other_sessions(
        &self,
        user_id: Uuid,
        current_session_id: Uuid,
    ) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false, is_current = false
            WHERE user_id = $1 AND id != $2 AND is_active = true
            "#,
        )
        .bind(user_id)
        .bind(current_session_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected() as i64)
    }

    /// 更新会话最后活跃时间
    pub async fn update_session_activity(&self, session_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_active_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 验证会话令牌是否有效
    /// 检查条件：is_active = true, is_blocked = false, 未过期
    pub async fn validate_session(&self, token_hash: &str) -> Result<Option<UserSession>> {
        let session = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT *
            FROM user_sessions
            WHERE session_token_hash = $1 
              AND is_active = true 
              AND is_blocked = false
              AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(session)
    }

    /// 检查设备是否被禁用
    pub async fn is_device_blocked(&self, session_id: Uuid) -> Result<bool> {
        let result: Option<(bool,)> = sqlx::query_as(
            r#"
            SELECT is_blocked FROM user_sessions WHERE id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.map(|r| r.0).unwrap_or(false))
    }

    // ============================================================
    // 登录历史管理
    // ============================================================

    /// 记录登录历史
    pub async fn record_login(
        &self,
        user_id: Uuid,
        request: RecordLoginRequest,
    ) -> Result<LoginHistory> {
        let ip_network = ipnetwork::IpNetwork::from(request.ip_address);

        let history = sqlx::query_as::<_, LoginHistory>(
            r#"
            INSERT INTO login_history (
                user_id, ip_address, device_info, location_info,
                login_status, failure_reason, is_suspicious, risk_level
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(ip_network)
        .bind(request.device_info)
        .bind(request.location_info)
        .bind(request.login_status.as_str())
        .bind(request.failure_reason)
        .bind(request.is_suspicious)
        .bind(request.risk_level.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(history)
    }

    /// 获取用户登录历史
    /// 支持分页，默认按时间倒序
    pub async fn get_login_history(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LoginHistoryEntry>> {
        let history = sqlx::query_as::<_, LoginHistory>(
            r#"
            SELECT *
            FROM login_history
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(history.into_iter().map(LoginHistoryEntry::from).collect())
    }

    /// 获取用户登录历史总数
    pub async fn get_login_history_count(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM login_history WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(count.0)
    }

    /// 获取最近的可疑登录记录
    pub async fn get_recent_suspicious_logins(
        &self,
        user_id: Uuid,
        days: i64,
    ) -> Result<Vec<LoginHistoryEntry>> {
        let since = Utc::now() - Duration::days(days);

        let history = sqlx::query_as::<_, LoginHistory>(
            r#"
            SELECT *
            FROM login_history
            WHERE user_id = $1 AND is_suspicious = true AND created_at > $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(history.into_iter().map(LoginHistoryEntry::from).collect())
    }

    // ============================================================
    // 异地登录检测
    // ============================================================

    /// 检测是否为异地登录
    /// 通过比较当前登录信息与最近成功的登录记录
    pub async fn detect_abnormal_login(
        &self,
        user_id: Uuid,
        ip: IpAddr,
        device_type: &str,
    ) -> Result<(bool, RiskLevel)> {
        // 获取用户最近的成功登录记录
        let recent_logins: Vec<LoginHistory> = sqlx::query_as(
            r#"
            SELECT *
            FROM login_history
            WHERE user_id = $1 AND login_status = 'success'
            ORDER BY created_at DESC
            LIMIT 5
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        // 如果是首次登录，不算异地
        if recent_logins.is_empty() {
            return Ok((false, RiskLevel::Low));
        }

        // 检查 IP 是否匹配
        let ip_matches = recent_logins.iter().any(|login| login.ip() == ip);

        // 检查设备类型是否匹配
        let device_matches = recent_logins.iter().any(|login| {
            login
                .device_info
                .as_ref()
                .and_then(|info| info.get("device_type").and_then(|d| d.as_str()))
                .map(|dt| dt == device_type)
                .unwrap_or(false)
        });

        // 判断风险等级
        let (is_suspicious, risk_level) = if !ip_matches && !device_matches {
            // IP 和设备都不同，高风险
            (true, RiskLevel::High)
        } else if !ip_matches {
            // 仅 IP 不同，中风险
            (true, RiskLevel::Medium)
        } else {
            // IP 相同，低风险
            (false, RiskLevel::Low)
        };

        Ok((is_suspicious, risk_level))
    }

    /// 发送异地登录提醒
    /// 当检测到异地登录时，向用户发送通知
    pub async fn send_abnormal_login_alert(
        &self,
        notification_service: &NotificationService,
        user_id: Uuid,
        ip: IpAddr,
        device_name: &str,
        location: Option<&str>,
    ) -> Result<()> {
        use crate::services::notification_service::SystemNotificationInfo;
        use crate::websocket::protocol::NotificationType;

        let location_str = location.unwrap_or("未知位置");
        let content = format!(
            "检测到您的账号在 {} 的 {} 上登录（IP: {}）。如非本人操作，请立即修改密码并检查账号安全。",
            location_str, device_name, ip
        );

        // 创建系统通知信息
        let notification_info = SystemNotificationInfo {
            notification_type: NotificationType::Important,
            title: "异地登录提醒".to_string(),
            content,
            data: Some(serde_json::json!({
                "ip": ip.to_string(),
                "device": device_name,
                "location": location_str,
                "alert_type": "abnormal_login"
            })),
            created_at: Utc::now(),
        };

        // 发送系统通知给指定用户
        notification_service
            .send_system_notification(notification_info, Some(vec![user_id]))
            .await?;

        Ok(())
    }

    // ============================================================
    // 账号安全概览
    // ============================================================

    /// 获取账号安全概览
    pub async fn get_security_overview(
        &self,
        user_id: Uuid,
        notification_settings: &NotificationSettings,
    ) -> Result<AccountSecurityOverview> {
        // 获取活跃设备数
        let active_devices_count = self.count_active_devices(user_id).await?;

        // 获取最近登录记录（最近5条）
        let recent_logins = self.get_login_history(user_id, 5, 0).await?;

        // 检查是否有可疑活动（最近7天）
        let suspicious_logins = self.get_recent_suspicious_logins(user_id, 7).await?;
        let has_suspicious_activity = !suspicious_logins.is_empty();

        Ok(AccountSecurityOverview {
            active_devices_count,
            recent_logins,
            has_suspicious_activity,
            abnormal_login_alert: notification_settings.system_notification,
        })
    }

    /// 统计用户活跃设备数
    async fn count_active_devices(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM user_sessions
            WHERE user_id = $1 AND is_active = true AND expires_at > NOW()
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(count.0)
    }

    // ============================================================
    // 辅助方法
    // ============================================================

    /// 从 User-Agent 解析设备名称
    pub fn parse_device_name(user_agent: Option<&str>) -> String {
        match user_agent {
            None => "未知设备".to_string(),
            Some(ua) => {
                let ua_lower = ua.to_lowercase();
                if ua_lower.contains("iphone") {
                    "iPhone".to_string()
                } else if ua_lower.contains("ipad") {
                    "iPad".to_string()
                } else if ua_lower.contains("android") {
                    if ua_lower.contains("mobile") {
                        "Android 手机".to_string()
                    } else {
                        "Android 平板".to_string()
                    }
                } else if ua_lower.contains("windows") {
                    "Windows PC".to_string()
                } else if ua_lower.contains("macintosh") || ua_lower.contains("mac os") {
                    "Mac".to_string()
                } else if ua_lower.contains("linux") {
                    "Linux".to_string()
                } else {
                    "未知设备".to_string()
                }
            }
        }
    }

    /// 计算会话过期时间
    pub fn calculate_session_expiry(hours: i64) -> DateTime<Utc> {
        Utc::now() + Duration::hours(hours)
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false
            WHERE expires_at < NOW() AND is_active = true
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_device_name() {
        assert_eq!(
            AccountSecurityService::parse_device_name(Some(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X)"
            )),
            "iPhone"
        );
        assert_eq!(
            AccountSecurityService::parse_device_name(Some(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
            )),
            "Windows PC"
        );
        assert_eq!(
            AccountSecurityService::parse_device_name(Some(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
            )),
            "Mac"
        );
        assert_eq!(AccountSecurityService::parse_device_name(None), "未知设备");
    }

    #[test]
    fn test_calculate_session_expiry() {
        let expiry = AccountSecurityService::calculate_session_expiry(24);
        let now = Utc::now();
        let diff = expiry - now;
        assert!(diff.num_hours() >= 23 && diff.num_hours() <= 24);
    }
}
