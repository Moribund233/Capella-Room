use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use std::fmt;
use std::net::IpAddr;
use uuid::Uuid;

use crate::models::user::UserRole;

/// 审计事件类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "audit_event_type", rename_all = "snake_case")]
pub enum AuditEventType {
    // 用户事件
    UserLogin,
    UserLogout,
    UserRegister,
    UserPasswordChange,
    UserProfileUpdate,
    // 房间事件
    RoomCreate,
    RoomDelete,
    RoomMemberAdd,
    RoomMemberRemove,
    RoomMemberRoleChange,
    // 消息事件
    MessageSend,
    MessageEdit,
    MessageDelete,
    MessageReport,
    // 管理员事件
    AdminUserDisable,
    AdminUserRoleChange,
    AdminUserDelete,
    AdminRoomDelete,
    AdminMessageDelete,
    AdminConfigUpdate,
    // 系统事件
    SystemLoginFailure,
    SystemUnauthorizedAccess,
    SystemRateLimitTriggered,
    // 审计系统事件
    AuditQuery,
    AuditExport,
    AuditStatsQuery,
    AlertQuery,
    AlertRuleUpdate,
    AuditCleanup,
}

impl fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AuditEventType::UserLogin => "user_login",
            AuditEventType::UserLogout => "user_logout",
            AuditEventType::UserRegister => "user_register",
            AuditEventType::UserPasswordChange => "user_password_change",
            AuditEventType::UserProfileUpdate => "user_profile_update",
            AuditEventType::RoomCreate => "room_create",
            AuditEventType::RoomDelete => "room_delete",
            AuditEventType::RoomMemberAdd => "room_member_add",
            AuditEventType::RoomMemberRemove => "room_member_remove",
            AuditEventType::RoomMemberRoleChange => "room_member_role_change",
            AuditEventType::MessageSend => "message_send",
            AuditEventType::MessageEdit => "message_edit",
            AuditEventType::MessageDelete => "message_delete",
            AuditEventType::MessageReport => "message_report",
            AuditEventType::AdminUserDisable => "admin_user_disable",
            AuditEventType::AdminUserRoleChange => "admin_user_role_change",
            AuditEventType::AdminUserDelete => "admin_user_delete",
            AuditEventType::AdminRoomDelete => "admin_room_delete",
            AuditEventType::AdminMessageDelete => "admin_message_delete",
            AuditEventType::AdminConfigUpdate => "admin_config_update",
            AuditEventType::SystemLoginFailure => "system_login_failure",
            AuditEventType::SystemUnauthorizedAccess => "system_unauthorized_access",
            AuditEventType::SystemRateLimitTriggered => "system_rate_limit_triggered",
            AuditEventType::AuditQuery => "audit_query",
            AuditEventType::AuditExport => "audit_export",
            AuditEventType::AuditStatsQuery => "audit_stats_query",
            AuditEventType::AlertQuery => "alert_query",
            AuditEventType::AlertRuleUpdate => "alert_rule_update",
            AuditEventType::AuditCleanup => "audit_cleanup",
        };
        write!(f, "{}", s)
    }
}

impl AuditEventType {
    /// 获取事件类型的分类
    pub fn category(&self) -> &'static str {
        match self {
            AuditEventType::UserLogin
            | AuditEventType::UserLogout
            | AuditEventType::UserRegister
            | AuditEventType::UserPasswordChange
            | AuditEventType::UserProfileUpdate => "user",
            AuditEventType::RoomCreate
            | AuditEventType::RoomDelete
            | AuditEventType::RoomMemberAdd
            | AuditEventType::RoomMemberRemove
            | AuditEventType::RoomMemberRoleChange => "room",
            AuditEventType::MessageSend
            | AuditEventType::MessageEdit
            | AuditEventType::MessageDelete
            | AuditEventType::MessageReport => "message",
            AuditEventType::AdminUserDisable
            | AuditEventType::AdminUserRoleChange
            | AuditEventType::AdminUserDelete
            | AuditEventType::AdminRoomDelete
            | AuditEventType::AdminMessageDelete
            | AuditEventType::AdminConfigUpdate => "admin",
            AuditEventType::SystemLoginFailure
            | AuditEventType::SystemUnauthorizedAccess
            | AuditEventType::SystemRateLimitTriggered => "system",
            AuditEventType::AuditQuery
            | AuditEventType::AuditExport
            | AuditEventType::AuditStatsQuery
            | AuditEventType::AlertQuery
            | AuditEventType::AlertRuleUpdate
            | AuditEventType::AuditCleanup => "audit",
        }
    }

    /// 获取默认的严重级别
    pub fn default_severity(&self) -> AuditSeverity {
        match self {
            AuditEventType::SystemLoginFailure => AuditSeverity::Warning,
            AuditEventType::SystemUnauthorizedAccess => AuditSeverity::Error,
            AuditEventType::SystemRateLimitTriggered => AuditSeverity::Warning,
            AuditEventType::AdminUserDelete
            | AuditEventType::AdminRoomDelete
            | AuditEventType::AdminMessageDelete => AuditSeverity::Warning,
            _ => AuditSeverity::Info,
        }
    }
}

