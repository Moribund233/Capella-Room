use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================
// 数据库模型
// ============================================================

/// 用户登录会话（设备管理）
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    /// SHA-256 哈希值，用于验证会话令牌
    pub session_token_hash: String,
    /// 设备名称（如 "iPhone 15", "Windows PC"）
    pub device_name: Option<String>,
    /// 设备类型：mobile, tablet, desktop, unknown
    pub device_type: Option<String>,
    /// IP 地址（使用 INET 类型存储）
    pub ip_address: Option<IpNetwork>,
    /// User-Agent 字符串
    pub user_agent: Option<String>,
    /// 位置信息（国家、城市等）
    pub location_info: Option<serde_json::Value>,
    /// 是否为当前会话
    pub is_current: bool,
    /// 是否活跃
    pub is_active: bool,
    /// 是否被用户禁用（禁用后该设备无法使用旧 Token 登录）
    pub is_blocked: bool,
    /// 最后活跃时间
    pub last_active_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 登录历史记录
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct LoginHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    /// IP 地址（使用 INET 类型存储）
    pub ip_address: IpNetwork,
    /// 设备信息（名称、类型、User-Agent等）
    pub device_info: Option<serde_json::Value>,
    /// 位置信息
    pub location_info: Option<serde_json::Value>,
    /// 登录状态：success, failed, blocked
    pub login_status: String,
    /// 失败原因（如 invalid_password, account_locked）
    pub failure_reason: Option<String>,
    /// 是否可疑登录
    pub is_suspicious: bool,
    /// 风险等级：low, medium, high
    pub risk_level: String,
    /// 登录时间
    pub created_at: DateTime<Utc>,
}

// ============================================================
// 响应结构体
// ============================================================

/// 设备信息响应
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub id: Uuid,
    pub device_name: String,
    pub device_type: String,
    pub ip_address: String,
    pub location: Option<String>,
    pub is_current: bool,
    pub is_active: bool,
    /// 是否被用户禁用
    pub is_blocked: bool,
    pub last_active_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<UserSession> for DeviceInfo {
    fn from(session: UserSession) -> Self {
        let location = session.location_info.as_ref().and_then(|info| {
            info.get("city")
                .and_then(|c| c.as_str())
                .zip(info.get("country").and_then(|c| c.as_str()))
                .map(|(city, country)| format!("{}, {}", city, country))
        });

        Self {
            id: session.id,
            device_name: session
                .device_name
                .unwrap_or_else(|| "未知设备".to_string()),
            device_type: session.device_type.unwrap_or_else(|| "unknown".to_string()),
            ip_address: session
                .ip_address
                .map(|ip| ip.to_string())
                .unwrap_or_default(),
            location,
            is_current: session.is_current,
            is_active: session.is_active,
            is_blocked: session.is_blocked,
            last_active_at: session.last_active_at,
            created_at: session.created_at,
        }
    }
}

impl UserSession {
    /// 获取 IP 地址作为标准库类型
    pub fn ip(&self) -> Option<std::net::IpAddr> {
        self.ip_address.map(|ip| ip.ip())
    }
}

/// 登录历史记录响应
#[derive(Debug, Clone, Serialize)]
pub struct LoginHistoryEntry {
    pub id: Uuid,
    pub ip_address: String,
    pub device_name: String,
    pub device_type: String,
    pub location: Option<String>,
    pub login_status: String,
    pub failure_reason: Option<String>,
    pub is_suspicious: bool,
    pub risk_level: String,
    pub created_at: DateTime<Utc>,
}

impl From<LoginHistory> for LoginHistoryEntry {
    fn from(history: LoginHistory) -> Self {
        let device_info = history.device_info.as_ref();
        let device_name = device_info
            .and_then(|info| info.get("device_name").and_then(|d| d.as_str()))
            .unwrap_or("未知设备")
            .to_string();
        let device_type = device_info
            .and_then(|info| info.get("device_type").and_then(|d| d.as_str()))
            .unwrap_or("unknown")
            .to_string();

        let location = history.location_info.as_ref().and_then(|info| {
            info.get("city")
                .and_then(|c| c.as_str())
                .zip(info.get("country").and_then(|c| c.as_str()))
                .map(|(city, country)| format!("{}, {}", city, country))
        });

        Self {
            id: history.id,
            ip_address: history.ip_address.to_string(),
            device_name,
            device_type,
            location,
            login_status: history.login_status,
            failure_reason: history.failure_reason,
            is_suspicious: history.is_suspicious,
            risk_level: history.risk_level,
            created_at: history.created_at,
        }
    }
}

impl LoginHistory {
    /// 获取 IP 地址作为标准库类型
    pub fn ip(&self) -> std::net::IpAddr {
        self.ip_address.ip()
    }
}

