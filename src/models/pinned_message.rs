use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PinnedMessage {
    pub id: Uuid,
    pub message_id: Uuid,
    pub room_id: Uuid,
    pub pinned_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PinnedMessageWithContent {
    pub id: Uuid,
    pub message_id: Uuid,
    pub room_id: Uuid,
    pub pinned_by: Uuid,
    pub content: String,
    pub sender_name: String,
    pub created_at: DateTime<Utc>,
}
