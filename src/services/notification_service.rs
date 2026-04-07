use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::json;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    websocket::{
        manager::WebSocketManager,
        protocol::{
            NotificationDbType, PendingActionInfo, PendingActionStatus, PendingActionType,
            WebSocketMessage,
        },
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
}

impl NotificationService {
    /// 创建新的通知服务
    pub fn new(db: Database, ws_manager: Arc<WebSocketManager>) -> Self {
        Self { db, ws_manager }
    }

    /// 发送私信通知
    ///
    /// # 参数
    /// - `receiver_id`: 接收者用户ID
    /// - `message`: 私信信息
    ///
    /// # 说明
    /// - 如果接收者在线，通过WebSocket实时推送
    /// - 如果接收者离线，存储到数据库待上线后同步
    pub async fn send_private_message(
        &self,
        receiver_id: Uuid,
        message: PrivateMessageInfo,
    ) -> Result<()> {
        debug!(
            "Sending private message notification to user: {}",
            receiver_id
        );

        let ws_message = WebSocketMessage::PrivateMessage {
            message_id: message.message_id,
            sender_id: message.sender_id,
            sender_name: message.sender_name.clone(),
            content: message.content.clone(),
            created_at: message.created_at,
        };

        // 检查接收者是否在线
        if self.ws_manager.is_user_online(receiver_id) {
            // 在线：通过WebSocket实时推送
            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(receiver_id, json).await {
                    warn!(
                        "Failed to send private message notification to online user {}: {}",
                        receiver_id, e
                    );
                    // 发送失败时存储到数据库
                    self.store_offline_notification(
                        receiver_id,
                        NotificationDbType::PrivateMessage,
                        None,
                        &format!("来自 {} 的私信", message.sender_name),
                        &serde_json::to_value(&message).unwrap_or_default(),
                    )
                    .await?;
                } else {
                    debug!(
                        "Private message notification sent to online user: {}",
                        receiver_id
                    );
                }
            }
        } else {
            // 离线：存储到数据库
            debug!(
                "User {} is offline, storing private message notification",
                receiver_id
            );
            self.store_offline_notification(
                receiver_id,
                NotificationDbType::PrivateMessage,
                None,
                &format!("来自 {} 的私信", message.sender_name),
                &serde_json::to_value(&message).unwrap_or_default(),
            )
            .await?;
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
    /// - 如果用户在线，通过WebSocket实时推送
    /// - 如果用户离线，存储到数据库
    pub async fn send_mention(
        &self,
        mentioned_user_id: Uuid,
        mention_info: MentionInfo,
    ) -> Result<()> {
        debug!(
            "Sending mention notification to user: {}",
            mentioned_user_id
        );

        let ws_message = WebSocketMessage::Mentioned {
            message_id: mention_info.message_id,
            room_id: mention_info.room_id,
            mentioned_by: mention_info.mentioned_by,
            mentioned_by_name: mention_info.mentioned_by_name.clone(),
            content_preview: mention_info.content_preview.clone(),
            created_at: mention_info.created_at,
        };

        // 检查用户是否在线
        if self.ws_manager.is_user_online(mentioned_user_id) {
            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(mentioned_user_id, json).await {
                    warn!(
                        "Failed to send mention notification to online user {}: {}",
                        mentioned_user_id, e
                    );
                    self.store_offline_notification(
                        mentioned_user_id,
                        NotificationDbType::Mentioned,
                        None,
                        &format!("{} 提到了你", mention_info.mentioned_by_name),
                        &serde_json::to_value(&mention_info).unwrap_or_default(),
                    )
                    .await?;
                }
            }
        } else {
            self.store_offline_notification(
                mentioned_user_id,
                NotificationDbType::Mentioned,
                None,
                &format!("{} 提到了你", mention_info.mentioned_by_name),
                &serde_json::to_value(&mention_info).unwrap_or_default(),
            )
            .await?;
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
    pub async fn send_room_invitation(
        &self,
        invited_user_id: Uuid,
        invitation: RoomInvitationInfo,
    ) -> Result<()> {
        debug!(
            "Sending room invitation notification to user: {}",
            invited_user_id
        );

        let ws_message = WebSocketMessage::RoomInvitation {
            invitation_id: invitation.invitation_id,
            room_id: invitation.room_id,
            room_name: invitation.room_name.clone(),
            invited_by: invitation.invited_by,
            invited_by_name: invitation.invited_by_name.clone(),
            created_at: invitation.created_at,
        };

        if self.ws_manager.is_user_online(invited_user_id) {
            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(invited_user_id, json).await {
                    warn!(
                        "Failed to send room invitation to online user {}: {}",
                        invited_user_id, e
                    );
                    self.store_offline_notification(
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
                }
            }
        } else {
            self.store_offline_notification(
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
        }

        Ok(())
    }

    /// 发送系统通知
    ///
    /// # 参数
    /// - `notification`: 系统通知信息
    /// - `target_users`: 目标用户列表，None表示广播给所有在线用户
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
                    if self.ws_manager.is_user_online(user_id) {
                        if let Ok(json) = ws_message.to_json() {
                            if let Err(e) = self.ws_manager.send_to_user(user_id, json).await {
                                warn!(
                                    "Failed to send system notification to user {}: {}",
                                    user_id, e
                                );
                            }
                        }
                    } else {
                        // 离线用户存储到数据库
                        self.store_offline_notification(
                            user_id,
                            NotificationDbType::SystemNotification,
                            Some(&notification.title),
                            &notification.content,
                            &serde_json::to_value(&notification).unwrap_or_default(),
                        )
                        .await?;
                    }
                }
            }
            None => {
                // 广播给所有在线用户
                // 注意：这里简化处理，实际应用可能需要更高效的广播机制
                // 目前只广播给已知的在线用户，无法遍历所有连接
                warn!("Broadcasting system notification to all users is not fully implemented");
            }
        }

