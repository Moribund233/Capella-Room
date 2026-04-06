use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::db::Database;
use crate::error::{AppError, Result};
use crate::models::audit::{AlertRule, AuditAlert, AuditSeverity};
use crate::services::notification_service::NotificationService;
use crate::websocket::protocol::NotificationType;

/// 告警处理器
/// 负责告警的通知、聚合和抑制
pub struct AlertHandler {
    db: Database,
    notification_service: Arc<NotificationService>,
    /// 告警冷却记录（告警类型:IP -> 最后触发时间）
    cooldown_records: Arc<RwLock<HashMap<String, Instant>>>,
    /// 聚合窗口内的告警（用于聚合相似告警）
    aggregation_window: Arc<RwLock<HashMap<String, Vec<AuditAlert>>>>,
    /// 默认冷却时间（分钟）
    default_cooldown_minutes: i32,
}

impl AlertHandler {
    /// 创建新的告警处理器
    pub fn new(db: Database, notification_service: Arc<NotificationService>) -> Self {
        let handler = Self {
            db,
            notification_service,
            cooldown_records: Arc::new(RwLock::new(HashMap::new())),
            aggregation_window: Arc::new(RwLock::new(HashMap::new())),
            default_cooldown_minutes: 5,
        };

        handler.start_background_tasks();
        handler
    }

