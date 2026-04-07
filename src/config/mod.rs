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
    pub cors: CorsConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_log_retention_days")]
    pub log_retention_days: i32,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default = "default_flush_interval_seconds")]
    pub flush_interval_seconds: u64,
    #[serde(default)]
    pub excluded_paths: Vec<String>,
    #[serde(default = "default_true")]
    pub alert_enabled: bool,
    #[serde(default = "default_alert_cooldown_minutes")]
    pub alert_cooldown_minutes: i32,
    #[serde(default = "default_true")]
    pub auto_archive_enabled: bool,
    #[serde(default = "default_archive_hour")]
    pub archive_hour: u8,
}

fn default_log_retention_days() -> i32 {
    90
}

fn default_buffer_size() -> usize {
    100
}

fn default_flush_interval_seconds() -> u64 {
    5
}

fn default_true() -> bool {
    true
}

fn default_alert_cooldown_minutes() -> i32 {
    10
}

fn default_archive_hour() -> u8 {
    3
}

/// Redis 配置
/// 支持通过环境变量配置
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis 连接地址，默认 "redis://127.0.0.1:6379"
    #[serde(default = "default_redis_url")]
    pub url: String,
    /// 是否启用 Redis，默认 false
    #[serde(default = "default_false")]
    pub enabled: bool,
    /// 连接池大小，默认 10
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: usize,
    /// 连接超时时间（秒），默认 5 秒
    #[serde(default = "default_redis_timeout_secs")]
    pub timeout_secs: u64,
    /// Pub/Sub 频道前缀，默认 "seredeli"
    #[serde(default = "default_redis_channel_prefix")]
    pub channel_prefix: String,
    /// Stream 最大长度（防止无限增长），默认 100000
    #[serde(default = "default_stream_max_len")]
    pub stream_max_len: u64,
    /// Consumer 批量消费大小，默认 100
    #[serde(default = "default_consumer_batch_size")]
    pub consumer_batch_size: usize,
    /// Consumer 消费间隔（毫秒），默认 1000
    #[serde(default = "default_consumer_poll_interval_ms")]
    pub consumer_poll_interval_ms: u64,
    /// 是否启用配置同步，默认 true（当 Redis 启用时）
    #[serde(default = "default_config_sync_enabled")]
    pub config_sync_enabled: bool,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: default_redis_url(),
            enabled: default_false(),
            pool_size: default_redis_pool_size(),
            timeout_secs: default_redis_timeout_secs(),
            channel_prefix: default_redis_channel_prefix(),
            stream_max_len: default_stream_max_len(),
            consumer_batch_size: default_consumer_batch_size(),
            consumer_poll_interval_ms: default_consumer_poll_interval_ms(),
            config_sync_enabled: default_config_sync_enabled(),
        }
    }
}

fn default_redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
}

fn default_redis_pool_size() -> usize {
    std::env::var("REDIS_POOL_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

fn default_redis_timeout_secs() -> u64 {
    std::env::var("REDIS_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}

fn default_redis_channel_prefix() -> String {
    std::env::var("REDIS_CHANNEL_PREFIX").unwrap_or_else(|_| "seredeli".to_string())
}

fn default_stream_max_len() -> u64 {
    std::env::var("REDIS_STREAM_MAX_LEN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100000)
}

fn default_consumer_batch_size() -> usize {
    std::env::var("REDIS_CONSUMER_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

fn default_consumer_poll_interval_ms() -> u64 {
    std::env::var("REDIS_CONSUMER_POLL_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
}

fn default_config_sync_enabled() -> bool {
    std::env::var("REDIS_CONFIG_SYNC_ENABLED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(true)
}

fn default_false() -> bool {
    std::env::var("REDIS_ENABLED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(false)
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
