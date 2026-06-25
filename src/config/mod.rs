use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod listener;
pub mod loader;
pub mod manager;

pub use listener::BatchMessageConfigListener;
pub use listener::{start_config_listeners, LoggingConfigListener, WebSocketConfigListener};
pub use loader::ConfigLoader;
pub use manager::{ConfigChangeEvent, ConfigManager};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
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
    pub batch_message: BatchMessageConfig,
    #[serde(default)]
    pub mail: MailConfig,
    #[serde(default)]
    pub oauth: OAuthConfig,
    #[serde(default)]
    pub webhook: WebhookConfig,
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
    #[serde(default)]
    pub chunked_upload_enabled: bool,
    #[serde(default)]
    pub default_chunk_size: u32,
    #[serde(default)]
    pub session_ttl_hours: u64,
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
    /// 是否启用 Redis（通过环境变量 REDIS_ENABLED 设置，默认 false）
    #[serde(default)]
    pub enabled: bool,
    /// 连接池大小（通过环境变量 REDIS_POOL_SIZE 设置，默认 10）
    #[serde(default)]
    pub pool_size: usize,
    /// 连接超时时间（秒）（通过环境变量 REDIS_TIMEOUT_SECS 设置，默认 5）
    #[serde(default)]
    pub timeout_secs: u64,
    /// Pub/Sub 频道前缀（通过环境变量 REDIS_CHANNEL_PREFIX 设置，默认 "capella"）
    #[serde(default)]
    pub channel_prefix: String,
    /// Stream 最大长度（通过环境变量 REDIS_STREAM_MAX_LEN 设置，默认 100000）
    #[serde(default)]
    pub stream_max_len: u64,
    /// Consumer 批量消费大小（通过环境变量 REDIS_CONSUMER_BATCH_SIZE 设置，默认 100）
    #[serde(default)]
    pub consumer_batch_size: usize,
    /// Consumer 消费间隔（毫秒）（通过环境变量 REDIS_CONSUMER_POLL_INTERVAL_MS 设置，默认 1000）
    #[serde(default)]
    pub consumer_poll_interval_ms: u64,
    /// 是否启用配置同步（通过环境变量 REDIS_CONFIG_SYNC_ENABLED 设置，默认 true）
    #[serde(default)]
    pub config_sync_enabled: bool,
    /// 是否启用死信队列（通过环境变量 REDIS_DLQ_ENABLED 设置，默认 true）
    #[serde(default)]
    pub dlq_enabled: bool,
    /// DLQ 最大重试次数（通过环境变量 REDIS_DLQ_MAX_RETRIES 设置，默认 3）
    #[serde(default)]
    pub dlq_max_retries: u32,
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

/// 批量消息写入配置
#[derive(Debug, Clone, Deserialize)]
pub struct BatchMessageConfig {
    /// 批量大小，达到此数量立即写入
    pub batch_size: usize,
    /// 刷新间隔（毫秒），达到此时间立即写入
    pub flush_interval_ms: u64,
    /// 队列最大长度，超过此长度将丢弃最旧的消息
    pub max_queue_size: usize,
}

/// 邮件后端类型
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MailBackend {
    #[default]
    Console,
    Smtp,
}

/// 邮件服务配置
#[derive(Debug, Clone, Deserialize)]
pub struct MailConfig {
    /// 邮件后端类型
    #[serde(default)]
    pub backend: MailBackend,
    /// SMTP 主机地址
    #[serde(default)]
    pub smtp_host: String,
    /// SMTP 端口
    #[serde(default)]
    pub smtp_port: u16,
    /// SMTP 用户名
    #[serde(default)]
    pub smtp_username: String,
    /// SMTP 密码（建议通过环境变量注入）
    #[serde(default)]
    pub smtp_password: String,
    /// 是否启用 TLS
    #[serde(default)]
    pub smtp_use_tls: bool,
    /// 发件人地址
    #[serde(default)]
    pub from_address: String,
    /// 发件人名称
    #[serde(default)]
    pub from_name: String,
    /// 验证码有效期（分钟）
    #[serde(default)]
    pub verification_code_ttl: u64,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            backend: MailBackend::default(),
            smtp_host: String::default(),
            smtp_port: 587,
            smtp_username: String::default(),
            smtp_password: String::default(),
            smtp_use_tls: true,
            from_address: String::default(),
            from_name: String::default(),
            verification_code_ttl: 10,
        }
    }
}

/// OAuth 2.0 配置
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    /// OAuth JWT 签名密钥（独立于应用 JWT）
    #[serde(default)]
    pub jwt_secret: Option<String>,
    /// access_token 有效期（秒），默认 3600
    #[serde(default = "default_access_token_ttl")]
    pub access_token_ttl: i64,
    /// refresh_token 有效期（秒），默认 2592000 (30 天)
    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl: i64,
    /// 授权码有效期（秒），默认 300 (5 分钟)
    #[serde(default = "default_auth_code_ttl")]
    pub authorization_code_ttl: i64,
}

fn default_access_token_ttl() -> i64 { 3600 }
fn default_refresh_token_ttl() -> i64 { 2592000 }
fn default_auth_code_ttl() -> i64 { 300 }

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: None,
            access_token_ttl: 3600,
            refresh_token_ttl: 2592000,
            authorization_code_ttl: 300,
        }
    }
}

