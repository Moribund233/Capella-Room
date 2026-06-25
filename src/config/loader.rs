use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

use super::AppConfig;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> Result<AppConfig> {
        Self::load_with_path(None)
    }

    pub fn load_with_path(config_path: Option<&str>) -> Result<AppConfig> {
        info!("Loading configuration...");

        // 从 .env 文件加载环境变量（如果存在）
        if Path::new(".env").exists() {
            debug!("Loading .env file");
            dotenvy::from_filename(".env").ok();
        }

        // 确定配置文件路径
        let config_file = config_path
            .map(|s| s.to_string())
            .or_else(|| std::env::var("CONFIG_FILE").ok())
            .unwrap_or_else(|| "config.toml".to_string());

        let config_path = Path::new(&config_file);

        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "Configuration file '{}' not found. Please create a config.toml file. \
                 You can copy from config.toml.example or refer to the documentation.",
                config_file
            ));
        }

        info!("Loading config from file: {}", config_file);
        let mut config = Self::load_from_file(&config_file)?;

        // 从环境变量加载敏感配置（必须存在，否则报错）
        Self::load_required_env_configs(&mut config)?;

        // 从环境变量加载可选配置
        Self::load_optional_env_configs(&mut config);

        Self::validate_config(&mut config)?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    pub fn load_from_file_only(path: &str) -> Result<AppConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let mut config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        Self::validate_config(&mut config)?;

        Ok(config)
    }

    fn load_from_file(path: &str) -> Result<AppConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        Ok(config)
    }

    /// 从环境变量加载必须存在的敏感配置
    ///
    /// # 说明
    /// 这些配置必须通过环境变量设置，不存在时会立即返回错误
    fn load_required_env_configs(config: &mut AppConfig) -> Result<()> {
        debug!("Loading required environment variable configs...");

        // 服务器主机地址（必须）
        config.server.host = std::env::var("SERVER_HOST").map_err(|_| {
            anyhow::anyhow!("SERVER_HOST is required. Please set it via environment variable.")
        })?;
        debug!("Loaded server.host from environment");

        // 服务器端口（必须）
        let port_str = std::env::var("SERVER_PORT").map_err(|_| {
            anyhow::anyhow!("SERVER_PORT is required. Please set it via environment variable.")
        })?;
        config.server.port = port_str
            .parse()
            .map_err(|_| anyhow::anyhow!("SERVER_PORT must be a valid number"))?;
        debug!("Loaded server.port from environment");

        // 数据库连接地址（必须）
        let db_url = std::env::var("DATABASE_URL").map_err(|_| {
            anyhow::anyhow!(
                "DATABASE_URL is required. Please set it via environment variable. \
                For security reasons, this cannot be set in config.toml."
            )
        })?;
        config.database.url = Some(db_url);
        debug!("Loaded database.url from environment");

        // JWT 密钥（必须）
        let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| {
            anyhow::anyhow!(
                "JWT_SECRET is required. Please set it via environment variable. \
                For security reasons, this cannot be set in config.toml."
            )
        })?;
        config.jwt.secret = Some(jwt_secret);
        debug!("Loaded jwt.secret from environment");

        Ok(())
    }

    /// 从环境变量加载可选配置
    ///
    /// # 说明
    /// 这些配置可以通过环境变量覆盖，如果不存在则使用 config.toml 中的值或默认值
    fn load_optional_env_configs(config: &mut AppConfig) {
        debug!("Loading optional environment variable configs...");

        // 数据库可选配置
        if let Ok(max_conn) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            if let Ok(max) = max_conn.parse() {
                debug!("Overriding database.max_connections from environment");
                config.database.max_connections = max;
            }
        }

        if let Ok(timeout) = std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECS") {
            if let Ok(t) = timeout.parse() {
                debug!("Overriding database.acquire_timeout_secs from environment");
                config.database.acquire_timeout_secs = t;
            }
        }

        if let Ok(timeout) = std::env::var("DATABASE_IDLE_TIMEOUT_SECS") {
            if let Ok(t) = timeout.parse() {
                debug!("Overriding database.idle_timeout_secs from environment");
                config.database.idle_timeout_secs = t;
            }
        }

        // JWT 可选配置
        if let Ok(hours) = std::env::var("JWT_EXPIRATION_HOURS") {
            if let Ok(h) = hours.parse() {
                debug!("Overriding jwt.expiration_hours from environment");
                config.jwt.expiration_hours = h;
            }
        }

        // 上传配置
        if let Ok(max_size) = std::env::var("UPLOAD_MAX_FILE_SIZE") {
            if let Ok(size) = max_size.parse() {
                debug!("Overriding upload.max_file_size from environment");
                config.upload.max_file_size = size;
            }
        }

        if let Ok(base_url) = std::env::var("UPLOAD_BASE_URL") {
            debug!("Overriding upload.base_url from environment");
            config.upload.base_url = base_url;
        }

        if let Ok(enabled) = std::env::var("UPLOAD_CHUNKED_ENABLED") {
            if let Ok(e) = enabled.parse() {
                debug!("Overriding upload.chunked_upload_enabled from environment");
                config.upload.chunked_upload_enabled = e;
            }
        }

        if let Ok(size) = std::env::var("UPLOAD_DEFAULT_CHUNK_SIZE") {
            if let Ok(s) = size.parse() {
                debug!("Overriding upload.default_chunk_size from environment");
                config.upload.default_chunk_size = s;
            }
        }

        // 日志配置
        if let Ok(level) = std::env::var("LOG_LEVEL") {
            debug!("Overriding logging.level from environment");
            config.logging.level = level;
        }

        // 系统配置
        if let Ok(maintenance) = std::env::var("MAINTENANCE_MODE") {
            if let Ok(m) = maintenance.parse() {
                debug!("Overriding system.maintenance_mode from environment");
                config.system.maintenance_mode = m;
            }
        }

        // Redis 配置（完全从环境变量读取，config.toml 中的配置被忽略）
        if let Ok(enabled) = std::env::var("REDIS_ENABLED") {
            if let Ok(e) = enabled.parse::<bool>() {
                debug!("Loading redis.enabled from environment");
                config.redis.enabled = e;
            }
        }

        if let Ok(url) = std::env::var("REDIS_URL") {
            debug!("Loading redis.url from environment");
            config.redis.url = url;
        }

        if let Ok(pool_size) = std::env::var("REDIS_POOL_SIZE") {
            if let Ok(size) = pool_size.parse() {
                debug!("Loading redis.pool_size from environment");
                config.redis.pool_size = size;
            }
        }

        if let Ok(timeout) = std::env::var("REDIS_TIMEOUT_SECS") {
            if let Ok(t) = timeout.parse() {
                debug!("Loading redis.timeout_secs from environment");
                config.redis.timeout_secs = t;
            }
        }

        if let Ok(prefix) = std::env::var("REDIS_CHANNEL_PREFIX") {
            debug!("Loading redis.channel_prefix from environment");
            config.redis.channel_prefix = prefix;
        }

        if let Ok(max_len) = std::env::var("REDIS_STREAM_MAX_LEN") {
            if let Ok(len) = max_len.parse() {
                debug!("Loading redis.stream_max_len from environment");
                config.redis.stream_max_len = len;
            }
        }

        if let Ok(batch_size) = std::env::var("REDIS_CONSUMER_BATCH_SIZE") {
            if let Ok(size) = batch_size.parse() {
                debug!("Loading redis.consumer_batch_size from environment");
                config.redis.consumer_batch_size = size;
            }
        }

        if let Ok(interval) = std::env::var("REDIS_CONSUMER_POLL_INTERVAL_MS") {
            if let Ok(i) = interval.parse() {
                debug!("Loading redis.consumer_poll_interval_ms from environment");
                config.redis.consumer_poll_interval_ms = i;
            }
        }

        if let Ok(sync_enabled) = std::env::var("REDIS_CONFIG_SYNC_ENABLED") {
            if let Ok(e) = sync_enabled.parse() {
                debug!("Loading redis.config_sync_enabled from environment");
                config.redis.config_sync_enabled = e;
            }
        }

        // OAuth 配置
        if let Ok(secret) = std::env::var("OAUTH_JWT_SECRET") {
            debug!("Loading oauth.jwt_secret from environment");
            config.oauth.jwt_secret = Some(secret);
        }

        // 邮件配置
        if let Ok(backend) = std::env::var("SMTP_BACKEND") {
            if backend.eq_ignore_ascii_case("smtp") {
                debug!("Loading mail.backend from environment: smtp");
                config.mail.backend = super::MailBackend::Smtp;
            }
        }
        if let Ok(password) = std::env::var("SMTP_PASSWORD") {
            debug!("Loading mail.smtp_password from environment");
            config.mail.smtp_password = password;
        }
        if let Ok(host) = std::env::var("SMTP_HOST") {
            debug!("Loading mail.smtp_host from environment");
            config.mail.smtp_host = host;
        }
        if let Ok(port) = std::env::var("SMTP_PORT") {
            if let Ok(p) = port.parse() {
                debug!("Loading mail.smtp_port from environment");
                config.mail.smtp_port = p;
            }
        }
        if let Ok(username) = std::env::var("SMTP_USERNAME") {
            debug!("Loading mail.smtp_username from environment");
            config.mail.smtp_username = username;
        }
        if let Ok(from) = std::env::var("SMTP_FROM_ADDRESS") {
            debug!("Loading mail.from_address from environment");
            config.mail.from_address = from;
        }
        if let Ok(name) = std::env::var("SMTP_FROM_NAME") {
            debug!("Loading mail.from_name from environment");
            config.mail.from_name = name;
        }
    }

    fn validate_config(config: &mut AppConfig) -> Result<()> {
        // 验证上传配置
        if config.upload.max_file_size == 0 {
            return Err(anyhow::anyhow!(
                "upload.max_file_size cannot be 0. Please set a valid value in config.toml"
            ));
        }

        // 验证 WebSocket 心跳配置
        if config.websocket.heartbeat_timeout_secs <= config.websocket.heartbeat_interval_secs {
            warn!(
                "websocket.heartbeat_timeout_secs ({}) should be greater than heartbeat_interval_secs ({})",
                config.websocket.heartbeat_timeout_secs, config.websocket.heartbeat_interval_secs
            );
        }

        // 验证批量消息配置
        if config.batch_message.batch_size == 0 {
            return Err(anyhow::anyhow!(
                "batch_message.batch_size cannot be 0. Please set a valid value in config.toml"
            ));
        }
        if config.batch_message.flush_interval_ms == 0 {
            return Err(anyhow::anyhow!(
                "batch_message.flush_interval_ms cannot be 0. Please set a valid value in config.toml"
            ));
        }
        if config.batch_message.max_queue_size == 0 {
            return Err(anyhow::anyhow!(
                "batch_message.max_queue_size cannot be 0. Please set a valid value in config.toml"
            ));
        }

        // 验证 Redis 配置（如果启用）
        if config.redis.enabled && config.redis.url.is_empty() {
            return Err(anyhow::anyhow!(
                "REDIS_URL is required when Redis is enabled. Please set it via environment variable."
            ));
        }

        // 验证邮件配置（如果启用 SMTP）
        if config.mail.backend == super::MailBackend::Smtp {
            if config.mail.smtp_host.is_empty() {
                return Err(anyhow::anyhow!(
                    "mail.smtp_host is required when mail.backend is 'smtp'. "
                ));
            }
            if config.mail.from_address.is_empty() {
                return Err(anyhow::anyhow!(
                    "mail.from_address is required when mail.backend is 'smtp'. "
                ));
            }
            if config.mail.from_name.is_empty() {
                return Err(anyhow::anyhow!(
                    "mail.from_name is required when mail.backend is 'smtp'. "
                ));
            }
            if config.mail.smtp_username.is_empty() {
                return Err(anyhow::anyhow!(
                    "mail.smtp_username is required when mail.backend is 'smtp'. "
                ));
            }
            if config.mail.smtp_password.is_empty() {
                return Err(anyhow::anyhow!(
                    "mail.smtp_password is required when mail.backend is 'smtp'. Set SMTP_PASSWORD environment variable."
                ));
            }
        }

        Ok(())
    }

    pub fn get_upload_dir() -> Result<String> {
        std::env::var("UPLOAD_DIR")
            .map_err(|_| anyhow::anyhow!("UPLOAD_DIR environment variable is required"))
    }

    pub fn apply_database_overrides(config: &mut AppConfig, db_configs: &HashMap<String, String>) {
        debug!("Applying database configuration overrides...");

        if let Some(value) = db_configs.get("jwt.expiration_hours") {
            if let Ok(hours) = value.parse() {
                config.jwt.expiration_hours = hours;
            }
        }

        if let Some(value) = db_configs.get("upload.max_file_size") {
            if let Ok(size) = value.parse() {
                config.upload.max_file_size = size;
            }
        }

        if let Some(value) = db_configs.get("upload.base_url") {
            config.upload.base_url = value.clone();
        }

        if let Some(value) = db_configs.get("websocket.heartbeat_interval_secs") {
            if let Ok(secs) = value.parse() {
                config.websocket.heartbeat_interval_secs = secs;
            }
        }

        if let Some(value) = db_configs.get("websocket.heartbeat_timeout_secs") {
            if let Ok(secs) = value.parse() {
                config.websocket.heartbeat_timeout_secs = secs;
            }
        }

        if let Some(value) = db_configs.get("websocket.auth_timeout_secs") {
            if let Ok(secs) = value.parse() {
                config.websocket.auth_timeout_secs = secs;
            }
        }

        if let Some(value) = db_configs.get("logging.level") {
            config.logging.level = value.clone();
        }

        if let Some(value) = db_configs.get("system.maintenance_mode") {
            if let Ok(mode) = value.parse() {
                config.system.maintenance_mode = mode;
            }
        }

        if let Some(value) = db_configs.get("system.maintenance_message") {
            config.system.maintenance_message = value.clone();
        }
    }
}
