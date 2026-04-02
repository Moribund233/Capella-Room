use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    models::message::{Message, MessageResponse, MessageType, SenderInfo},
};

/// 消息服务
pub struct MessageService {
    db: Database,
}

impl MessageService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 创建消息
    pub async fn create_message(
        &self,
        room_id: Uuid,
        sender_id: Uuid,
        content: &str,
        reply_to: Option<Uuid>,
    ) -> Result<Message> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (room_id, sender_id, content, message_type, reply_to)
            VALUES ($1, $2, $3, 'text', $4)
            RETURNING *
            "#,
        )
        .bind(room_id)
        .bind(sender_id)
        .bind(content)
        .bind(reply_to)
        .fetch_one(self.db.pool())
        .await?;

        Ok(message)
    }

    /// 获取聊天室消息历史
    pub async fn get_room_messages(
        &self,
        room_id: Uuid,
        limit: i64,
        before: Option<Uuid>,
    ) -> Result<Vec<MessageResponse>> {
        let messages = if let Some(before_id) = before {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE room_id = $1 AND id < $2 AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(room_id)
            .bind(before_id)
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?
        } else {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE room_id = $1 AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(room_id)
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?
        };

        // 获取发送者信息并转换为响应
        let mut responses = Vec::new();
        for msg in messages {
            let sender = self.get_sender_info(msg.sender_id).await?;
            responses.push(msg.to_response(sender));
        }

        Ok(responses)
    }

    /// 搜索消息
    pub async fn search_messages(
        &self,
        room_id: Option<Uuid>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<MessageResponse>> {
        let messages = if let Some(rid) = room_id {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE room_id = $1 AND content ILIKE $2 AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(rid)
            .bind(format!("%{}%", query))
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?
        } else {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE content ILIKE $1 AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(format!("%{}%", query))
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?
        };

        let mut responses = Vec::new();
        for msg in messages {
            let sender = self.get_sender_info(msg.sender_id).await?;
            responses.push(msg.to_response(sender));
        }

        Ok(responses)
    }

    /// 删除消息（软删除）
    pub async fn delete_message(&self, message_id: Uuid, user_id: Uuid) -> Result<()> {
        // 检查消息是否存在且属于该用户
        let message: Option<Message> = sqlx::query_as(
            r#"
            SELECT * FROM messages WHERE id = $1 AND sender_id = $2 AND is_deleted = false
            "#,
        )
        .bind(message_id)
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        if message.is_none() {
            return Err(AppError::NotFound);
        }

        // 软删除
        sqlx::query(
            r#"
            UPDATE messages SET is_deleted = true WHERE id = $1
            "#,
        )
        .bind(message_id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 获取最新消息
    pub async fn get_latest_messages(&self, room_id: Uuid, limit: i64) -> Result<Vec<MessageResponse>> {
        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages 
            WHERE room_id = $1 AND is_deleted = false
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(room_id)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await?;

        let mut responses = Vec::new();
        for msg in messages {
            let sender = self.get_sender_info(msg.sender_id).await?;
            responses.push(msg.to_response(sender));
        }

        Ok(responses)
    }

    /// 获取发送者信息
    async fn get_sender_info(&self, user_id: Uuid) -> Result<SenderInfo> {
        let row: (String, Option<String>) = sqlx::query_as(
            r#"
            SELECT username, avatar_url FROM users WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await?;

        Ok(SenderInfo {
            id: user_id,
            username: row.0,
            avatar_url: row.1,
        })
    }

    /// 通过ID获取消息
    pub async fn get_message_by_id(&self, message_id: Uuid) -> Result<Option<Message>> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages WHERE id = $1
            "#,
        )
        .bind(message_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(message)
    }

    /// 获取离线消息（断线重连后获取错过的消息）
    pub async fn get_missed_messages(
        &self,
        room_id: Uuid,
        last_message_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<MessageResponse>> {
        let messages = if let Some(last_id) = last_message_id {
            // 获取指定消息之后的新消息
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE room_id = $1 AND id > $2 AND is_deleted = false
                ORDER BY created_at ASC
                LIMIT $3
                "#,
            )
            .bind(room_id)
            .bind(last_id)
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?
        } else {
            // 如果没有指定最后消息ID，获取最新的消息
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE room_id = $1 AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(room_id)
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?
        };

        // 获取发送者信息并转换为响应
        let mut responses = Vec::new();
        for msg in messages {
            let sender = self.get_sender_info(msg.sender_id).await?;
            responses.push(msg.to_response(sender));
        }

        Ok(responses)
    }
}