/// 审计严重级别
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "audit_severity", rename_all = "lowercase")]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        };
        write!(f, "{}", s)
    }
}

impl AuditSeverity {
    /// 获取严重级别的优先级数值
    pub fn priority(&self) -> i32 {
        match self {
            AuditSeverity::Info => 0,
            AuditSeverity::Warning => 1,
            AuditSeverity::Error => 2,
            AuditSeverity::Critical => 3,
        }
    }
}

/// 告警状态
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "alert_status", rename_all = "lowercase")]
pub enum AlertStatus {
    New,
    Acknowledged,
    Resolved,
    Ignored,
}

/// 审计日志数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub actor_role: Option<UserRole>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub action: String,
    pub description: String,
    pub metadata: Option<JsonValue>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 审计告警数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuditAlert {
    pub id: Uuid,
    pub rule_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: AuditSeverity,
    pub title: String,
    pub description: String,
    pub related_logs: Option<Vec<Uuid>>,
    pub source_ip: Option<String>,
    pub affected_user_id: Option<Uuid>,
    pub status: AlertStatus,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 告警规则数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub event_type: Option<AuditEventType>,
    pub condition: JsonValue,
    pub severity: AuditSeverity,
    pub enabled: bool,
    pub cooldown_minutes: i32,
    pub notify_admins: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建审计日志请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAuditLogRequest {
    pub event_type: AuditEventType,
    pub severity: Option<AuditSeverity>,
    pub actor_id: Option<Uuid>,
    pub actor_role: Option<UserRole>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub action: String,
    pub description: String,
    pub metadata: Option<AuditMetadata>,
    pub status: Option<String>,
    pub error_message: Option<String>,
}

impl CreateAuditLogRequest {
    /// 创建新的审计日志请求
    pub fn new(
        event_type: AuditEventType,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            event_type,
            severity: None,
            actor_id: None,
            actor_role: None,
            target_type: None,
            target_id: None,
            action: action.into(),
            description: description.into(),
            metadata: None,
            status: Some("success".to_string()),
            error_message: None,
        }
    }

    /// 设置操作者信息
    pub fn with_actor(mut self, user_id: Uuid, role: UserRole) -> Self {
        self.actor_id = Some(user_id);
        self.actor_role = Some(role);
        self
    }

    /// 设置目标信息
    pub fn with_target(mut self, target_type: impl Into<String>, target_id: Uuid) -> Self {
        self.target_type = Some(target_type.into());
        self.target_id = Some(target_id);
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: AuditMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// 设置严重级别
    pub fn with_severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// 设置为失败状态
    pub fn with_error(mut self, error_message: impl Into<String>) -> Self {
        self.status = Some("failure".to_string());
        self.error_message = Some(error_message.into());
        self.severity = Some(AuditSeverity::Error);
        self
    }

    /// 获取严重级别（使用默认值或指定值）
    pub fn severity(&self) -> AuditSeverity {
        self.severity
            .clone()
            .unwrap_or_else(|| self.event_type.default_severity())
    }
}

/// 审计元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub request_path: Option<String>,
    pub request_method: Option<String>,
    pub request_params: Option<JsonValue>,
    pub extra: Option<JsonValue>,
}

