use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct CustomEventRecord {
    pub id: Uuid,
    pub event_name: String,
    pub room_id: Uuid,
    pub source_app: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomEventHttpRequest {
    pub event_name: String,
    pub room_id: Uuid,
    pub data: serde_json::Value,
    pub persistent: Option<bool>,
}
