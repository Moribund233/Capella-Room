use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

use crate::config::DatabaseConfig;

/// 数据库连接池
#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// 创建数据库连接池
    pub async fn new(config: &DatabaseConfig) -> anyhow::Result<Self> {
        info!("Connecting to database...");

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .connect(&config.url)
            .await?;

        // 测试连接
        sqlx::query("SELECT 1").fetch_one(&pool).await?;

        info!("Database connected successfully");

        Ok(Self { pool })
    }

    /// 运行数据库迁移
    pub async fn migrate(&self) -> anyhow::Result<()> {
        info!("Running database migrations...");

        sqlx::migrate!("./migrations").run(&self.pool).await?;

        info!("Database migrations completed");
        Ok(())
    }

    /// 获取连接池
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要数据库连接
    // 在实际项目中应该使用测试数据库

    async fn create_test_db() -> anyhow::Result<Database> {
        let config = DatabaseConfig {
            url: std::env::var("DATABASE_URL")?,
            max_connections: 5,
        };
        Database::new(&config).await
    }

    #[tokio::test]
    async fn test_database_connection() {
        let result = create_test_db().await;
        assert!(result.is_ok());
    }
}