impl AuditMetadata {
    /// 创建新的元数据
    pub fn new() -> Self {
        Self {
            ip_address: None,
            user_agent: None,
            request_path: None,
            request_method: None,
            request_params: None,
            extra: None,
        }
    }

    /// 设置IP地址
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// 设置User-Agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// 设置请求信息
    pub fn with_request(
        mut self,
        method: impl Into<String>,
        path: impl Into<String>,
        params: Option<JsonValue>,
    ) -> Self {
        self.request_method = Some(method.into());
        self.request_path = Some(path.into());
        self.request_params = params;
        self
    }

    /// 设置额外数据
    pub fn with_extra(mut self, extra: JsonValue) -> Self {
        self.extra = Some(extra);
        self
    }
}

impl Default for AuditMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// 审计日志查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct AuditLogQuery {
    pub event_type: Option<AuditEventType>,
    pub severity: Option<AuditSeverity>,
    pub actor_id: Option<Uuid>,
    pub target_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for AuditLogQuery {
    fn default() -> Self {
        Self {
            event_type: None,
            severity: None,
            actor_id: None,
            target_id: None,
            target_type: None,
            status: None,
            start_time: None,
            end_time: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

/// 审计日志列表响应
#[derive(Debug, Clone, Serialize)]
pub struct AuditLogListResponse {
    pub logs: Vec<AuditLog>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// 告警查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct AlertQuery {
    pub status: Option<AlertStatus>,
    pub severity: Option<AuditSeverity>,
    pub alert_type: Option<String>,
    pub affected_user_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for AlertQuery {
    fn default() -> Self {
        Self {
            status: None,
            severity: None,
            alert_type: None,
            affected_user_id: None,
            start_time: None,
            end_time: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

/// 告警列表响应
#[derive(Debug, Clone, Serialize)]
pub struct AlertListResponse {
    pub alerts: Vec<AuditAlert>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// 更新告警状态请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAlertStatusRequest {
    pub status: AlertStatus,
}

/// 更新告警规则请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAlertRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub condition: Option<JsonValue>,
    pub severity: Option<AuditSeverity>,
    pub enabled: Option<bool>,
    pub cooldown_minutes: Option<i32>,
    pub notify_admins: Option<bool>,
}

/// 创建告警请求
#[derive(Debug, Clone)]
pub struct CreateAlertRequest {
    pub rule_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: AuditSeverity,
    pub title: String,
    pub description: String,
    pub related_logs: Vec<Uuid>,
    pub source_ip: Option<String>,
    pub affected_user_id: Option<Uuid>,
}

/// 审计统计信息
#[derive(Debug, Clone, Serialize)]
pub struct AuditStats {
    pub total_logs: i64,
    pub today_logs: i64,
    pub week_logs: i64,
    pub month_logs: i64,
    pub logs_by_severity: Vec<SeverityCount>,
    pub logs_by_event_type: Vec<EventTypeCount>,
    pub logs_by_day: Vec<DailyCount>,
    pub alerts_count: i64,
    pub new_alerts_count: i64,
}

/// 严重级别统计
#[derive(Debug, Clone, Serialize)]
pub struct SeverityCount {
    pub severity: AuditSeverity,
    pub count: i64,
}

/// 事件类型统计
#[derive(Debug, Clone, Serialize)]
pub struct EventTypeCount {
    pub event_type: AuditEventType,
    pub count: i64,
}

/// 每日统计
#[derive(Debug, Clone, Serialize)]
pub struct DailyCount {
    pub date: String,
    pub count: i64,
}

/// 审计统计查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct AuditStatsQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// 审计日志导出查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct AuditLogExportQuery {
    pub event_type: Option<AuditEventType>,
    pub severity: Option<AuditSeverity>,
    pub actor_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub format: ExportFormat,
}

/// 导出格式
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
}

/// 告警规则响应
#[derive(Debug, Clone, Serialize)]
pub struct AlertRuleResponse {
    pub rules: Vec<AlertRule>,
}

/// 单个告警规则详情响应
#[derive(Debug, Clone, Serialize)]
pub struct AlertRuleDetailResponse {
    pub rule: AlertRule,
}
