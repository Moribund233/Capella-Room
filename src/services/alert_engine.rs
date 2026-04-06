use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Timelike;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;

use crate::db::Database;
use crate::error::{AppError, Result};
use crate::models::audit::{AlertRule, AuditAlert, AuditEventType, AuditLog, AuditSeverity};

/// 告警条件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    /// 阈值触发 - 达到指定次数触发
    Threshold,
    /// 频率触发 - 单位时间内达到指定次数触发
    Frequency,
    /// 模式匹配 - 匹配特定模式触发
    Pattern,
}

/// 告警条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    /// 条件类型
    pub condition_type: ConditionType,
    /// 阈值（用于 threshold 和 frequency 类型）
    pub threshold: Option<i32>,
    /// 时间窗口（分钟，用于 frequency 类型）
    pub time_window_minutes: Option<i32>,
    /// 匹配模式（用于 pattern 类型）
    pub pattern: Option<String>,
    /// 事件类型过滤
    pub event_types: Option<Vec<AuditEventType>>,
    /// 严重级别过滤
    pub severity: Option<AuditSeverity>,
}

/// 告警引擎
/// 负责分析审计日志并根据规则生成告警
pub struct AlertEngine {
    db: Database,
    /// 规则列表
    rules: Arc<RwLock<Vec<AlertRule>>>,
    /// 最近事件缓存（用于频率检测）
    recent_events: Arc<RwLock<VecDeque<AuditLog>>>,
    /// 事件缓存最大大小
    max_events_cache: usize,
    /// 告警冷却记录（告警类型 -> 最后触发时间）
    cooldown_records: Arc<RwLock<HashMap<String, Instant>>>,
    /// 最近告警记录（用于去重）
    recent_alerts: Arc<RwLock<VecDeque<(String, Instant)>>>,
}

impl AlertEngine {
    /// 创建新的告警引擎
    pub fn new(db: Database) -> Self {
        let engine = Self {
            db,
            rules: Arc::new(RwLock::new(Vec::new())),
            recent_events: Arc::new(RwLock::new(VecDeque::new())),
            max_events_cache: 10000,
            cooldown_records: Arc::new(RwLock::new(HashMap::new())),
            recent_alerts: Arc::new(RwLock::new(VecDeque::new())),
        };

        // 启动后台任务
        engine.start_background_tasks();
        engine
    }

