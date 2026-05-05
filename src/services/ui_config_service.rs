//! 用户 UI 配置服务
//!
//! 提供用户界面配置的 CRUD 操作

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::ui_config::{SaveUIConfigRequest, UIConfigResponse, UserUIConfig};

/// UI 配置服务
pub struct UIConfigService {
    pool: PgPool,
}

impl UIConfigService {
    /// 创建服务实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取用户 UI 配置
    /// 注意：app_config 字段已从数据库中移除
    pub async fn get_user_config(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UIConfigResponse>, AppError> {
        let config: Option<UserUIConfig> = sqlx::query_as::<_, UserUIConfig>(
            r#"
            SELECT id, user_id, theme_config, sidebar_config,
                   quickbar_config, dock_config, created_at, updated_at
            FROM user_ui_configs
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(config.map(|c| c.to_response()))
    }

    /// 保存用户 UI 配置
    /// 注意：应用配置(app)已从云端配置中移除，改为从 ui.ts 配置文件读取
    pub async fn save_user_config(
        &self,
        user_id: Uuid,
        request: SaveUIConfigRequest,
    ) -> Result<(), AppError> {
        let theme_config = request
            .theme
            .map(|c| serde_json::to_value(c).unwrap_or_default());
        let sidebar_config = request
            .sidebar
            .map(|c| serde_json::to_value(c).unwrap_or_default());
        let quickbar_config = request
            .quickbar
            .map(|c| serde_json::to_value(c).unwrap_or_default());
        let dock_config = request
            .dock
            .map(|c| serde_json::to_value(c).unwrap_or_default());

        sqlx::query(
            r#"
            INSERT INTO user_ui_configs (
                user_id, theme_config, sidebar_config,
                quickbar_config, dock_config
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id)
            DO UPDATE SET
                theme_config = COALESCE(EXCLUDED.theme_config, user_ui_configs.theme_config),
                sidebar_config = COALESCE(EXCLUDED.sidebar_config, user_ui_configs.sidebar_config),
                quickbar_config = COALESCE(EXCLUDED.quickbar_config, user_ui_configs.quickbar_config),
                dock_config = COALESCE(EXCLUDED.dock_config, user_ui_configs.dock_config),
                updated_at = NOW()
            "#
        )
        .bind(user_id)
        .bind(theme_config)
        .bind(sidebar_config)
        .bind(quickbar_config)
        .bind(dock_config)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 删除用户 UI 配置（重置）
    pub async fn delete_user_config(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM user_ui_configs
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 更新主题配置
    pub async fn update_theme_config(
        &self,
        user_id: Uuid,
        theme_config: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_ui_configs (user_id, theme_config)
            VALUES ($1, $2)
            ON CONFLICT (user_id)
            DO UPDATE SET
                theme_config = EXCLUDED.theme_config,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(theme_config)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 更新侧边栏配置
    pub async fn update_sidebar_config(
        &self,
        user_id: Uuid,
        sidebar_config: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_ui_configs (user_id, sidebar_config)
            VALUES ($1, $2)
            ON CONFLICT (user_id)
            DO UPDATE SET
                sidebar_config = EXCLUDED.sidebar_config,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(sidebar_config)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 更新 QuickBar 配置
    pub async fn update_quickbar_config(
        &self,
        user_id: Uuid,
        quickbar_config: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_ui_configs (user_id, quickbar_config)
            VALUES ($1, $2)
            ON CONFLICT (user_id)
            DO UPDATE SET
                quickbar_config = EXCLUDED.quickbar_config,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(quickbar_config)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }
}
