use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    models::message::{Message, MessageResponse, MessageType, ReplyToInfo, SenderInfo},
    utils::logging::PerformanceTimer,
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
        let mut timer = PerformanceTimer::new("db_create_message");
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

        timer.finish();
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

    /// 验证被回复的消息是否有效
    /// 检查：消息是否存在、是否在同一会话、是否已被删除
    pub async fn validate_reply_message(&self, reply_to_id: Uuid, room_id: Uuid) -> Result<()> {
        let message: Option<Message> = sqlx::query_as(
            r#"
            SELECT * FROM messages WHERE id = $1
            "#,
        )
        .bind(reply_to_id)
        .fetch_optional(self.db.pool())
        .await?;

        let message = message.ok_or_else(|| AppError::NotFound)?;

        // 检查消息是否在同一会话
        if message.room_id != room_id {
            return Err(AppError::Validation("只能回复同一会话中的消息".to_string()));
        }

        // 检查消息是否已被删除
        if message.is_deleted {
            return Err(AppError::Validation("无法回复已删除的消息".to_string()));
        }

        Ok(())
    }

    /// 获取被引用消息的简要信息
    pub async fn get_reply_to_info(&self, message_id: Uuid) -> Result<Option<ReplyToInfo>> {
        // 先查询消息
        let message: Option<Message> = sqlx::query_as(
            r#"
            SELECT * FROM messages WHERE id = $1
            "#,
        )
        .bind(message_id)
        .fetch_optional(self.db.pool())
        .await?;

        let message = match message {
            Some(m) => m,
            None => return Ok(None),
        };

        // 再查询发送者信息
        let sender = self.get_sender_info(message.sender_id).await?;

        let content = if message.is_deleted {
            "[此消息已被删除]".to_string()
        } else {
            message.content
        };

        Ok(Some(ReplyToInfo {
            id: message.id,
            sender,
            content,
            created_at: message.created_at,
        }))
    }

    /// 批量获取被引用消息的信息
    pub async fn get_reply_to_infos(
        &self,
        message_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, ReplyToInfo>> {
        if message_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // 查询所有消息
        let messages: Vec<Message> = sqlx::query_as(
            r#"
            SELECT * FROM messages WHERE id = ANY($1)
            "#,
        )
        .bind(message_ids)
        .fetch_all(self.db.pool())
        .await?;

        // 收集所有发送者 ID
        let sender_ids: Vec<Uuid> = messages.iter().map(|m| m.sender_id).collect();

        // 批量查询发送者信息
        let sender_infos = self.get_sender_infos(&sender_ids).await?;

        let mut map = HashMap::new();
        for message in messages {
            let content = if message.is_deleted {
                "[此消息已被删除]".to_string()
            } else {
                message.content.clone()
            };

            if let Some(sender) = sender_infos.get(&message.sender_id).cloned() {
                let info = ReplyToInfo {
                    id: message.id,
                    sender,
                    content,
                    created_at: message.created_at,
                };
                map.insert(message.id, info);
            }
        }

        Ok(map)
    }

    /// 批量获取发送者信息
    async fn get_sender_infos(&self, user_ids: &[Uuid]) -> Result<HashMap<Uuid, SenderInfo>> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, username, avatar_url FROM users WHERE id = ANY($1)
            "#,
        )
        .bind(user_ids)
        .fetch_all(self.db.pool())
        .await?;

        let mut map = HashMap::new();
        for (id, username, avatar_url) in rows {
            map.insert(id, SenderInfo::new(id, username, avatar_url));
        }

        Ok(map)
    }

    /// 获取聊天室消息历史
    pub async fn get_room_messages(
        &self,
        room_id: Uuid,
        limit: i64,
        before: Option<Uuid>,
    ) -> Result<Vec<MessageResponse>> {
        let mut timer = PerformanceTimer::new("db_get_room_messages");
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

        // 收集所有需要查询的 reply_to ID
        let reply_to_ids: Vec<Uuid> = messages.iter().filter_map(|msg| msg.reply_to).collect();

        // 批量获取被引用消息的信息
        let reply_to_infos = self.get_reply_to_infos(&reply_to_ids).await?;

        // 批量获取发送者信息
        let sender_ids: Vec<Uuid> = messages.iter().map(|msg| msg.sender_id).collect();
        let sender_infos = self.get_sender_infos(&sender_ids).await?;

        let mut responses = Vec::new();
        for msg in messages {
            if let Some(sender) = sender_infos.get(&msg.sender_id).cloned() {
                let reply_to_message = msg.reply_to.and_then(|id| reply_to_infos.get(&id).cloned());
                responses.push(msg.to_response_with_reply(sender, reply_to_message));
            }
        }

        timer.finish();
        Ok(responses)
    }

    /// 搜索消息
    pub async fn search_messages(
        &self,
        room_id: Option<Uuid>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<MessageResponse>> {
        // 使用 tsvector 全文搜索替代 ILIKE，以利用索引并提升性能
        let search_query = query
            .split_whitespace()
            .map(|word| format!("{}:*", word))
            .collect::<Vec<_>>()
            .join(" | ");

        let messages = if let Some(rid) = room_id {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages
                WHERE room_id = $1 AND content_tsv @@ to_tsquery('simple', $2) AND is_deleted = false
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
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages
                WHERE content_tsv @@ to_tsquery('simple', $1) AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(&search_query)
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?
        };

        // 收集所有需要查询的 reply_to ID
        let reply_to_ids: Vec<Uuid> = messages.iter().filter_map(|msg| msg.reply_to).collect();

        // 批量获取被引用消息的信息
        let reply_to_infos = self.get_reply_to_infos(&reply_to_ids).await?;

        // 批量获取发送者信息
        let sender_ids: Vec<Uuid> = messages.iter().map(|msg| msg.sender_id).collect();
        let sender_infos = self.get_sender_infos(&sender_ids).await?;

        let mut responses = Vec::new();
        for msg in messages {
            if let Some(sender) = sender_infos.get(&msg.sender_id).cloned() {
                let reply_to_message = msg.reply_to.and_then(|id| reply_to_infos.get(&id).cloned());
                responses.push(msg.to_response_with_reply(sender, reply_to_message));
            }
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
    pub async fn get_latest_messages(
        &self,
        room_id: Uuid,
        limit: i64,
    ) -> Result<Vec<MessageResponse>> {
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

        let sender_ids: Vec<Uuid> = messages.iter().map(|msg| msg.sender_id).collect();
        let sender_infos = self.get_sender_infos(&sender_ids).await?;

        let mut responses = Vec::new();
        for msg in messages {
            if let Some(sender) = sender_infos.get(&msg.sender_id).cloned() {
                responses.push(msg.to_response(sender));
            }
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

        // 批量获取发送者信息
        let sender_ids: Vec<Uuid> = messages.iter().map(|msg| msg.sender_id).collect();
        let sender_infos = self.get_sender_infos(&sender_ids).await?;

        let mut responses = Vec::new();
        for msg in messages {
            if let Some(sender) = sender_infos.get(&msg.sender_id).cloned() {
                responses.push(msg.to_response(sender));
            }
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
        if matches!(
            message.message_type,
            crate::models::message::MessageType::System
        ) {
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

        // 批量获取编辑者信息
        let editor_ids: Vec<Uuid> = edits.iter().map(|edit| edit.editor_id).collect();
        let sender_infos = self.get_sender_infos(&editor_ids).await?;

        let mut responses = Vec::new();
        for edit in edits {
            if let Some(editor) = sender_infos.get(&edit.editor_id).cloned() {
                responses.push(crate::models::message::MessageEditResponse {
                    id: edit.id,
                    message_id: edit.message_id,
                    editor,
                    old_content: edit.old_content,
                    new_content: edit.new_content,
                    created_at: edit.created_at,
                });
            }
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

        let sender_ids: Vec<Uuid> = messages.iter().map(|msg| msg.sender_id).collect();
        let sender_infos = self.get_sender_infos(&sender_ids).await?;

        let mut responses = Vec::new();
        for msg in messages {
            if let Some(sender) = sender_infos.get(&msg.sender_id).cloned() {
                responses.push(msg.to_response(sender));
            }
        }

        Ok(responses)
    }

    /// 管理员：获取所有消息列表（支持搜索和分页）
    pub async fn list_all_messages(
        &self,
        search: Option<&str>,
        room_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MessageResponse>, i64)> {
        let messages = if let Some(rid) = room_id {
            if let Some(query) = search {
                sqlx::query_as::<_, Message>(
                    r#"
                    SELECT * FROM messages 
                    WHERE room_id = $1 AND content ILIKE $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(rid)
                .bind(format!("%{}%", query))
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            } else {
                sqlx::query_as::<_, Message>(
                    r#"
                    SELECT * FROM messages 
                    WHERE room_id = $1
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(rid)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            }
        } else if let Some(query) = search {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                WHERE content ILIKE $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(format!("%{}%", query))
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db.pool())
            .await?
        } else {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT * FROM messages 
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db.pool())
            .await?
        };

        let total = self.count_all_messages().await?;

        let sender_ids: Vec<Uuid> = messages.iter().map(|msg| msg.sender_id).collect();
        let sender_infos = self.get_sender_infos(&sender_ids).await?;

        let mut responses = Vec::new();
        for msg in messages {
            if let Some(sender) = sender_infos.get(&msg.sender_id).cloned() {
                responses.push(msg.to_response(sender));
            }
        }

        Ok((responses, total))
    }

    /// 管理员：统计所有消息数
    pub async fn count_all_messages(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM messages WHERE is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(count.0)
    }

    /// 管理员：删除消息（不检查权限）
    pub async fn admin_delete_message(&self, message_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE messages SET is_deleted = true WHERE id = $1
            "#,
        )
        .bind(message_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 获取活跃度统计
    /// 使用单个SQL查询获取所有统计数据，减少数据库往返
    pub async fn get_activity_stats(&self) -> Result<ActivityStats> {
        let stats: ActivityStatsRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(DISTINCT sender_id) FILTER (WHERE created_at > NOW() - INTERVAL '1 day') AS daily_active_users,
                COUNT(DISTINCT sender_id) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') AS weekly_active_users,
                COUNT(DISTINCT sender_id) FILTER (WHERE created_at > NOW() - INTERVAL '30 days') AS monthly_active_users,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '1 day') AS daily_messages,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') AS weekly_messages,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '30 days') AS monthly_messages
            FROM messages
            WHERE is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(ActivityStats {
            daily_active_users: stats.daily_active_users.unwrap_or(0),
            weekly_active_users: stats.weekly_active_users.unwrap_or(0),
            monthly_active_users: stats.monthly_active_users.unwrap_or(0),
            daily_messages: stats.daily_messages.unwrap_or(0),
            weekly_messages: stats.weekly_messages.unwrap_or(0),
            monthly_messages: stats.monthly_messages.unwrap_or(0),
        })
    }

    /// 获取消息类型分布统计
    pub async fn get_message_type_stats(&self) -> Result<MessageTypeStats> {
        let text_messages: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM messages
            WHERE message_type = 'text' AND is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        let image_messages: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM messages
            WHERE message_type = 'image' AND is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        let file_messages: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM messages
            WHERE message_type = 'file' AND is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        let system_messages: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM messages
            WHERE message_type = 'system' AND is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        let reply_messages: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM messages
            WHERE reply_to IS NOT NULL AND is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(MessageTypeStats {
            text_messages,
            image_messages,
            file_messages,
            system_messages,
            reply_messages,
        })
    }

    /// 获取消息时间分布（按小时）
    pub async fn get_message_hourly_distribution(&self) -> Result<Vec<MessageHourlyDistribution>> {
        let rows: Vec<(i32, i64)> = sqlx::query_as(
            r#"
            SELECT 
                EXTRACT(HOUR FROM created_at)::int as hour,
                COUNT(*) as count
            FROM messages
            WHERE created_at > NOW() - INTERVAL '7 days'
            AND is_deleted = false
            GROUP BY EXTRACT(HOUR FROM created_at)
            ORDER BY hour
            "#,
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|(hour, count)| MessageHourlyDistribution { hour, count })
            .collect())
    }
}

/// 活跃度统计数据（数据库查询行映射）
#[derive(Debug, Clone, sqlx::FromRow)]
struct ActivityStatsRow {
    daily_active_users: Option<i64>,
    weekly_active_users: Option<i64>,
    monthly_active_users: Option<i64>,
    daily_messages: Option<i64>,
    weekly_messages: Option<i64>,
    monthly_messages: Option<i64>,
}

/// 活跃度统计数据
#[derive(Debug, Clone)]
pub struct ActivityStats {
    pub daily_active_users: i64,
    pub weekly_active_users: i64,
    pub monthly_active_users: i64,
    pub daily_messages: i64,
    pub weekly_messages: i64,
    pub monthly_messages: i64,
}

/// 消息类型统计
#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageTypeStats {
    pub text_messages: i64,
    pub image_messages: i64,
    pub file_messages: i64,
    pub system_messages: i64,
    pub reply_messages: i64,
}

/// 消息时间分布
#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageHourlyDistribution {
    pub hour: i32,
    pub count: i64,
}
