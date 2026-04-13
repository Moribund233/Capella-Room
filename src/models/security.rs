use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use std::net::IpAddr;
use uuid::Uuid;

/// IP 列表类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "ip_list_type", rename_all = "snake_case")]
pub enum IpListType {
    Whitelist,
    Blacklist,
}

impl fmt::Display for IpListType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IpListType::Whitelist => "whitelist",
            IpListType::Blacklist => "blacklist",
        };
        write!(f, "{}", s)
    }
}

/// IP 检查事件（用于审计）
#[derive(Debug, Clone)]
pub enum IpCheckResult {
    Allowed,
    BlockedByBlacklist { reason: String },
    NotInWhitelist,
    RateLimited,
}

impl IpCheckResult {
    /// 检查是否允许连接
    pub fn is_allowed(&self) -> bool {
        matches!(self, IpCheckResult::Allowed)
    }

    /// 获取拒绝原因
    pub fn rejection_reason(&self) -> Option<String> {
        match self {
            IpCheckResult::Allowed => None,
            IpCheckResult::BlockedByBlacklist { reason } => Some(reason.clone()),
            IpCheckResult::NotInWhitelist => Some("IP address not in whitelist".to_string()),
            IpCheckResult::RateLimited => Some("Rate limit exceeded".to_string()),
        }
    }
}

/// IP 列表项数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct IpListEntry {
    pub id: Uuid,
    pub ip_address: String,
    pub ip_range_cidr: Option<String>,
    pub list_type: IpListType,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IpListEntry {
    /// 获取 IP 地址
    pub fn ip(&self) -> Option<IpAddr> {
        self.ip_address.parse().ok()
    }
}

/// 创建 IP 列表项请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateIpListRequest {
    pub ip_address: IpAddr,
    pub ip_range_cidr: Option<String>,
    pub list_type: IpListType,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CreateIpListRequest {
    /// 创建新的请求
    pub fn new(ip_address: IpAddr, list_type: IpListType) -> Self {
        Self {
            ip_address,
            ip_range_cidr: None,
            list_type,
            description: None,
            expires_at: None,
        }
    }

    /// 设置 CIDR 范围
    pub fn with_cidr(mut self, cidr: impl Into<String>) -> Self {
        self.ip_range_cidr = Some(cidr.into());
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 设置过期时间
    pub fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

/// 更新 IP 列表项请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIpListRequest {
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// IP 列表查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct IpListQuery {
    pub list_type: Option<IpListType>,
    pub ip_address: Option<IpAddr>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for IpListQuery {
    fn default() -> Self {
        Self {
            list_type: None,
            ip_address: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

/// IP 检查请求（用于管理 API）
#[derive(Debug, Clone, Deserialize)]
pub struct IpCheckRequest {
    pub ip_address: IpAddr,
}

/// IP 检查结果响应
#[derive(Debug, Clone, Serialize)]
pub struct IpCheckResponse {
    pub ip_address: IpAddr,
    pub allowed: bool,
    pub reason: Option<String>,
    pub list_type: Option<IpListType>,
}

/// IP 列表项响应（包含创建者信息）
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct IpListEntryResponse {
    pub id: Uuid,
    pub ip_address: String,
    pub ip_range_cidr: Option<String>,
    pub list_type: IpListType,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_by_username: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IpListEntryResponse {
    /// 获取 IP 地址
    pub fn ip(&self) -> Option<IpAddr> {
        self.ip_address.parse().ok()
    }
}

/// IP 安全统计信息
#[derive(Debug, Clone, Serialize)]
pub struct IpSecurityStats {
    pub total_whitelist: i64,
    pub total_blacklist: i64,
    pub expired_entries: i64,
    pub active_entries: i64,
}

/// IP 列表批量操作请求
#[derive(Debug, Clone, Deserialize)]
pub struct BatchIpListRequest {
    pub entries: Vec<CreateIpListRequest>,
}

/// IP 列表批量操作响应
#[derive(Debug, Clone, Serialize)]
pub struct BatchIpListResponse {
    pub success_count: usize,
    pub failed_count: usize,
    pub failed_ips: Vec<String>,
}
