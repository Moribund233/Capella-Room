use config::{Config, Environment, File};
use serde::Deserialize;

/// 应用配置结构
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
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
            
        // 手动映射环境变量到配置结构
        let app_config = AppConfig {
            server: ServerConfig {
                host: config.get_string("server.host")
                    .or_else(|_| std::env::var("SERVER_HOST"))
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: config.get::<u16>("server.port")
                    .or_else(|_| std::env::var("SERVER_PORT").map(|s| s.parse().unwrap_or(3000)))
                    .unwrap_or(3000),
            },
            database: DatabaseConfig {
                url: config.get_string("database.url")
                    .or_else(|_| std::env::var("DATABASE_URL"))
                    .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
                max_connections: config.get::<u32>("database.max_connections")
                    .or_else(|_| std::env::var("DATABASE_MAX_CONNECTIONS").map(|s| s.parse().unwrap_or(10)))
                    .unwrap_or(10),
            },
            jwt: JwtConfig {
                secret: config.get_string("jwt.secret")
                    .or_else(|_| std::env::var("JWT_SECRET"))
                    .map_err(|_| anyhow::anyhow!("JWT_SECRET is required"))?,
                expiration_hours: config.get::<i64>("jwt.expiration_hours")
                    .or_else(|_| std::env::var("JWT_EXPIRATION_HOURS").map(|s| s.parse().unwrap_or(24)))
                    .unwrap_or(24),
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

    #[test]
    fn test_default_config() {
        // 设置必需的环境变量
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("JWT_SECRET", "test-secret");
        
        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.jwt.expiration_hours, 24);
    }

    #[test]
    fn test_env_override() {
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("JWT_SECRET", "test-secret");
        std::env::set_var("SERVER_PORT", "8080");
        
        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server.port, 8080);
        
        // 清理
        std::env::remove_var("SERVER_PORT");
    }
}