/// Webhook 配置
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    /// 后台重试扫描间隔（秒），默认 30
    #[serde(default = "default_retry_scan_interval")]
    pub retry_scan_interval_secs: u64,
    /// 默认 HTTP 超时（毫秒），默认 5000
    #[serde(default = "default_webhook_timeout_ms")]
    pub default_timeout_ms: u64,
    /// 默认最大重试次数，默认 3
    #[serde(default = "default_max_retries")]
    pub default_max_retries: i32,
}

fn default_retry_scan_interval() -> u64 { 30 }
fn default_webhook_timeout_ms() -> u64 { 5000 }
fn default_max_retries() -> i32 { 3 }

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            retry_scan_interval_secs: 30,
            default_timeout_ms: 5000,
            default_max_retries: 3,
        }
    }
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

// ==================== 客户端配置 ====================

/// 客户端配置
/// 供前端应用获取的服务端配置子集
#[derive(Debug, Clone, Serialize)]
pub struct ClientConfig {
    /// WebSocket 配置
    pub websocket: ClientWebSocketConfig,
    /// 重连配置
    pub reconnect: ClientReconnectConfig,
    /// 上传配置
    pub upload: ClientUploadConfig,
    /// 系统状态
    pub system: ClientSystemConfig,
    /// 监控配置
    pub monitor: ClientMonitorConfig,
}

/// 客户端监控配置
#[derive(Debug, Clone, Serialize)]
pub struct ClientMonitorConfig {
    /// 监控数据刷新频率（秒）
    pub refresh_interval_secs: u64,
}

/// 客户端 WebSocket 配置
#[derive(Debug, Clone, Serialize)]
pub struct ClientWebSocketConfig {
    /// 心跳间隔（秒）
    pub heartbeat_interval_secs: u64,
    /// 心跳超时（秒）
    pub heartbeat_timeout_secs: u64,
    /// 认证超时（秒）
    pub auth_timeout_secs: u64,
}

/// 客户端重连配置
#[derive(Debug, Clone, Serialize)]
pub struct ClientReconnectConfig {
    /// 基础延迟（毫秒）
    pub base_delay_ms: u64,
    /// 最大延迟（毫秒）
    pub max_delay_ms: u64,
    /// 最大重连次数
    pub max_attempts: u32,
    /// 延迟倍数（指数退避）
    pub multiplier: u32,
}

/// 客户端上传配置
#[derive(Debug, Clone, Serialize)]
pub struct ClientUploadConfig {
    /// 最大文件大小（字节）
    pub max_file_size: usize,
    /// 人类可读的最大文件大小（如 "10MB"）
    pub max_file_size_human: String,
    /// 是否启用分片上传
    pub chunked_upload_enabled: bool,
    /// 默认分片大小（字节）
    pub default_chunk_size: u32,
    /// 上传会话过期时间（小时）
    pub session_ttl_hours: u64,
}

/// 客户端系统配置
#[derive(Debug, Clone, Serialize)]
pub struct ClientSystemConfig {
    /// 系统名称
    pub name: String,
    /// 系统版本
    pub version: String,
    /// 是否处于维护模式
    pub maintenance_mode: bool,
    /// 维护模式提示消息
    pub maintenance_message: String,
}

impl ClientConfig {
    /// 从 AppConfig 创建客户端配置
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            websocket: ClientWebSocketConfig {
                heartbeat_interval_secs: config.websocket.heartbeat_interval_secs,
                heartbeat_timeout_secs: config.websocket.heartbeat_timeout_secs,
                auth_timeout_secs: config.websocket.auth_timeout_secs,
            },
            reconnect: ClientReconnectConfig {
                base_delay_ms: config.reconnect.base_delay_ms,
                max_delay_ms: config.reconnect.max_delay_ms,
                max_attempts: config.reconnect.max_attempts,
                multiplier: config.reconnect.multiplier,
            },
            upload: ClientUploadConfig {
                max_file_size: config.upload.max_file_size,
                max_file_size_human: format_file_size(config.upload.max_file_size),
                chunked_upload_enabled: config.upload.chunked_upload_enabled,
                default_chunk_size: config.upload.default_chunk_size,
                session_ttl_hours: config.upload.session_ttl_hours,
            },
            system: ClientSystemConfig {
                name: config.system.name.clone(),
                version: config.system.version.clone(),
                maintenance_mode: config.system.maintenance_mode,
                maintenance_message: config.system.maintenance_message.clone(),
            },
            monitor: ClientMonitorConfig {
                refresh_interval_secs: 30, // 默认30秒刷新一次
            },
        }
    }
}

/// 将字节大小格式化为人类可读的字符串
fn format_file_size(size: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if size == 0 {
        return "0 B".to_string();
    }
    let exp = (size as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = size as f64 / 1024f64.powi(exp as i32);
    if exp == 0 {
        format!("{} {}", size, UNITS[0])
    } else {
        format!("{:.1} {}", value, UNITS[exp])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(10 * 1024 * 1024), "10.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_app_config_includes_mail() {
        let config = AppConfig {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            jwt: JwtConfig::default(),
            upload: UploadConfig::default(),
            websocket: WebSocketConfig::default(),
            reconnect: ReconnectConfig::default(),
            logging: LoggingConfig::default(),
            system: SystemConfig::default(),
            admin: AdminConfig::default(),
            audit: AuditConfig::default(),
            redis: RedisConfig::default(),
            batch_message: BatchMessageConfig {
                batch_size: 0,
                flush_interval_ms: 0,
                max_queue_size: 0,
            },
            mail: MailConfig::default(),
            oauth: OAuthConfig::default(),
            webhook: WebhookConfig::default(),
        };
        assert_eq!(config.mail.from_address, "");
        assert_eq!(config.mail.backend, MailBackend::Console);
    }
}
