use serde::Deserialize;
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
    pub cors: CorsConfig,
    #[serde(default)]
    pub system: SystemConfig,
    #[serde(default)]
    pub admin: AdminConfig,
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
    /// 连接超时时间（秒），默认30秒
    #[serde(default = "default_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,
    /// 空闲超时时间（秒），默认600秒
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
}

fn default_acquire_timeout_secs() -> u64 {
    30
}

fn default_idle_timeout_secs() -> u64 {
    600
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
    pub max_file_size: usize,
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WebSocketConfig {
    pub heartbeat_interval_secs: u64,
    pub heartbeat_timeout_secs: u64,
    pub auth_timeout_secs: u64,
    pub message_buffer_size: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ReconnectConfig {
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub max_attempts: u32,
    pub multiplier: u32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LoggingConfig {
    pub level: String,
    pub structured: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SystemConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub maintenance_mode: bool,
    pub maintenance_message: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AdminConfig {
    #[serde(default)]
    pub initial: InitialAdminConfig,
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
