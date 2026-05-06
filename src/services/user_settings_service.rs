use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::user_settings::{
    AccessibilitySettings, LanguageSettings, MediaSettings, MessageSettings, NotificationSettings,
    PrivacySettings, UpdateRoomSettingsRequest, UpdateUserSettingsRequest, UserRoomSettings,
    UserRoomSettingsResponse, UserSettingsResponse,
};

/// 用户设置服务
pub struct UserSettingsService {
    pool: PgPool,
}

impl UserSettingsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取用户完整设置
    /// 如果 user_settings 表没有该用户的行，返回所有分组的默认值
    pub async fn get_user_settings(&self, user_id: Uuid) -> Result<UserSettingsResponse> {
        let row = sqlx::query_as::<_, UserSettingsRow>(
            r#"
            SELECT
                notification_settings,
                privacy_settings,
                message_settings,
                language_settings,
                accessibility_settings,
                media_settings
            FROM user_settings
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        match row {
            Some(r) => Ok(UserSettingsResponse {
                notification: r
                    .notification_settings
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                privacy: r
                    .privacy_settings
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                message: r
                    .message_settings
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                language: r
                    .language_settings
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                accessibility: r
                    .accessibility_settings
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                media: r
                    .media_settings
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
            }),
            None => Ok(UserSettingsResponse {
                notification: NotificationSettings::default(),
                privacy: PrivacySettings::default(),
                message: MessageSettings::default(),
                language: LanguageSettings::default(),
                accessibility: AccessibilitySettings::default(),
                media: MediaSettings::default(),
            }),
        }
    }

    /// 部分更新用户设置
    /// 只更新请求中包含的分组，其他分组保持不变
    pub async fn update_user_settings(
        &self,
        user_id: Uuid,
        request: UpdateUserSettingsRequest,
    ) -> Result<UserSettingsResponse> {
        // 先确保 user_settings 行存在
        sqlx::query(
            r#"
            INSERT INTO user_settings (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        // 逐个分组更新（只更新请求中存在的分组）
        if let Some(settings) = request.notification {
            let value = serde_json::to_value(&settings).unwrap_or_default();
            sqlx::query(
                r#"
                UPDATE user_settings
                SET notification_settings = $1::jsonb, updated_at = NOW()
                WHERE user_id = $2
                "#,
            )
            .bind(value)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        }

        if let Some(settings) = request.privacy {
            let value = serde_json::to_value(&settings).unwrap_or_default();
            sqlx::query(
                r#"
                UPDATE user_settings
                SET privacy_settings = $1::jsonb, updated_at = NOW()
                WHERE user_id = $2
                "#,
            )
            .bind(value)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        }

        if let Some(settings) = request.message {
            let value = serde_json::to_value(&settings).unwrap_or_default();
            sqlx::query(
                r#"
                UPDATE user_settings
                SET message_settings = $1::jsonb, updated_at = NOW()
                WHERE user_id = $2
                "#,
            )
            .bind(value)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        }

        if let Some(settings) = request.language {
            let value = serde_json::to_value(&settings).unwrap_or_default();
            sqlx::query(
                r#"
                UPDATE user_settings
                SET language_settings = $1::jsonb, updated_at = NOW()
                WHERE user_id = $2
                "#,
            )
            .bind(value)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        }

        if let Some(settings) = request.accessibility {
            let value = serde_json::to_value(&settings).unwrap_or_default();
            sqlx::query(
                r#"
                UPDATE user_settings
                SET accessibility_settings = $1::jsonb, updated_at = NOW()
                WHERE user_id = $2
                "#,
            )
            .bind(value)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        }

        if let Some(settings) = request.media {
            let value = serde_json::to_value(&settings).unwrap_or_default();
            sqlx::query(
                r#"
                UPDATE user_settings
                SET media_settings = $1::jsonb, updated_at = NOW()
                WHERE user_id = $2
                "#,
            )
            .bind(value)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        }

        // 返回更新后的完整设置
        self.get_user_settings(user_id).await
    }

    // ============================================================
    // 房间级设置
    // ============================================================

    /// 获取用户在某个房间的设置
    pub async fn get_room_settings(
        &self,
        user_id: Uuid,
        room_id: Uuid,
    ) -> Result<UserRoomSettingsResponse> {
        let settings = sqlx::query_as::<_, UserRoomSettings>(
            r#"
            SELECT id, user_id, room_id, is_muted, notification_preference,
                   is_pinned, custom_name, custom_color, created_at, updated_at
            FROM user_room_settings
            WHERE user_id = $1 AND room_id = $2
            "#,
        )
        .bind(user_id)
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        match settings {
            Some(s) => Ok(s.into()),
            None => Ok(UserRoomSettingsResponse {
                room_id,
                is_muted: false,
                notification_preference: "all".to_string(),
                is_pinned: false,
                custom_name: None,
                custom_color: None,
            }),
        }
    }

    /// 获取用户在所有房间的设置列表
    pub async fn list_room_settings(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserRoomSettingsResponse>> {
        let settings = sqlx::query_as::<_, UserRoomSettings>(
            r#"
            SELECT id, user_id, room_id, is_muted, notification_preference,
                   is_pinned, custom_name, custom_color, created_at, updated_at
            FROM user_room_settings
            WHERE user_id = $1
            ORDER BY is_pinned DESC, updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(settings.into_iter().map(|s| s.into()).collect())
    }

    /// 更新用户在某个房间的设置
    pub async fn update_room_settings(
        &self,
        user_id: Uuid,
        room_id: Uuid,
        request: UpdateRoomSettingsRequest,
    ) -> Result<UserRoomSettingsResponse> {
        // UPSERT: 存在则更新，不存在则插入
        let settings = sqlx::query_as::<_, UserRoomSettings>(
            r#"
            INSERT INTO user_room_settings (user_id, room_id, is_muted, notification_preference,
                                            is_pinned, custom_name, custom_color)
            VALUES ($1, $2, COALESCE($3, false), COALESCE($4, 'all'), COALESCE($5, false), $6, $7)
            ON CONFLICT (user_id, room_id)
            DO UPDATE SET
                is_muted = COALESCE($3, user_room_settings.is_muted),
                notification_preference = COALESCE($4, user_room_settings.notification_preference),
                is_pinned = COALESCE($5, user_room_settings.is_pinned),
                custom_name = COALESCE($6, user_room_settings.custom_name),
                custom_color = COALESCE($7, user_room_settings.custom_color),
                updated_at = NOW()
            RETURNING id, user_id, room_id, is_muted, notification_preference,
                      is_pinned, custom_name, custom_color, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(room_id)
        .bind(request.is_muted)
        .bind(&request.notification_preference)
        .bind(request.is_pinned)
        .bind(&request.custom_name)
        .bind(&request.custom_color)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(settings.into())
    }

    /// 删除用户在某个房间的设置（重置为默认）
    pub async fn delete_room_settings(&self, user_id: Uuid, room_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_room_settings
            WHERE user_id = $1 AND room_id = $2
            "#,
        )
        .bind(user_id)
        .bind(room_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 批量获取已置顶的房间 ID 列表
    pub async fn get_pinned_room_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>> {
        let ids: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT room_id FROM user_room_settings
            WHERE user_id = $1 AND is_pinned = true
            ORDER BY updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(ids.into_iter().map(|r| r.0).collect())
    }

    /// 批量获取已静音的房间 ID 列表
    pub async fn get_muted_room_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>> {
        let ids: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT room_id FROM user_room_settings
            WHERE user_id = $1 AND is_muted = true
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(ids.into_iter().map(|r| r.0).collect())
    }
}

/// 数据库行结构（用于反序列化 JSONB 列）
#[derive(Debug, sqlx::FromRow)]
struct UserSettingsRow {
    notification_settings: Option<serde_json::Value>,
    privacy_settings: Option<serde_json::Value>,
    message_settings: Option<serde_json::Value>,
    language_settings: Option<serde_json::Value>,
    accessibility_settings: Option<serde_json::Value>,
    media_settings: Option<serde_json::Value>,
}