    /// 启动后台任务
    fn start_background_tasks(&self) {
        // 清理过期冷却记录
        let cooldown_records = Arc::clone(&self.cooldown_records);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                let mut records = cooldown_records.write().await;
                let now = Instant::now();
                records.retain(|_, time| now.duration_since(*time) < Duration::from_secs(3600));
                debug!("Cleaned up {} expired cooldown records", records.len());
            }
        });

        // 处理聚合窗口中的告警
        let aggregation_window = Arc::clone(&self.aggregation_window);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut window = aggregation_window.write().await;
                // 清空聚合窗口，触发聚合通知
                for (key, alerts) in window.drain() {
                    if alerts.len() > 1 {
                        info!(
                            "Aggregated {} alerts of type {} for notification",
                            alerts.len(),
                            key
                        );
                    }
                }
            }
        });
    }

    /// 处理新告警
    /// 包括冷却检查、聚合和通知
    pub async fn handle_alert(&self, alert: &AuditAlert) -> Result<()> {
        // 检查是否在冷却期内
        if self.is_in_cooldown(alert).await {
            debug!("Alert {} is in cooldown, skipping notification", alert.id);
            return Ok(());
        }

        // 检查是否需要聚合
        if self.should_aggregate(alert).await {
            self.add_to_aggregation(alert).await;
            return Ok(());
        }

        // 更新冷却记录
        self.update_cooldown(alert).await;

        // 发送通知
        self.send_alert_notification(alert).await?;

        Ok(())
    }

    /// 检查告警是否在冷却期内
    async fn is_in_cooldown(&self, alert: &AuditAlert) -> bool {
        let cooldown_key = self.get_cooldown_key(alert);
        let cooldown_minutes = self.get_cooldown_minutes(alert).await;

        let records = self.cooldown_records.read().await;
        if let Some(last_time) = records.get(&cooldown_key) {
            let elapsed = Instant::now().duration_since(*last_time);
            let cooldown_duration = Duration::from_secs(cooldown_minutes as u64 * 60);
            elapsed < cooldown_duration
        } else {
            false
        }
    }

    /// 更新冷却记录
    async fn update_cooldown(&self, alert: &AuditAlert) {
        let cooldown_key = self.get_cooldown_key(alert);
        let mut records = self.cooldown_records.write().await;
        records.insert(cooldown_key, Instant::now());
    }

    /// 获取冷却键
    fn get_cooldown_key(&self, alert: &AuditAlert) -> String {
        let base_key = format!(
            "{}:{}",
            alert.alert_type,
            alert.rule_id.unwrap_or(Uuid::nil())
        );

        if let Some(ip) = &alert.source_ip {
            format!("{}:{}", base_key, ip)
        } else if let Some(user_id) = alert.affected_user_id {
            format!("{}:user:{}", base_key, user_id)
        } else {
            base_key
        }
    }

    /// 获取告警的冷却时间（分钟）
    async fn get_cooldown_minutes(&self, alert: &AuditAlert) -> i32 {
        if let Some(rule_id) = alert.rule_id {
            let pool = self.db.pool();
            if let Ok(Some(rule)) =
                sqlx::query_as::<_, AlertRule>("SELECT * FROM audit_alert_rules WHERE id = $1")
                    .bind(rule_id)
                    .fetch_optional(pool)
                    .await
            {
                return rule.cooldown_minutes;
            }
        }
        self.default_cooldown_minutes
    }

    /// 检查告警是否需要聚合
    async fn should_aggregate(&self, alert: &AuditAlert) -> bool {
        // Critical 级别告警不聚合
        if alert.severity == AuditSeverity::Critical {
            return false;
        }

        let window = self.aggregation_window.read().await;
        let key = self.get_aggregation_key(alert);

        // 如果聚合窗口中已有相同类型的告警，则进行聚合
        if let Some(alerts) = window.get(&key) {
            return alerts.len() >= 3; // 3个及以上才聚合
        }

        false
    }

    /// 获取聚合键
    fn get_aggregation_key(&self, alert: &AuditAlert) -> String {
        format!(
            "{}:{:?}",
            alert.alert_type,
            alert.affected_user_id.unwrap_or(Uuid::nil())
        )
    }

    /// 添加告警到聚合窗口
    async fn add_to_aggregation(&self, alert: &AuditAlert) {
        let mut window = self.aggregation_window.write().await;
        let key = self.get_aggregation_key(alert);
        window.entry(key).or_default().push(alert.clone());
    }

    /// 发送告警通知
    async fn send_alert_notification(&self, alert: &AuditAlert) -> Result<()> {
        match alert.severity {
            AuditSeverity::Critical => {
                // Critical 级别：WebSocket 实时推送 + 系统通知
                self.notify_admins(alert).await?;
                self.send_websocket_alert(alert).await?;
            }
            AuditSeverity::Error => {
                // Error 级别：WebSocket 推送
                self.send_websocket_alert(alert).await?;
            }
            AuditSeverity::Warning => {
                // Warning 级别：仅系统通知
                self.notify_admins(alert).await?;
            }
            AuditSeverity::Info => {
                // Info 级别：仅记录，不通知
                debug!("Info level alert recorded: {}", alert.id);
            }
        }

        Ok(())
    }

    /// 通知所有管理员
    async fn notify_admins(&self, alert: &AuditAlert) -> Result<()> {
        let pool = self.db.pool();

        // 获取所有管理员用户
        let admin_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM users WHERE role IN ('admin', 'super_admin') AND disabled = false",
        )
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

    /// 通过 WebSocket 发送实时告警
    async fn send_websocket_alert(&self, alert: &AuditAlert) -> Result<()> {
        use crate::websocket::protocol::WebSocketMessage;

        let _ws_message = WebSocketMessage::SystemNotification {
            notification_type: NotificationType::Warning,
            title: format!("安全告警: {}", alert.title),
            content: alert.description.clone(),
            data: Some(json!({
                "alert_id": alert.id,
                "alert_type": alert.alert_type,
                "severity": format!("{:?}", alert.severity),
                "created_at": alert.created_at,
            })),
            created_at: Utc::now(),
        };

        // 使用通知服务发送系统通知给所有管理员
        use crate::services::notification_service::SystemNotificationInfo;

        let notification_info = SystemNotificationInfo {
            notification_type: NotificationType::Warning,
            title: format!("安全告警: {}", alert.title),
            content: alert.description.clone(),
            data: Some(json!({
                "alert_id": alert.id,
                "alert_type": alert.alert_type,
                "severity": format!("{:?}", alert.severity),
                "created_at": alert.created_at,
            })),
            created_at: Utc::now(),
        };

        // 获取所有管理员用户
        let pool = self.db.pool();
        let admin_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM users WHERE role IN ('admin', 'super_admin') AND disabled = false",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        // 发送系统通知
        if let Err(e) = self
            .notification_service
            .send_system_notification(notification_info, Some(admin_ids))
            .await
        {
            error!("Failed to send WebSocket alert notification: {}", e);
        }

        Ok(())
    }

    /// 处理聚合告警
    /// 当聚合窗口中的告警数量达到阈值时，发送聚合通知
    pub async fn process_aggregated_alerts(&self, alert_type: &str) -> Result<()> {
        let mut window = self.aggregation_window.write().await;

        if let Some(alerts) = window.remove(alert_type) {
            if alerts.is_empty() {
                return Ok(());
            }

            // 构建聚合通知
            let severity = alerts
                .iter()
                .map(|a| a.severity.priority())
                .max()
                .map(|p| match p {
                    3 => AuditSeverity::Critical,
                    2 => AuditSeverity::Error,
                    1 => AuditSeverity::Warning,
                    _ => AuditSeverity::Info,
                })
                .unwrap_or(AuditSeverity::Info);

            let aggregated_content = format!(
                "检测到 {} 个 '{}' 类型的安全告警，请查看审计日志了解详情。",
                alerts.len(),
                alert_type
            );

            info!(
                "Processing {} aggregated alerts of type {}",
                alerts.len(),
                alert_type
            );

            // 发送聚合通知
            use crate::services::notification_service::SystemNotificationInfo;

            let notification_info = SystemNotificationInfo {
                notification_type: NotificationType::Warning,
                title: format!("聚合安全告警: {}", alert_type),
                content: aggregated_content,
                data: Some(json!({
                    "alert_type": alert_type,
                    "count": alerts.len(),
                    "severity": format!("{:?}", severity),
                    "alert_ids": alerts.iter().map(|a| a.id).collect::<Vec<_>>(),
                })),
                created_at: Utc::now(),
            };

            let pool = self.db.pool();
            let admin_ids: Vec<Uuid> = sqlx::query_scalar(
                "SELECT id FROM users WHERE role IN ('admin', 'super_admin') AND disabled = false",
            )
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

            if let Err(e) = self
                .notification_service
                .send_system_notification(notification_info, Some(admin_ids))
                .await
            {
                error!("Failed to send aggregated alert notification: {}", e);
            }
        }

        Ok(())
    }
}