/// 账号安全概览响应
#[derive(Debug, Clone, Serialize)]
pub struct AccountSecurityOverview {
    /// 当前活跃设备数
    pub active_devices_count: i64,
    /// 最近登录记录
    pub recent_logins: Vec<LoginHistoryEntry>,
    /// 是否有可疑活动
    pub has_suspicious_activity: bool,
    /// 异地登录提醒是否启用
    pub abnormal_login_alert: bool,
}

// ============================================================
// 请求结构体
// ============================================================

/// 创建设备会话请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSessionRequest {
    pub session_token_hash: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: std::net::IpAddr,
    pub user_agent: Option<String>,
    pub location_info: Option<serde_json::Value>,
    pub expires_at: DateTime<Utc>,
}

/// 记录登录历史请求
#[derive(Debug, Clone, Deserialize)]
pub struct RecordLoginRequest {
    pub ip_address: std::net::IpAddr,
    pub device_info: Option<serde_json::Value>,
    pub location_info: Option<serde_json::Value>,
    pub login_status: LoginStatus,
    pub failure_reason: Option<String>,
    pub is_suspicious: bool,
    pub risk_level: RiskLevel,
}

/// 更新异地登录提醒设置请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAbnormalLoginAlertRequest {
    pub enabled: bool,
}

// ============================================================
// 枚举类型
// ============================================================

/// 登录状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoginStatus {
    Success,
    Failed,
    Blocked,
}

impl LoginStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoginStatus::Success => "success",
            LoginStatus::Failed => "failed",
            LoginStatus::Blocked => "blocked",
        }
    }
}

impl From<String> for LoginStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "success" => LoginStatus::Success,
            "failed" => LoginStatus::Failed,
            "blocked" => LoginStatus::Blocked,
            _ => LoginStatus::Failed,
        }
    }
}

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
        }
    }
}

impl From<String> for RiskLevel {
    fn from(s: String) -> Self {
        match s.as_str() {
            "low" => RiskLevel::Low,
            "medium" => RiskLevel::Medium,
            "high" => RiskLevel::High,
            _ => RiskLevel::Low,
        }
    }
}

/// 设备类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Mobile,
    Tablet,
    Desktop,
    Unknown,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::Mobile => "mobile",
            DeviceType::Tablet => "tablet",
            DeviceType::Desktop => "desktop",
            DeviceType::Unknown => "unknown",
        }
    }

    /// 从 User-Agent 解析设备类型
    pub fn from_user_agent(user_agent: &str) -> Self {
        let ua = user_agent.to_lowercase();
        if ua.contains("mobile") || ua.contains("iphone") || ua.contains("android") {
            DeviceType::Mobile
        } else if ua.contains("tablet") || ua.contains("ipad") {
            DeviceType::Tablet
        } else if ua.contains("windows") || ua.contains("macintosh") || ua.contains("linux") {
            DeviceType::Desktop
        } else {
            DeviceType::Unknown
        }
    }
}

// ============================================================
// 辅助结构体
// ============================================================

/// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub country: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub timezone: Option<String>,
}

impl LocationInfo {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "country": self.country,
            "city": self.city,
            "region": self.region,
            "timezone": self.timezone,
        })
    }
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDetail {
    pub device_name: String,
    pub device_type: String,
    pub user_agent: String,
}

impl DeviceDetail {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "device_name": self.device_name,
            "device_type": self.device_type,
            "user_agent": self.user_agent,
        })
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_from_user_agent() {
        assert_eq!(
            DeviceType::from_user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X)"),
            DeviceType::Mobile
        );
        assert_eq!(
            DeviceType::from_user_agent("Mozilla/5.0 (iPad; CPU OS 16_0 like Mac OS X)"),
            DeviceType::Tablet
        );
        assert_eq!(
            DeviceType::from_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
            DeviceType::Desktop
        );
        assert_eq!(DeviceType::from_user_agent("Unknown"), DeviceType::Unknown);
    }

    #[test]
    fn test_login_status_as_str() {
        assert_eq!(LoginStatus::Success.as_str(), "success");
        assert_eq!(LoginStatus::Failed.as_str(), "failed");
        assert_eq!(LoginStatus::Blocked.as_str(), "blocked");
    }

    #[test]
    fn test_risk_level_as_str() {
        assert_eq!(RiskLevel::Low.as_str(), "low");
        assert_eq!(RiskLevel::Medium.as_str(), "medium");
        assert_eq!(RiskLevel::High.as_str(), "high");
    }

    #[test]
    fn test_location_info_to_json() {
        let location = LocationInfo {
            country: Some("中国".to_string()),
            city: Some("北京".to_string()),
            region: Some("北京市".to_string()),
            timezone: Some("Asia/Shanghai".to_string()),
        };
        let json = location.to_json_value();
        assert_eq!(json["country"], "中国");
        assert_eq!(json["city"], "北京");
    }
}
