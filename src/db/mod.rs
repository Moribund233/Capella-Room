use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

use crate::config::DatabaseConfig;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> anyhow::Result<Self> {
        let url = config
            .url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database URL is required"))?;

        info!("Connecting to database...");

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_secs))
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_secs))
            .connect(url)
            .await?;

        sqlx::query("SELECT 1").fetch_one(&pool).await?;

        info!("Database connected successfully");

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        info!("Running database migrations...");

        sqlx::migrate!("./migrations").run(&self.pool).await?;

        info!("Database migrations completed");
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let url = std::env::var("DATABASE_URL");
        if url.is_err() {
            eprintln!("Skipping test_database_connection: DATABASE_URL not set");
            return;
        }

        let config = DatabaseConfig {
            url: Some(url.unwrap()),
            max_connections: 5,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,
        };
        let result = Database::new(&config).await;
        assert!(result.is_ok());
    }
}
