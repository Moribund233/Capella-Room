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
        message_type: MessageType,
        reply_to: Option<Uuid>,
    ) -> Result<Message> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (room_id, sender_id, content, message_type, reply_to)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(room_id)
        .bind(sender_id)
        .bind(content)
        .bind(message_type)
        .bind(reply_to)
        .fetch_one(self.db.pool())
        .await?;

        Ok(message)
    }

    /// 创建文本消息（便捷方法）
    pub async fn create_text_message(
        &self,
        room_id: Uuid,
        sender_id: Uuid,
        content: &str,
        reply_to: Option<Uuid>,
    ) -> Result<Message> {
        self.create_message(room_id, sender_id, content, MessageType::Text, reply_to)
            .await
    }

    /// 获取聊天室消息历史
    pub async fn get_room_messages(
        &self,
        room_id: Uuid,
        limit: i64,
        before: Option<Uuid>,
    ) -> Result<Vec<MessageResponse>> {
        let messages = if let Some(before_id) = before {
            // 使用 created_at 进行游标分页，而不是 UUID 比较
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE room_id = $1 AND is_deleted = false
                AND created_at < (SELECT created_at FROM messages WHERE id = $2)
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
        // 检查消息是否存在
        let message: Option<Message> = sqlx::query_as(
            r#"
            SELECT * FROM messages WHERE id = $1 AND is_deleted = false
            "#,
        )
        .bind(message_id)
        .fetch_optional(self.db.pool())
        .await?;

        let message = message.ok_or(AppError::NotFound)?;

        // 检查权限：只有消息发送者才能删除
        if message.sender_id != user_id {
            return Err(AppError::Forbidden);
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
    pub async fn get_sender_info(&self, user_id: Uuid) -> Result<SenderInfo> {
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
            // 获取指定消息之后的新消息（使用 created_at 比较）
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE room_id = $1 AND is_deleted = false
                AND created_at > (SELECT created_at FROM messages WHERE id = $2)
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
            // 如果没有指定最后消息 ID，获取最新的消息
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

    /// 编辑消息
    /// 只有消息发送者才能编辑自己的消息
    pub async fn edit_message(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        new_content: &str,
    ) -> Result<Message> {
        // 检查消息是否存在
        let message: Option<Message> = sqlx::query_as(
            r#"
            SELECT * FROM messages WHERE id = $1 AND is_deleted = false
            "#,
        )
        .bind(message_id)
        .fetch_optional(self.db.pool())
        .await?;

        let message = message.ok_or(AppError::NotFound)?;

        // 检查是否是系统消息（系统消息不能编辑）
        if matches!(message.message_type, crate::models::message::MessageType::System) {
            return Err(AppError::Forbidden);
        }

        // 检查权限：只有消息发送者才能编辑
        if message.sender_id != user_id {
            return Err(AppError::Forbidden);
        }

        // 开始事务
        let mut tx = self.db.pool().begin().await?;

        // 记录编辑历史
        sqlx::query(
            r#"
            INSERT INTO message_edits (message_id, editor_id, old_content, new_content)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(message_id)
        .bind(user_id)
        .bind(&message.content)
        .bind(new_content)
        .execute(&mut *tx)
        .await?;

        // 更新消息内容
        let updated_message = sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages
            SET 
                content = $1,
                edit_count = edit_count + 1,
                edited_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(new_content)
        .bind(message_id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(updated_message)
    }

    /// 获取消息的编辑历史
    pub async fn get_message_edit_history(
        &self,
        message_id: Uuid,
        limit: i64,
    ) -> Result<Vec<crate::models::message::MessageEditResponse>> {
        // 检查消息是否存在
        let message_exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(SELECT 1 FROM messages WHERE id = $1)
            "#,
        )
        .bind(message_id)
        .fetch_one(self.db.pool())
        .await?;

        if !message_exists.0 {
            return Err(AppError::NotFound);
        }

        // 获取编辑历史
        let edits = sqlx::query_as::<_, crate::models::message::MessageEdit>(
            r#"
            SELECT * FROM message_edits 
            WHERE message_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(message_id)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await?;

        // 转换为响应格式
        let mut responses = Vec::new();
        for edit in edits {
            let editor = self.get_sender_info(edit.editor_id).await?;
            responses.push(crate::models::message::MessageEditResponse {
                id: edit.id,
                message_id: edit.message_id,
                editor,
                old_content: edit.old_content,
                new_content: edit.new_content,
                created_at: edit.created_at,
            });
        }

        Ok(responses)
    }

    /// 使用全文搜索搜索消息
    pub async fn search_messages_fulltext(
        &self,
        room_id: Option<Uuid>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<crate::models::message::MessageResponse>> {
        // 构建搜索查询 - 使用 | (OR) 操作符连接多个词，并添加前缀匹配
        let search_query = query
            .split_whitespace()
            .map(|word| format!("{}:*", word))
            .collect::<Vec<_>>()
            .join(" | ");

        let messages = if let Some(rid) = room_id {
            sqlx::query_as::<_, crate::models::message::Message>(
                r#"
                SELECT * FROM messages 
                WHERE room_id = $1 
                AND content_tsv @@ to_tsquery('simple', $2)
                AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(rid)
            .bind(&search_query)
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?
        } else {
            sqlx::query_as::<_, crate::models::message::Message>(
                r#"
                SELECT * FROM messages 
                WHERE content_tsv @@ to_tsquery('simple', $1)
                AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(&search_query)
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
}
