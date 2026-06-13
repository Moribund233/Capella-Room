use std::collections::HashMap;
use uuid::Uuid;

use crate::db::Database;
use crate::error::AppError;
use crate::models::message_reaction::{MessageReaction, ReactionSummary};

type Result<T> = std::result::Result<T, AppError>;

pub struct ReactionService {
    db: Database,
}

impl ReactionService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn add_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<MessageReaction> {
        let reaction = sqlx::query_as::<_, MessageReaction>(
            r#"
            INSERT INTO message_reactions (message_id, user_id, emoji)
            VALUES ($1, $2, $3)
            ON CONFLICT (message_id, user_id, emoji) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .fetch_optional(self.db.pool())
        .await?;

        reaction.ok_or(AppError::Conflict("已经添加过该表情反应".to_string()))
    }

    pub async fn remove_reaction(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM message_reactions
            WHERE message_id = $1 AND user_id = $2 AND emoji = $3
            "#,
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn get_message_reactions(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<ReactionSummary>> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT emoji, COUNT(*) as cnt
            FROM message_reactions
            WHERE message_id = $1
            GROUP BY emoji
            ORDER BY cnt DESC
            "#,
        )
        .bind(message_id)
        .fetch_all(self.db.pool())
        .await?;

        let mut summaries = Vec::new();
        for (emoji, count) in rows {
            let users: Vec<(Uuid,)> = sqlx::query_as(
                r#"
                SELECT user_id FROM message_reactions
                WHERE message_id = $1 AND emoji = $2
                ORDER BY created_at ASC
                "#,
            )
            .bind(message_id)
            .bind(&emoji)
            .fetch_all(self.db.pool())
            .await?;

            summaries.push(ReactionSummary {
                emoji,
                count: count as i32,
                users: users.into_iter().map(|u| u.0).collect(),
            });
        }

        Ok(summaries)
    }

    pub async fn get_messages_reactions(
        &self,
        message_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<ReactionSummary>>> {
        if message_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query_as::<_, (Uuid, String, i64)>(
            r#"
            SELECT message_id, emoji, COUNT(*) as cnt
            FROM message_reactions
            WHERE message_id = ANY($1)
            GROUP BY message_id, emoji
            ORDER BY message_id, cnt DESC
            "#,
        )
        .bind(message_ids)
        .fetch_all(self.db.pool())
        .await?;

        let mut grouped: HashMap<Uuid, Vec<ReactionSummary>> = HashMap::new();
        for (message_id, emoji, count) in rows {
            let users: Vec<(Uuid,)> = sqlx::query_as(
                r#"
                SELECT user_id FROM message_reactions
                WHERE message_id = $1 AND emoji = $2
                ORDER BY created_at ASC
                "#,
            )
            .bind(message_id)
            .bind(&emoji)
            .fetch_all(self.db.pool())
            .await?;

            grouped
                .entry(message_id)
                .or_default()
                .push(ReactionSummary {
                    emoji,
                    count: count as i32,
                    users: users.into_iter().map(|u| u.0).collect(),
                });
        }

        Ok(grouped)
    }
}
