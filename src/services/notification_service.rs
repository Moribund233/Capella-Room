use std::sync::Arc;

use chrono::{DateTime, Utc};
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    services::user_settings_service::UserSettingsService,
    websocket::{
        manager::WebSocketManager,
        protocol::{NotificationDbType, PendingActionInfo, PendingActionStatus, PendingActionType, WebSocketMessage},
    },
};

/// 私信信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PrivateMessageInfo {
    pub message_id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// @提及信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct MentionInfo {
    pub message_id: Uuid,
    pub room_id: Uuid,
    pub mentioned_by: Uuid,
    pub mentioned_by_name: String,
    pub content_preview: String,
    pub created_at: DateTime<Utc>,
}

/// 房间邀请信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoomInvitationInfo {
    pub invitation_id: Uuid,
    pub room_id: Uuid,
    pub room_name: String,
    pub invited_by: Uuid,
    pub invited_by_name: String,
    pub created_at: DateTime<Utc>,
}

/// 系统通知信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemNotificationInfo {
    pub notification_type: crate::websocket::protocol::NotificationType,
    pub title: String,
    pub content: String,
    pub data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// 文件信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    pub file_id: Uuid,
    pub file_name: String,
    pub file_url: String,
    pub file_size: u64,
    pub uploaded_at: DateTime<Utc>,
}

/// 通知服务
/// 管理各类通知的发送、存储和查询
#[derive(Debug)]
pub struct NotificationService {
    db: Database,
    ws_manager: Arc<WebSocketManager>,
    user_settings_service: Arc<UserSettingsService>,
}

impl NotificationService {
    /// 创建新的通知服务
    pub fn new(
        db: Database,
        ws_manager: Arc<WebSocketManager>,
        user_settings_service: Arc<UserSettingsService>,
    ) -> Self {
        Self {
            db,
            ws_manager,
            user_settings_service,
        }
    }

    /// 检查用户是否启用了特定类型的通知
    async fn is_notification_enabled(&self, user_id: Uuid, notification_type: &str) -> bool {
        match self
            .user_settings_service
            .is_notification_enabled(user_id, notification_type)
            .await
        {
            Ok(enabled) => enabled,
            Err(e) => {
                warn!(
                    "Failed to check notification settings for user {}: {}",
                    user_id, e
                );
                true // 默认启用通知（故障开放）
            }
        }
    }

    /// 发送私信通知
    ///
    /// # 参数
    /// - `receiver_id`: 接收者用户ID
    /// - `message`: 私信信息
    ///
    /// # 说明
    /// - 检查用户设置，如果用户关闭了私信通知，则不发送
    /// - 无论接收者是否在线，都先将通知存储到数据库（持久化）
    /// - 如果接收者在线，额外通过WebSocket实时推送
    /// - 这种"双写"模式确保通知不会丢失，同时保证实时性
    pub async fn send_private_message(
        &self,
        receiver_id: Uuid,
        message: PrivateMessageInfo,
    ) -> Result<()> {
        debug!(
            "Sending private message notification to user: {}",
            receiver_id
        );

        // 检查用户是否启用了私信通知
        if !self
            .is_notification_enabled(receiver_id, "private_message")
            .await
        {
            debug!(
                "User {} has disabled private message notifications, skipping",
                receiver_id
            );
            return Ok(());
        }

        // 1. 先存储到数据库（无论在线与否，确保持久化）
        let notification_id = self
            .store_notification(
                receiver_id,
                NotificationDbType::PrivateMessage,
                None,
                &format!("来自 {} 的私信", message.sender_name),
                &serde_json::to_value(&message).unwrap_or_default(),
            )
            .await?;

        debug!(
            "Private message notification stored to database, id: {}",
            notification_id
        );

        // 2. 如果用户在线，额外推送WebSocket（异步，失败不影响已存储的通知）
        if self.ws_manager.is_user_online(receiver_id) {
            let ws_message = WebSocketMessage::PrivateMessage {
                message_id: message.message_id,
                sender_id: message.sender_id,
                sender_name: message.sender_name.clone(),
                content: message.content.clone(),
                created_at: message.created_at,
            };

            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(receiver_id, json).await {
                    warn!(
                        "Failed to send private message WebSocket notification to user {}: {}. \
                         Notification is already persisted in database.",
                        receiver_id, e
                    );
                } else {
                    debug!(
                        "Private message WebSocket notification sent to online user: {}",
                        receiver_id
                    );
                }
            }
        }

