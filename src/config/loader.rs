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

        // 第1步：优先从 .env 文件加载基础环境变量（包含 APP_ENV）
        if Path::new(".env").exists() {
            debug!("Loading base .env file");
            dotenvy::from_filename(".env").ok();
        }

        // 第2步：确定运行环境（优先级：系统环境变量 > .env文件中的APP_ENV > 默认值）
        let env = std::env::var("APP_ENV")
            .ok()
            .filter(|e| !e.is_empty())
            .unwrap_or_else(|| "development".to_string());

        info!("Application environment: {}", env);

        // 第3步：加载对应环境的 .env.{env} 文件
        let env_file = format!(".env.{}", env);
        if Path::new(&env_file).exists() {
            debug!("Loading environment file: {}", env_file);
            dotenvy::from_filename(&env_file).ok();
        }

        // 第4步：确定配置文件路径
        let config_file = config_path
            .map(|s| s.to_string())
            .or_else(|| std::env::var("CONFIG_FILE").ok())
            .or_else(|| {
                // 根据环境选择对应的配置文件
                let env_config = format!("config.{}.toml", env);
                if Path::new(&env_config).exists() {
                    Some(env_config)
                } else {
                    None
                }
            })
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

        // 第5步：应用环境变量覆盖
        Self::apply_env_overrides(&mut config);

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

    fn apply_env_overrides(config: &mut AppConfig) {
        debug!("Applying environment variable overrides...");

        if let Ok(host) = std::env::var("SERVER_HOST") {
            debug!("Overriding server.host from environment");
            config.server.host = host;
        }

        if let Ok(port) = std::env::var("SERVER_PORT") {
            if let Ok(port) = port.parse() {
                debug!("Overriding server.port from environment");
                config.server.port = port;
            }
        }

        if let Ok(url) = std::env::var("DATABASE_URL") {
            debug!("Overriding database.url from environment");
            config.database.url = Some(url);
        }

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

        if let Ok(secret) = std::env::var("JWT_SECRET") {
            debug!("Overriding jwt.secret from environment");
            config.jwt.secret = Some(secret);
        }

        if let Ok(hours) = std::env::var("JWT_EXPIRATION_HOURS") {
            if let Ok(h) = hours.parse() {
                debug!("Overriding jwt.expiration_hours from environment");
                config.jwt.expiration_hours = h;
            }
        }

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

        if let Ok(level) = std::env::var("LOG_LEVEL") {
            debug!("Overriding logging.level from environment");
            config.logging.level = level;
        }

        if let Ok(maintenance) = std::env::var("MAINTENANCE_MODE") {
            if let Ok(m) = maintenance.parse() {
                debug!("Overriding system.maintenance_mode from environment");
                config.system.maintenance_mode = m;
            }
        }
    }

    fn validate_config(config: &mut AppConfig) -> Result<()> {
        // 敏感配置必须从环境变量读取，不允许从config.toml读取
        let db_url = std::env::var("DATABASE_URL").ok();
        if db_url.is_none() {
            return Err(anyhow::anyhow!(
                "DATABASE_URL is required. Please set it via environment variable. \
                 For security reasons, this cannot be set in config.toml."
            ));
        }
        config.database.url = db_url;

        let jwt_secret = std::env::var("JWT_SECRET").ok();
        if jwt_secret.is_none() {
            return Err(anyhow::anyhow!(
                "JWT_SECRET is required. Please set it via environment variable. \
                 For security reasons, this cannot be set in config.toml."
            ));
        }
        config.jwt.secret = jwt_secret;

        if config.upload.max_file_size == 0 {
            return Err(anyhow::anyhow!(
                "upload.max_file_size cannot be 0. Please set a valid value in config.toml"
            ));
        }

        if config.websocket.heartbeat_timeout_secs <= config.websocket.heartbeat_interval_secs {
            warn!(
                "websocket.heartbeat_timeout_secs ({}) should be greater than heartbeat_interval_secs ({})",
                config.websocket.heartbeat_timeout_secs,
                config.websocket.heartbeat_interval_secs
            );
        }

        if config.server.host.is_empty() {
            return Err(anyhow::anyhow!("server.host is required in config.toml"));
        }

        if config.server.port == 0 {
            return Err(anyhow::anyhow!("server.port is required in config.toml"));
        }

        Ok(())
    }

    pub fn get_upload_dir() -> Result<String> {
        std::env::var("UPLOAD_DIR")
            .map_err(|_| anyhow::anyhow!("UPLOAD_DIR environment variable is required"))
    }

    pub fn apply_database_overrides(config: &mut AppConfig, db_configs: &HashMap<String, String>) {
        debug!("Applying database configuration overrides...");

        if let Some(value) = db_configs.get("server.host") {
            config.server.host = value.clone();
        }

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