    /// 启动后台任务
    fn start_background_tasks(&self) {
        let events_cache = Arc::clone(&self.recent_events);
        let max_cache = self.max_events_cache;

        // 定期清理过期事件缓存
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut events = events_cache.write().await;
                while events.len() > max_cache {
                    events.pop_front();
                }
            }
        });

        // 清理冷却记录
        let cooldown_records = Arc::clone(&self.cooldown_records);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                let mut records = cooldown_records.write().await;
                let now = Instant::now();
                records.retain(|_, time| now.duration_since(*time) < Duration::from_secs(3600));
            }
        });

        // 清理最近告警记录
        let recent_alerts = Arc::clone(&self.recent_alerts);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                let mut alerts = recent_alerts.write().await;
                let now = Instant::now();
                alerts.retain(|(_, time)| now.duration_since(*time) < Duration::from_secs(3600));
            }
        });
    }

    /// 加载告警规则
    pub async fn load_rules(&self) -> Result<()> {
        let pool = self.db.pool();
        let rules: Vec<AlertRule> = sqlx::query_as(
            "SELECT id, name, description, event_type, condition, severity, enabled, 
             cooldown_minutes, notify_admins, created_at, updated_at 
             FROM audit_alert_rules WHERE enabled = true ORDER BY created_at",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        let mut guard = self.rules.write().await;
        *guard = rules;
        info!("Loaded {} alert rules", guard.len());
        Ok(())
    }

    /// 分析单条审计日志
    pub async fn analyze_log(&self, log: &AuditLog) -> Result<Vec<AuditAlert>> {
        // 添加到事件缓存
        {
            let mut events = self.recent_events.write().await;
            events.push_back(log.clone());
            if events.len() > self.max_events_cache {
                events.pop_front();
            }
        }

        let rules = self.rules.read().await;
        let mut alerts = Vec::new();

        for rule in rules.iter().filter(|r| r.enabled) {
            if let Some(alert) = self.check_rule(log, rule).await? {
                // 检查是否在冷却期内
                if !self
                    .is_in_cooldown(
                        &alert.alert_type,
                        log.metadata
                            .as_ref()
                            .and_then(|m| m.get("ip_address"))
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse().ok()),
                    )
                    .await
                {
                    alerts.push(alert);
                }
            }
        }

        Ok(alerts)
    }

    /// 检查规则是否匹配
    async fn check_rule(&self, log: &AuditLog, rule: &AlertRule) -> Result<Option<AuditAlert>> {
        let condition: AlertCondition = serde_json::from_value(rule.condition.clone())
            .map_err(|e| AppError::Config(format!("Invalid alert condition: {}", e)))?;

        // 检查事件类型匹配
        if let Some(ref event_types) = condition.event_types {
            let contains = event_types.iter().any(|et: &AuditEventType| {
                std::mem::discriminant(et) == std::mem::discriminant(&log.event_type)
            });
            if !contains {
                return Ok(None);
            }
        } else if let Some(ref rule_event_type) = rule.event_type {
            if std::mem::discriminant(rule_event_type) != std::mem::discriminant(&log.event_type) {
                return Ok(None);
            }
        }

        // 检查严重级别匹配
        if let Some(ref severity) = condition.severity {
            if std::mem::discriminant(severity) != std::mem::discriminant(&log.severity) {
                return Ok(None);
            }
        }

        // 根据条件类型进行检测
        let matched = match condition.condition_type {
            ConditionType::Threshold => self.check_threshold(log, &condition).await?,
            ConditionType::Frequency => self.check_frequency(log, &condition).await?,
            ConditionType::Pattern => self.check_pattern(log, &condition),
        };

        if matched {
            let alert = self.create_alert_from_rule(rule, log).await?;
            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    /// 阈值检测
    async fn check_threshold(&self, log: &AuditLog, condition: &AlertCondition) -> Result<bool> {
        let threshold = condition.threshold.unwrap_or(1);
        let time_window_minutes = condition.time_window_minutes.unwrap_or(60) as i64;
        let since = Utc::now() - chrono::Duration::minutes(time_window_minutes);

        let pool = self.db.pool();

        // 查询数据库中相同类型的最近事件数量
        let count: i64 = if let Some(actor_id) = log.actor_id {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM audit_logs 
                 WHERE event_type = $1 AND actor_id = $2 AND created_at >= $3",
            )
            .bind(&log.event_type)
            .bind(actor_id)
            .bind(since)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM audit_logs 
                 WHERE event_type = $1 AND created_at >= $2",
            )
            .bind(&log.event_type)
            .bind(since)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?
        };

        Ok(count >= threshold as i64)
    }

    /// 频率检测
    async fn check_frequency(&self, log: &AuditLog, condition: &AlertCondition) -> Result<bool> {
        let threshold = condition.threshold.unwrap_or(5);
        let time_window =
            Duration::from_secs((condition.time_window_minutes.unwrap_or(5) * 60) as u64);

        let events = self.recent_events.read().await;
        let now = Utc::now();

        let count = events
            .iter()
            .filter(|e| {
                e.event_type == log.event_type
                    && e.created_at
                        > now
                            - chrono::Duration::from_std(time_window)
                                .unwrap_or(chrono::Duration::minutes(5))
                    && (log.actor_id.is_none() || e.actor_id == log.actor_id)
            })
            .count();

        Ok(count >= threshold as usize)
    }

    /// 模式匹配检测
    fn check_pattern(&self, log: &AuditLog, condition: &AlertCondition) -> bool {
        if let Some(pattern) = &condition.pattern {
            match pattern.as_str() {
                "suspicious_ip" => {
                    // 检查是否为可疑IP（这里可以实现更复杂的逻辑）
                    false
                }
                "abnormal_time" => {
                    // 检查是否在异常时间操作（如凌晨）
                    let hour = log.created_at.hour();
                    !(6..=23).contains(&hour)
                }
                "privilege_escalation" => {
                    // 检查是否为权限提升操作
                    matches!(log.event_type, AuditEventType::AdminUserRoleChange)
                }
                _ => {
                    // 默认匹配描述
                    log.description.contains(pattern)
                }
            }
        } else {
            true
        }
    }

    /// 从规则创建告警
    async fn create_alert_from_rule(&self, rule: &AlertRule, log: &AuditLog) -> Result<AuditAlert> {
        let source_ip = log
            .metadata
            .as_ref()
            .and_then(|m| m.get("ip_address"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let pool = self.db.pool();

        let alert: AuditAlert = sqlx::query_as(
            "INSERT INTO audit_alerts 
             (rule_id, alert_type, severity, title, description, related_logs, source_ip, affected_user_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, rule_id, alert_type, severity, title, description, related_logs, 
                       source_ip, affected_user_id, status, acknowledged_by, acknowledged_at, 
                       resolved_by, resolved_at, created_at, updated_at"
        )
        .bind(rule.id)
        .bind(format!("{:?}", log.event_type))
        .bind(&rule.severity)
        .bind(&rule.name)
        .bind(format!("{} - {}", rule.description.clone().unwrap_or_default(), log.description))
        .bind(vec![log.id])
        .bind(source_ip)
        .bind(log.actor_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        info!("Created security alert: {} - {}", alert.id, alert.title);

        Ok(alert)
    }

    /// 检查是否在冷却期内
    async fn is_in_cooldown(&self, alert_type: &str, source_ip: Option<IpAddr>) -> bool {
        let cooldown_key = if let Some(ip) = source_ip {
            format!("{}:{}", alert_type, ip)
        } else {
            alert_type.to_string()
        };

        let records = self.cooldown_records.read().await;
        if let Some(last_time) = records.get(&cooldown_key) {
            let elapsed = Instant::now().duration_since(*last_time);
            // 默认冷却时间5分钟
            elapsed < Duration::from_secs(300)
        } else {
            false
        }
    }

    /// 更新冷却记录
    pub async fn update_cooldown(&self, alert_type: &str, source_ip: Option<IpAddr>) {
        let cooldown_key = if let Some(ip) = source_ip {
            format!("{}:{}", alert_type, ip)
        } else {
            alert_type.to_string()
        };

        let mut records = self.cooldown_records.write().await;
        records.insert(cooldown_key, Instant::now());
    }

    /// 检查告警是否需要聚合
    pub async fn should_aggregate(&self, alert: &AuditAlert) -> bool {
        let alerts = self.recent_alerts.read().await;
        let alert_key = format!("{}:{:?}", alert.alert_type, alert.affected_user_id);

        alerts.iter().any(|(key, time)| {
            key == &alert_key && Instant::now().duration_since(*time) < Duration::from_secs(300)
        })
    }

    /// 记录告警（用于聚合检测）
    pub async fn record_alert(&self, alert: &AuditAlert) {
        let mut alerts = self.recent_alerts.write().await;
        let alert_key = format!("{}:{:?}", alert.alert_type, alert.affected_user_id);
        alerts.push_back((alert_key, Instant::now()));

        // 限制缓存大小
        while alerts.len() > 1000 {
            alerts.pop_front();
        }
    }
}

/// 创建暴力破解检测规则
pub fn create_brute_force_rule() -> AlertCondition {
    AlertCondition {
        condition_type: ConditionType::Frequency,
        threshold: Some(5),
        time_window_minutes: Some(5),
        pattern: None,
        event_types: Some(vec![AuditEventType::SystemLoginFailure]),
        severity: Some(AuditSeverity::Warning),
    }
}

/// 创建异常登录检测规则
pub fn create_abnormal_login_rule() -> AlertCondition {
    AlertCondition {
        condition_type: ConditionType::Pattern,
        threshold: None,
        time_window_minutes: None,
        pattern: Some("abnormal_time".to_string()),
        event_types: Some(vec![AuditEventType::UserLogin]),
        severity: Some(AuditSeverity::Info),
    }
}

/// 创建越权访问检测规则
pub fn create_unauthorized_access_rule() -> AlertCondition {
    AlertCondition {
        condition_type: ConditionType::Threshold,
        threshold: Some(1),
        time_window_minutes: Some(1),
        pattern: None,
        event_types: Some(vec![AuditEventType::SystemUnauthorizedAccess]),
        severity: Some(AuditSeverity::Error),
    }
}

/// 创建敏感操作监控规则
pub fn create_sensitive_operation_rule() -> AlertCondition {
    AlertCondition {
        condition_type: ConditionType::Threshold,
        threshold: Some(1),
        time_window_minutes: Some(1),
        pattern: None,
        event_types: Some(vec![
            AuditEventType::AdminUserDelete,
            AuditEventType::AdminRoomDelete,
            AuditEventType::AdminMessageDelete,
        ]),
        severity: Some(AuditSeverity::Warning),
    }
}