        Ok(())
    }

    /// 发送@提及通知
    ///
    /// # 参数
    /// - `mentioned_user_id`: 被提及的用户ID
    /// - `mention_info`: @提及信息
    ///
    /// # 说明
    /// - 检查用户设置，如果用户关闭了@提及通知，则不发送
    /// - 无论用户是否在线，都先将通知存储到数据库（持久化）
    /// - 如果用户在线，额外通过WebSocket实时推送
    pub async fn send_mention(
        &self,
        mentioned_user_id: Uuid,
        mention_info: MentionInfo,
    ) -> Result<()> {
        debug!(
            "Sending mention notification to user: {}",
            mentioned_user_id
        );

        // 检查用户是否启用了@提及通知
        if !self
            .is_notification_enabled(mentioned_user_id, "mentioned")
            .await
        {
            debug!(
                "User {} has disabled mention notifications, skipping",
                mentioned_user_id
            );
            return Ok(());
        }

        // 1. 先存储到数据库（无论在线与否，确保持久化）
        let notification_id = self
            .store_notification(
                mentioned_user_id,
                NotificationDbType::Mentioned,
                None,
                &format!("{} 提到了你", mention_info.mentioned_by_name),
                &serde_json::to_value(&mention_info).unwrap_or_default(),
            )
            .await?;

        debug!(
            "Mention notification stored to database, id: {}",
            notification_id
        );

        // 2. 如果用户在线，额外推送WebSocket（异步，失败不影响已存储的通知）
        if self.ws_manager.is_user_online(mentioned_user_id) {
            let ws_message = WebSocketMessage::Mentioned {
                message_id: mention_info.message_id,
                room_id: mention_info.room_id,
                mentioned_by: mention_info.mentioned_by,
                mentioned_by_name: mention_info.mentioned_by_name.clone(),
                content_preview: mention_info.content_preview.clone(),
                created_at: mention_info.created_at,
            };

            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(mentioned_user_id, json).await {
                    warn!(
                        "Failed to send mention WebSocket notification to user {}: {}. \
                         Notification is already persisted in database.",
                        mentioned_user_id, e
                    );
                } else {
                    debug!(
                        "Mention WebSocket notification sent to online user: {}",
                        mentioned_user_id
                    );
                }
            }
        }

        Ok(())
    }

    /// 批量发送@提及通知
    ///
    /// # 参数
    /// - `mentioned_user_ids`: 被提及的用户ID列表
    /// - `mention_info`: @提及信息
    pub async fn send_mentions(
        &self,
        mentioned_user_ids: Vec<Uuid>,
        mention_info: MentionInfo,
    ) -> Result<()> {
        for user_id in mentioned_user_ids {
            let info = mention_info.clone();
            if let Err(e) = self.send_mention(user_id, info).await {
                error!("Failed to send mention to user {}: {}", user_id, e);
            }
        }
        Ok(())
    }

    /// 发送房间邀请通知
    ///
    /// # 参数
    /// - `invited_user_id`: 被邀请的用户ID
    /// - `invitation`: 邀请信息
    ///
    /// # 说明
    /// - 检查用户设置，如果用户关闭了房间邀请通知，则不发送
    /// - 无论用户是否在线，都先将通知存储到数据库（持久化）
    /// - 如果用户在线，额外通过WebSocket实时推送
    pub async fn send_room_invitation(
        &self,
        invited_user_id: Uuid,
        invitation: RoomInvitationInfo,
    ) -> Result<()> {
        debug!(
            "Sending room invitation notification to user: {}",
            invited_user_id
        );

        // 检查用户是否启用了房间邀请通知
        if !self
            .is_notification_enabled(invited_user_id, "room_invitation")
            .await
        {
            debug!(
                "User {} has disabled room invitation notifications, skipping",
                invited_user_id
            );
            return Ok(());
        }

        // 1. 先存储到数据库（无论在线与否，确保持久化）
        let notification_id = self
            .store_notification(
                invited_user_id,
                NotificationDbType::RoomInvitation,
                None,
                &format!(
                    "{} 邀请你加入 {}",
                    invitation.invited_by_name, invitation.room_name
                ),
                &serde_json::to_value(&invitation).unwrap_or_default(),
            )
            .await?;

        debug!(
            "Room invitation notification stored to database, id: {}",
            notification_id
        );

        // 2. 如果用户在线，额外推送WebSocket（异步，失败不影响已存储的通知）
        if self.ws_manager.is_user_online(invited_user_id) {
            let ws_message = WebSocketMessage::RoomInvitation {
                invitation_id: invitation.invitation_id,
                room_id: invitation.room_id,
                room_name: invitation.room_name.clone(),
                invited_by: invitation.invited_by,
                invited_by_name: invitation.invited_by_name.clone(),
                created_at: invitation.created_at,
            };

            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(invited_user_id, json).await {
                    warn!(
                        "Failed to send room invitation WebSocket notification to user {}: {}. \
                         Notification is already persisted in database.",
                        invited_user_id, e
                    );
                } else {
                    debug!(
                        "Room invitation WebSocket notification sent to online user: {}",
                        invited_user_id
                    );
                }
            }
        }

        Ok(())
    }

    /// 发送系统通知
    ///
    /// # 参数
    /// - `notification`: 系统通知信息
    /// - `target_users`: 目标用户列表，None表示广播给所有在线用户
    ///
    /// # 说明
    /// - 检查用户设置，如果用户关闭了系统通知，则不发送
    /// - 无论用户是否在线，都先将通知存储到数据库（持久化）
    /// - 如果用户在线，额外通过WebSocket实时推送
    pub async fn send_system_notification(
        &self,
        notification: SystemNotificationInfo,
        target_users: Option<Vec<Uuid>>,
    ) -> Result<()> {
        debug!("Sending system notification");

        let ws_message = WebSocketMessage::SystemNotification {
            notification_type: notification.notification_type.clone(),
            title: notification.title.clone(),
            content: notification.content.clone(),
            data: notification.data.clone(),
            created_at: notification.created_at,
        };

        match target_users {
            Some(user_ids) => {
                // 发送给指定用户
                for user_id in user_ids {
                    // 检查用户是否启用了系统通知
                    if !self
                        .is_notification_enabled(user_id, "system_notification")
                        .await
                    {
                        debug!(
                            "User {} has disabled system notifications, skipping",
                            user_id
                        );
                        continue;
                    }

                    // 1. 先存储到数据库（无论在线与否，确保持久化）
                    let notification_id = self
                        .store_notification(
                            user_id,
                            NotificationDbType::SystemNotification,
                            Some(&notification.title),
                            &notification.content,
                            &serde_json::to_value(&notification).unwrap_or_default(),
                        )
                        .await?;

                    debug!(
                        "System notification stored to database for user {}, id: {}",
                        user_id, notification_id
                    );

                    // 2. 如果用户在线，额外推送WebSocket（异步，失败不影响已存储的通知）
                    if self.ws_manager.is_user_online(user_id) {
                        if let Ok(json) = ws_message.to_json() {
                            if let Err(e) = self.ws_manager.send_to_user(user_id, json).await {
                                warn!(
                                    "Failed to send system WebSocket notification to user {}: {}. \
                                     Notification is already persisted in database.",
                                    user_id, e
                                );
                            } else {
                                debug!(
                                    "System WebSocket notification sent to online user: {}",
                                    user_id
                                );
                            }
                        }
                    }
                }
            }
            None => {
                // 广播给所有在线用户
                // 注意：广播模式下无法存储到数据库（不知道目标用户列表）
                // 如果需要持久化广播通知，需要传入目标用户列表
                warn!(
                    "Broadcasting system notification to all users is not fully implemented. \
                       To persist notifications, provide target user list instead of None."
                );
            }
        }

        Ok(())
    }

    /// 发送文件上传完成通知
    ///
    /// # 参数
    /// - `user_id`: 上传文件的用户ID
    /// - `file_info`: 文件信息
    ///
    /// # 说明
    /// - 检查用户设置，如果用户关闭了文件上传通知，则不发送
    /// - 无论用户是否在线，都先将通知存储到数据库（持久化）
    /// - 如果用户在线，额外通过WebSocket实时推送
    pub async fn send_file_upload_complete(
        &self,
        user_id: Uuid,
        file_info: FileInfo,
    ) -> Result<()> {
        debug!(
            "Sending file upload complete notification to user: {}",
            user_id
        );

        // 检查用户是否启用了文件上传完成通知
        if !self
            .is_notification_enabled(user_id, "file_upload_complete")
            .await
        {
            debug!(
                "User {} has disabled file upload notifications, skipping",
                user_id
            );
            return Ok(());
        }

        // 1. 先存储到数据库（无论在线与否，确保持久化）
        let notification_id = self
            .store_notification(
                user_id,
                NotificationDbType::FileUploadComplete,
                None,
                &format!("文件 {} 上传完成", file_info.file_name),
                &serde_json::to_value(&file_info).unwrap_or_default(),
            )
            .await?;

        debug!(
            "File upload complete notification stored to database, id: {}",
            notification_id
        );

        // 2. 如果用户在线，额外推送WebSocket（异步，失败不影响已存储的通知）
        if self.ws_manager.is_user_online(user_id) {
            let ws_message = WebSocketMessage::FileUploadComplete {
                file_id: file_info.file_id,
                file_name: file_info.file_name.clone(),
                file_url: file_info.file_url.clone(),
                file_size: file_info.file_size,
                uploaded_at: file_info.uploaded_at,
            };

            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(user_id, json).await {
                    warn!(
                        "Failed to send file upload WebSocket notification to user {}: {}. \
                         Notification is already persisted in database.",
                        user_id, e
                    );
                } else {
                    debug!(
                        "File upload WebSocket notification sent to online user: {}",
                        user_id
                    );
                }
            }
        }

        Ok(())
    }

    /// 存储通知到数据库（统一存储方法）
    ///
    /// # 参数
    /// - `user_id`: 接收通知的用户ID
    /// - `notification_type`: 通知类型
    /// - `title`: 通知标题（可选）
    /// - `content`: 通知内容
    /// - `data`: 附加数据（JSON格式）
    ///
    /// # 返回
    /// - 成功返回通知ID
    async fn store_notification(
        &self,
        user_id: Uuid,
        notification_type: NotificationDbType,
        title: Option<&str>,
        content: &str,
        data: &serde_json::Value,
    ) -> Result<Uuid> {
        let result = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO notifications (user_id, notification_type, title, content, data)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(notification_type)
        .bind(title)
        .bind(content)
        .bind(data)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| {
            error!("Failed to store notification: {}", e);
            AppError::Database(e)
        })?;

        Ok(result)
    }

    /// 获取用户的未读通知
    ///
    /// # 参数
    /// - `user_id`: 用户ID
    /// - `limit`: 返回的最大通知数量
    pub async fn get_unread_notifications(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<crate::websocket::protocol::Notification>> {
        let notifications: Vec<crate::websocket::protocol::Notification> = sqlx::query_as(
            r#"
            SELECT id, notification_type, title, content, data, is_read, read_at, created_at
            FROM notifications
            WHERE user_id = $1 AND is_read = false
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        Ok(notifications)
    }

    /// 获取用户的所有通知（支持分页）
    pub async fn get_notifications(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::websocket::protocol::Notification>> {
        let notifications: Vec<crate::websocket::protocol::Notification> = sqlx::query_as(
            r#"
            SELECT id, notification_type, title, content, data, is_read, read_at, created_at
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        Ok(notifications)
    }

    /// 标记通知为已读
    pub async fn mark_as_read(&self, user_id: Uuid, notification_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = true, read_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 标记所有通知为已读
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = true, read_at = NOW()
            WHERE user_id = $1 AND is_read = false
            "#,
        )
        .bind(user_id)
        .execute(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 获取未读通知数量
    pub async fn get_unread_count(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM notifications
            WHERE user_id = $1 AND is_read = false
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        Ok(count.0)
    }

    /// 删除过期通知
    ///
    /// # 参数
    /// - `days`: 删除多少天前的通知
    pub async fn delete_expired_notifications(&self, days: i32) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE created_at < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(days)
        .execute(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    // ==================== 待办通知系统 ====================

    /// 发送待办通知（需要管理员确认的配置变更）
    ///
    /// # 参数
    /// - `admin_user_id`: 管理员用户ID
    /// - `action_info`: 待办通知信息
    ///
    /// # 说明
    /// - 如果管理员在线，通过WebSocket实时推送
    /// - 如果管理员离线，存储到数据库待上线后同步
    pub async fn send_pending_action(
        &self,
        admin_user_id: Uuid,
        action_info: PendingActionInfo,
    ) -> Result<()> {
        debug!(
            "Sending pending action notification to admin: {}",
            admin_user_id
        );

        // 1. 先存储到数据库（标记为需要操作）
        let notification_id = self
            .store_pending_action(admin_user_id, &action_info)
            .await?;

        // 2. 如果管理员在线，WebSocket实时推送
        let ws_message = WebSocketMessage::PendingAction {
            notification_id,
            action_type: action_info.action_type.clone(),
            title: action_info.title.clone(),
            description: action_info.description.clone(),
            deadline: action_info.deadline,
            data: serde_json::to_value(&action_info).ok(),
            created_at: action_info.created_at,
        };

        if self.ws_manager.is_user_online(admin_user_id) {
            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(admin_user_id, json).await {
                    warn!(
                        "Failed to send pending action WebSocket notification to admin {}: {}. \
                         Notification is already persisted in database.",
                        admin_user_id, e
                    );
                } else {
                    debug!(
                        "Pending action WebSocket notification sent to online admin: {}",
                        admin_user_id
                    );
                }
            }
        }

        Ok(())
    }

    /// 存储待办通知到数据库
    async fn store_pending_action(
        &self,
        admin_user_id: Uuid,
        action_info: &PendingActionInfo,
    ) -> Result<Uuid> {
        let record: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO notifications (
                user_id, notification_type, title, content, data,
                requires_action, action_type, action_status, action_deadline
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(admin_user_id)
        .bind(NotificationDbType::PendingAction)
        .bind(&action_info.title)
        .bind(&action_info.description)
        .bind(serde_json::json!({
            "related_config_key": action_info.related_config_key,
            "related_config_value": action_info.related_config_value,
        }))
        .bind(true) // requires_action
        .bind(&action_info.action_type)
        .bind(PendingActionStatus::Pending)
        .bind(action_info.deadline)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| {
            error!("Failed to store pending action: {}", e);
            AppError::Database(e)
        })?;

        Ok(record.0)
    }

    /// 获取管理员的待办通知列表
    pub async fn get_pending_actions(
        &self,
        admin_user_id: Uuid,
    ) -> Result<Vec<crate::websocket::protocol::PendingActionInfo>> {
        let actions: Vec<crate::websocket::protocol::PendingActionInfo> = sqlx::query_as(
            r#"
            SELECT 
                id as notification_id, 
                action_type, 
                title, 
                content as description, 
                action_deadline as deadline, 
                data, 
                action_status as action_status, 
                data->>'related_config_key' as related_config_key,
                data->>'related_config_value' as related_config_value,
                created_at
            FROM notifications
            WHERE user_id = $1 
                AND requires_action = true 
                AND action_status = 'pending'
            ORDER BY created_at DESC
            "#,
        )
        .bind(admin_user_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        Ok(actions)
    }

    /// 处理待办通知
    pub async fn process_pending_action(
        &self,
        admin_user_id: Uuid,
        action_id: Uuid,
        action: PendingActionType,
        comment: Option<String>,
    ) -> Result<()> {
        let new_status = match action {
            PendingActionType::Approve => PendingActionStatus::Approved,
            PendingActionType::Reject => PendingActionStatus::Rejected,
            PendingActionType::Snooze => PendingActionStatus::Snoozed,
        };

        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET action_status = $1, 
                is_read = true, 
                read_at = NOW(),
                data = jsonb_set(
                    jsonb_set(COALESCE(data, '{}'::jsonb), '{processed_by}', to_jsonb($2::text)),
                    '{comment}', 
                    COALESCE(to_jsonb($3::text), 'null'::jsonb)
                )
            WHERE id = $4 AND user_id = $2 AND requires_action = true AND action_status = 'pending'
            "#,
        )
        .bind(new_status)
        .bind(admin_user_id)
        .bind(comment)
        .bind(action_id)
        .execute(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }
}
