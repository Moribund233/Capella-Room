use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    db::Database,
    error::Result,
    models::custom_event::CustomEventRecord,
};

pub struct CustomEventService {
    db: Database,
}

impl CustomEventService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn store_event(
        &self,
        event_name: &str,
        room_id: Uuid,
        source_app: &str,
        data: &serde_json::Value,
    ) -> Result<CustomEventRecord> {
        let record = sqlx::query_as::<_, CustomEventRecord>(
            r#"INSERT INTO custom_events (event_name, room_id, source_app, data)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#
        )
        .bind(event_name)
        .bind(room_id)
        .bind(source_app)
        .bind(data)
        .fetch_one(self.db.pool())
        .await?;
        Ok(record)
    }

    pub async fn get_missed_events(
        &self,
        room_id: Uuid,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<CustomEventRecord>> {
        let events = sqlx::query_as::<_, CustomEventRecord>(
            r#"SELECT * FROM custom_events
               WHERE room_id = $1 AND created_at > $2
               ORDER BY created_at DESC LIMIT $3"#
        )
        .bind(room_id)
        .bind(since)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await?;
        Ok(events)
    }
}

impl Clone for CustomEventService {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}
