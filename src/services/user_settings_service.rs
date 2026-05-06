use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::user_settings::{
    AccessibilitySettings, LanguageSettings, MediaSettings, MessageSettings, NotificationSettings,
    PrivacySettings, RoomNotificationPreference, UpdateRoomSettingsRequest,
    UpdateUserSettingsRequest, UserRoomSettings, UserRoomSettingsResponse, UserSettingsResponse,
};

/// 用户设置服务
#[derive(Debug, Clone)]
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
        // 先获取当前设置（用于合并）
        let current = self.get_user_settings(user_id).await?;

        // 构建更新后的完整设置
        let notification = request.notification.unwrap_or(current.notification);
        let privacy = request.privacy.unwrap_or(current.privacy);
        let message = request.message.unwrap_or(current.message);
        let language = request.language.unwrap_or(current.language);
        let accessibility = request.accessibility.unwrap_or(current.accessibility);
        let media = request.media.unwrap_or(current.media);

        // 使用 UPSERT 保存完整设置
        sqlx::query(
            r#"
            INSERT INTO user_settings (
                user_id,
                notification_settings,
                privacy_settings,
                message_settings,
                language_settings,
                accessibility_settings,
                media_settings
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id)
            DO UPDATE SET
                notification_settings = EXCLUDED.notification_settings,
                privacy_settings = EXCLUDED.privacy_settings,
                message_settings = EXCLUDED.message_settings,
                language_settings = EXCLUDED.language_settings,
                accessibility_settings = EXCLUDED.accessibility_settings,
                media_settings = EXCLUDED.media_settings,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(serde_json::to_value(&notification).unwrap_or_default())
        .bind(serde_json::to_value(&privacy).unwrap_or_default())
        .bind(serde_json::to_value(&message).unwrap_or_default())
        .bind(serde_json::to_value(&language).unwrap_or_default())
        .bind(serde_json::to_value(&accessibility).unwrap_or_default())
        .bind(serde_json::to_value(&media).unwrap_or_default())
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        // 返回更新后的设置
        Ok(UserSettingsResponse {
            notification,
            privacy,
            message,
            language,
            accessibility,
            media,
        })
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
                notification_preference: RoomNotificationPreference::default().as_str().to_string(),
                is_pinned: false,
                custom_name: None,
                custom_color: None,
            }),
        }
    }

    /// 获取用户在所有房间的设置列表
    pub async fn list_room_settings(&self, user_id: Uuid) -> Result<Vec<UserRoomSettingsResponse>> {
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
        // 将枚举转换为字符串存储
        let notification_pref_str = request
            .notification_preference
            .as_ref()
            .map(|p| p.as_str().to_string());

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
        .bind(notification_pref_str)
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

    /// 检查用户是否将某个房间设置为静音
    pub async fn is_room_muted(&self, user_id: Uuid, room_id: Uuid) -> Result<bool> {
        let result: Option<(bool,)> = sqlx::query_as(
            r#"
            SELECT is_muted FROM user_room_settings
            WHERE user_id = $1 AND room_id = $2
            "#,
        )
        .bind(user_id)
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.map(|r| r.0).unwrap_or(false))
    }

    /// 获取用户的通知偏好设置
    pub async fn get_notification_settings(&self, user_id: Uuid) -> Result<NotificationSettings> {
        let settings = self.get_user_settings(user_id).await?;
        Ok(settings.notification)
    }

    /// 检查用户是否启用了特定类型的通知
    pub async fn is_notification_enabled(
        &self,
        user_id: Uuid,
        notification_type: &str,
    ) -> Result<bool> {
        let settings = self.get_notification_settings(user_id).await?;

        let enabled = match notification_type {
            "private_message" => settings.private_message,
            "mentioned" => settings.mentioned,
            "room_invitation" => settings.room_invitation,
            "system_notification" => settings.system_notification,
            "file_upload_complete" => settings.file_upload_complete,
            _ => true, // 默认启用未知类型
        };

        Ok(enabled && !settings.do_not_disturb)
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