        Ok(())
    }

    /// 发送文件上传完成通知
    ///
    /// # 参数
    /// - `user_id`: 上传文件的用户ID
    /// - `file_info`: 文件信息
    pub async fn send_file_upload_complete(
        &self,
        user_id: Uuid,
        file_info: FileInfo,
    ) -> Result<()> {
        debug!(
            "Sending file upload complete notification to user: {}",
            user_id
        );

        let ws_message = WebSocketMessage::FileUploadComplete {
            file_id: file_info.file_id,
            file_name: file_info.file_name.clone(),
            file_url: file_info.file_url.clone(),
            file_size: file_info.file_size,
            uploaded_at: file_info.uploaded_at,
        };

        if self.ws_manager.is_user_online(user_id) {
            if let Ok(json) = ws_message.to_json() {
                if let Err(e) = self.ws_manager.send_to_user(user_id, json).await {
                    warn!(
                        "Failed to send file upload notification to online user {}: {}",
                        user_id, e
                    );
                    self.store_offline_notification(
                        user_id,
                        NotificationDbType::FileUploadComplete,
                        None,
                        &format!("文件 {} 上传完成", file_info.file_name),
                        &serde_json::to_value(&file_info).unwrap_or_default(),
                    )
                    .await?;
                }
            }
        } else {
            self.store_offline_notification(
                user_id,
                NotificationDbType::FileUploadComplete,
                None,
                &format!("文件 {} 上传完成", file_info.file_name),
                &serde_json::to_value(&file_info).unwrap_or_default(),
            )
            .await?;
        }

        Ok(())
    }

    /// 存储离线通知到数据库
    async fn store_offline_notification(
        &self,
        user_id: Uuid,
        notification_type: NotificationDbType,
        title: Option<&str>,
        content: &str,
        data: &serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, notification_type, title, content, data)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user_id)
        .bind(notification_type)
        .bind(title)
        .bind(content)
        .bind(data)
        .execute(self.db.pool())
        .await
        .map_err(|e| {
            error!("Failed to store offline notification: {}", e);
            AppError::Database(e)
        })?;

        Ok(())
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
                        "Failed to send pending action to online admin {}: {}",
                        admin_user_id, e
                    );
                } else {
                    debug!(
                        "Pending action notification sent to online admin: {}",
                        admin_user_id
                    );
                }
            }
        } else {
            debug!(
                "Admin {} is offline, pending action stored for later sync",
                admin_user_id
            );
        }

        Ok(())
    }

    /// 存储待办通知到数据库
    async fn store_pending_action(
        &self,
        user_id: Uuid,
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
        .bind(user_id)
        .bind(NotificationDbType::PendingAction)
        .bind(&action_info.title)
        .bind(&action_info.description)
        .bind(json!({
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

    /// 处理待办响应
    ///
    /// # 参数
    /// - `admin_user_id`: 管理员用户ID
    /// - `notification_id`: 通知ID
    /// - `action`: 操作类型（Approve/Reject/Snooze）
    /// - `comment`: 可选备注
    pub async fn handle_pending_action_response(
        &self,
        admin_user_id: Uuid,
        notification_id: Uuid,
        action: PendingActionType,
        comment: Option<String>,
    ) -> Result<PendingActionStatus> {
        // 1. 验证通知是否存在且属于该管理员
        let _notification: crate::websocket::protocol::Notification = sqlx::query_as(
            r#"
            SELECT id, notification_type, title, content, data, is_read, read_at, created_at
            FROM notifications
            WHERE id = $1 AND user_id = $2 AND requires_action = true
            "#,
        )
        .bind(notification_id)
        .bind(admin_user_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

        // 2. 根据操作类型更新状态
        let new_status = match action {
            PendingActionType::Approve => PendingActionStatus::Approved,
            PendingActionType::Reject => PendingActionStatus::Rejected,
            PendingActionType::Snooze => PendingActionStatus::Snoozed,
        };

        // 3. 更新通知状态
        sqlx::query(
            r#"
            UPDATE notifications
            SET action_status = $1,
                action_by = $2,
                action_at = NOW(),
                action_result = $3,
                is_read = true,
                read_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(&new_status)
        .bind(admin_user_id)
        .bind(json!({
            "action": action.to_string(),
            "comment": comment,
        }))
        .bind(notification_id)
        .execute(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        info!(
            "Pending action {} handled by admin {}: {:?}",
            notification_id, admin_user_id, action
        );

        // 4. 发送确认响应给管理员
        let response_message = WebSocketMessage::PendingActionResponse {
            notification_id,
            success: true,
            message: format!("Action {:?} processed successfully", action),
            new_status: format!("{:?}", new_status).to_lowercase(),
        };

        if let Ok(json) = response_message.to_json() {
            let _ = self.ws_manager.send_to_user(admin_user_id, json).await;
        }

        Ok(new_status)
    }

    /// 获取管理员的待办列表
    ///
    /// # 参数
    /// - `admin_user_id`: 管理员用户ID
    /// - `action_type`: 可选的操作类型过滤
    pub async fn get_pending_actions(
        &self,
        admin_user_id: Uuid,
        action_type: Option<String>,
    ) -> Result<Vec<PendingActionInfo>> {
        let actions: Vec<PendingActionInfo> = if let Some(ref action_type_filter) = action_type {
            sqlx::query_as(
                r#"
                SELECT 
                    id as notification_id,
                    action_type,
                    title,
                    content as description,
                    action_deadline as deadline,
                    action_status,
                    data->>'related_config_key' as related_config_key,
                    data->>'related_config_value' as related_config_value,
                    created_at
                FROM notifications
                WHERE user_id = $1 
                    AND requires_action = true 
                    AND action_status = 'pending'
                    AND action_type = $2
                ORDER BY action_deadline ASC NULLS LAST, created_at DESC
                "#,
            )
            .bind(admin_user_id)
            .bind(action_type_filter)
            .fetch_all(self.db.pool())
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_as(
                r#"
                SELECT 
                    id as notification_id,
                    action_type,
                    title,
                    content as description,
                    action_deadline as deadline,
                    action_status,
                    data->>'related_config_key' as related_config_key,
                    data->>'related_config_value' as related_config_value,
                    created_at
                FROM notifications
                WHERE user_id = $1 
                    AND requires_action = true 
                    AND action_status = 'pending'
                ORDER BY action_deadline ASC NULLS LAST, created_at DESC
                "#,
            )
            .bind(admin_user_id)
            .fetch_all(self.db.pool())
            .await
            .map_err(AppError::Database)?
        };

        Ok(actions)
    }

    /// 获取待办数量
    pub async fn get_pending_action_count(&self, admin_user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM notifications
            WHERE user_id = $1 
                AND requires_action = true 
                AND action_status = 'pending'
            "#,
        )
        .bind(admin_user_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(AppError::Database)?;

        Ok(count.0)
    }

    /// 创建配置重载待办通知（便捷方法）
    ///
    /// # 参数
    /// - `admin_user_id`: 管理员用户ID
    /// - `config_key`: 配置项键名
    /// - `config_value`: 新的配置值
    /// - `requires_restart`: 是否需要重启生效
    pub async fn send_config_reload_notification(
        &self,
        admin_user_id: Uuid,
        config_key: &str,
        config_value: &str,
        requires_restart: bool,
    ) -> Result<()> {
        let title = format!("配置变更需要确认: {}", config_key);
        let description = if requires_restart {
            format!(
                "配置项 '{}' 已修改为 '{}'，需要重启服务才能生效。请确认是否执行重启操作。",
                config_key, config_value
            )
        } else {
            format!(
                "配置项 '{}' 已修改为 '{}'，将在下次清理任务时自动生效。",
                config_key, config_value
            )
        };

        let action_info = PendingActionInfo {
            notification_id: Uuid::new_v4(), // 临时ID，实际由数据库生成
            action_type: "config_reload".to_string(),
            title,
            description,
            deadline: Some(Utc::now() + chrono::Duration::hours(24)), // 24小时截止
            action_status: PendingActionStatus::Pending,
            related_config_key: Some(config_key.to_string()),
            related_config_value: Some(config_value.to_string()),
            created_at: Utc::now(),
        };

        self.send_pending_action(admin_user_id, action_info).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_message_info() {
        let info = PrivateMessageInfo {
            message_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            sender_name: "test_user".to_string(),
            content: "Hello!".to_string(),
            created_at: Utc::now(),
        };
        assert_eq!(info.sender_name, "test_user");
        assert_eq!(info.content, "Hello!");
    }

    #[test]
    fn test_mention_info() {
        let info = MentionInfo {
            message_id: Uuid::new_v4(),
            room_id: Uuid::new_v4(),
            mentioned_by: Uuid::new_v4(),
            mentioned_by_name: "test_user".to_string(),
            content_preview: "Hello @user".to_string(),
            created_at: Utc::now(),
        };
        assert_eq!(info.mentioned_by_name, "test_user");
        assert_eq!(info.content_preview, "Hello @user");
    }

    #[test]
    fn test_room_invitation_info() {
        let info = RoomInvitationInfo {
            invitation_id: Uuid::new_v4(),
            room_id: Uuid::new_v4(),
            room_name: "Test Room".to_string(),
            invited_by: Uuid::new_v4(),
            invited_by_name: "admin".to_string(),
            created_at: Utc::now(),
        };
        assert_eq!(info.room_name, "Test Room");
        assert_eq!(info.invited_by_name, "admin");
    }
}
