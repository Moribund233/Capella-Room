use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct MessageReaction {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReactionRequest {
    pub emoji: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReactionSummary {
    pub emoji: String,
    pub count: i32,
    pub users: Vec<Uuid>,
}
