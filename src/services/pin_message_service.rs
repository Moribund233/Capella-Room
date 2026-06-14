use uuid::Uuid;

use crate::db::Database;
use crate::error::AppError;
use crate::models::pinned_message::{PinnedMessage, PinnedMessageWithContent};

type Result<T> = std::result::Result<T, AppError>;

pub struct PinMessageService {
    db: Database,
}

impl PinMessageService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn pin_message(
        &self,
        message_id: Uuid,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<PinnedMessage> {
        let pinned = sqlx::query_as::<_, PinnedMessage>(
            r#"
            INSERT INTO pinned_messages (message_id, room_id, pinned_by)
            VALUES ($1, $2, $3)
            ON CONFLICT (message_id) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(message_id)
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        pinned.ok_or(AppError::Conflict("消息已被置顶".to_string()))
    }

    pub async fn unpin_message(&self, message_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM pinned_messages WHERE message_id = $1
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

    pub async fn get_room_pinned_messages(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<PinnedMessageWithContent>> {
        let rows = sqlx::query_as::<_, PinnedMessageWithContent>(
            r#"
            SELECT
                pm.id,
                pm.message_id,
                pm.room_id,
                pm.pinned_by,
                m.content,
                u.username as sender_name,
                pm.created_at
            FROM pinned_messages pm
            JOIN messages m ON m.id = pm.message_id
            JOIN users u ON u.id = m.sender_id
            WHERE pm.room_id = $1 AND m.is_deleted = false
            ORDER BY pm.created_at DESC
            "#,
        )
        .bind(room_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows)
    }
}
