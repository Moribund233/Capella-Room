use config::{Config, Environment, File};
use serde::Deserialize;

/// 应用配置结构
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub upload: UploadConfig,
}

/// 文件上传配置
#[derive(Debug, Clone, Deserialize)]
pub struct UploadConfig {
    /// 最大文件大小（字节），默认 10MB
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,
    /// 文件访问的基础URL路径，默认 "/uploads"
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

fn default_max_file_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_base_url() -> String {
    "/uploads".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

impl AppConfig {
    /// 获取上传目录路径（从环境变量 UPLOAD_DIR 读取）
    pub fn upload_dir() -> anyhow::Result<String> {
        std::env::var("UPLOAD_DIR")
            .map_err(|_| anyhow::anyhow!("UPLOAD_DIR environment variable is required"))
    }

    /// 从环境变量加载配置
    /// 加载顺序（后面的覆盖前面的）：
    /// 1. 默认配置
    /// 2. .env.{APP_ENV} 文件（dotenvy 加载）
    /// 3. .env 文件（dotenvy 加载）
    /// 4. 环境变量（APP_前缀，双下划线分隔）
    pub fn from_env() -> anyhow::Result<Self> {
        // 先加载 .env 文件到环境变量
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        
        // 尝试加载特定环境的 .env 文件
        let env_file = format!(".env.{}", env);
        if std::path::Path::new(&env_file).exists() {
            dotenvy::from_filename(&env_file).ok();
        } else if std::path::Path::new(".env").exists() {
            dotenvy::dotenv().ok();
        }
        
        let config = Config::builder()
            // 1. 默认配置
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("database.max_connections", 10)?
            .set_default("jwt.expiration_hours", 24)?
            .set_default("upload.max_file_size", default_max_file_size() as i64)?
            .set_default("upload.base_url", default_base_url())?
            // 2. 环境变量（APP_前缀，双下划线分隔）
            // 注意：环境变量名需要与结构体字段匹配
            // APP_SERVER__HOST -> server.host
            // APP_DATABASE__URL -> database.url
            // APP_JWT__SECRET -> jwt.secret
            .add_source(
                Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__")
            )
            // 3. 也支持不带前缀的环境变量（直接从 .env 加载的）
            .add_source(
                Environment::default()
                    .ignore_empty(true)
            )
            .build()?;
            
        // 手动映射环境变量到配置结构（优先使用环境变量）
        let app_config = AppConfig {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST")
                    .or_else(|_| config.get_string("server.host"))
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| config.get::<u16>("server.port").ok())
                    .unwrap_or(3000),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .or_else(|_| config.get_string("database.url"))
                    .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| config.get::<u32>("database.max_connections").ok())
                    .unwrap_or(10),
            },
            jwt: JwtConfig {
                secret: std::env::var("JWT_SECRET")
                    .or_else(|_| config.get_string("jwt.secret"))
                    .map_err(|_| anyhow::anyhow!("JWT_SECRET is required"))?,
                expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| config.get::<i64>("jwt.expiration_hours").ok())
                    .unwrap_or(24),
            },
            upload: UploadConfig {
                max_file_size: std::env::var("UPLOAD_MAX_FILE_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| config.get::<i64>("upload.max_file_size").ok())
                    .map(|v| v as usize)
                    .unwrap_or_else(default_max_file_size),
                base_url: std::env::var("UPLOAD_BASE_URL")
                    .or_else(|_| config.get_string("upload.base_url"))
                    .unwrap_or_else(|_| default_base_url()),
            },
        };
            
        Ok(app_config)
    }

    /// 从指定文件加载配置（用于测试）
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let config = Config::builder()
            .add_source(File::with_name(path).required(true))
            .build()?
            .try_deserialize()?;
            
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 使用互斥锁确保测试串行执行
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_env() {
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("JWT_SECRET", "test-secret");
    }

    fn cleanup_env() {
        std::env::remove_var("SERVER_PORT");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("JWT_SECRET");
    }

    #[test]
    fn test_default_config() {
        let _lock = TEST_MUTEX.lock().unwrap();
        cleanup_env();
        setup_env();

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.jwt.expiration_hours, 24);

        cleanup_env();
    }

    #[test]
    fn test_env_override() {
        let _lock = TEST_MUTEX.lock().unwrap();
        cleanup_env();
        setup_env();
        std::env::set_var("SERVER_PORT", "8080");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server.port, 8080);

        cleanup_env();
    }
}
