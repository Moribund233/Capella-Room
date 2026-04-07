use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod listener;
pub mod loader;
pub mod manager;

pub use listener::{start_config_listeners, LoggingConfigListener, WebSocketConfigListener};
pub use loader::ConfigLoader;
pub use manager::{ConfigChangeEvent, ConfigManager};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppConfigSection,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub jwt: JwtConfig,
    #[serde(default)]
    pub upload: UploadConfig,
    #[serde(default)]
    pub websocket: WebSocketConfig,
    #[serde(default)]
    pub reconnect: ReconnectConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub system: SystemConfig,
    #[serde(default)]
    pub admin: AdminConfig,
    #[serde(default)]
    pub audit: AuditConfig,
    #[serde(default)]
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfigSection {
    #[serde(default)]
    pub env: String,
    pub config_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub max_connections: u32,
    #[serde(default)]
    pub acquire_timeout_secs: u64,
    #[serde(default)]
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct JwtConfig {
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub expiration_hours: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UploadConfig {
    #[serde(default)]
    pub max_file_size: usize,
    #[serde(default)]
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WebSocketConfig {
    #[serde(default)]
    pub heartbeat_interval_secs: u64,
    #[serde(default)]
    pub heartbeat_timeout_secs: u64,
    #[serde(default)]
    pub auth_timeout_secs: u64,
    #[serde(default)]
    pub message_buffer_size: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ReconnectConfig {
    #[serde(default)]
    pub base_delay_ms: u64,
    #[serde(default)]
    pub max_delay_ms: u64,
    #[serde(default)]
    pub max_attempts: u32,
    #[serde(default)]
    pub multiplier: u32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub structured: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SystemConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub maintenance_mode: bool,
    #[serde(default)]
    pub maintenance_message: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AdminConfig {
    #[serde(default)]
    pub initial: InitialAdminConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub log_retention_days: i32,
    #[serde(default)]
    pub buffer_size: usize,
    #[serde(default)]
    pub flush_interval_seconds: u64,
    #[serde(default)]
    pub excluded_paths: Vec<String>,
    #[serde(default)]
    pub alert_enabled: bool,
    #[serde(default)]
    pub alert_cooldown_minutes: i32,
    #[serde(default)]
    pub auto_archive_enabled: bool,
    #[serde(default)]
    pub archive_hour: u8,
}

/// Redis 配置
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RedisConfig {
    /// Redis 连接地址（通过环境变量 REDIS_URL 设置）
    #[serde(default)]
    pub url: String,
    /// 是否启用 Redis（可通过环境变量 REDIS_ENABLED 覆盖）
    #[serde(default)]
    pub enabled: bool,
    /// 连接池大小
    #[serde(default)]
    pub pool_size: usize,
    /// 连接超时时间（秒）
    #[serde(default)]
    pub timeout_secs: u64,
    /// Pub/Sub 频道前缀
    #[serde(default)]
    pub channel_prefix: String,
    /// Stream 最大长度（防止无限增长）
    #[serde(default)]
    pub stream_max_len: u64,
    /// Consumer 批量消费大小
    #[serde(default)]
    pub consumer_batch_size: usize,
    /// Consumer 消费间隔（毫秒）
    #[serde(default)]
    pub consumer_poll_interval_ms: u64,
    /// 是否启用配置同步
    #[serde(default)]
    pub config_sync_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct InitialAdminConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SystemConfigRecord {
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_editable: bool,
    pub is_hot_reloadable: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemConfigItem {
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_editable: bool,
    pub is_hot_reloadable: bool,
}

impl From<SystemConfigRecord> for SystemConfigItem {
    fn from(record: SystemConfigRecord) -> Self {
        Self {
            key: record.key,
            value: record.value,
            value_type: record.value_type,
            description: record.description,
            category: record.category,
            is_editable: record.is_editable,
            is_hot_reloadable: record.is_hot_reloadable,
        }
    }
}

pub type SharedConfig = Arc<RwLock<AppConfig>>;
