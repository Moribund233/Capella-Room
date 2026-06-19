use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ─── DB Models ───

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct WebhookSubscription {
    pub id: Uuid,
    pub app_id: Uuid,
    pub url: String,
    #[serde(skip_serializing)]
    pub secret: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub max_retries: i32,
    pub retry_interval_secs: i32,
    pub timeout_ms: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub event_id: Uuid,
    pub payload: serde_json::Value,
    pub status: String,
    pub http_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempt_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ─── Request DTOs ───

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateWebhookSubscriptionRequest {
    pub url: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebhookSubscriptionRequest {
    pub url: Option<String>,
    pub secret: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

// ─── Response DTOs ───

#[derive(Debug, Clone, Serialize)]
pub struct WebhookSubscriptionResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub max_retries: i32,
    pub retry_interval_secs: i32,
    pub timeout_ms: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WebhookSubscription> for WebhookSubscriptionResponse {
    fn from(s: WebhookSubscription) -> Self {
        Self {
            id: s.id,
            app_id: s.app_id,
            url: s.url,
            events: s.events,
            is_active: s.is_active,
            max_retries: s.max_retries,
            retry_interval_secs: s.retry_interval_secs,
            timeout_ms: s.timeout_ms,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookDeliveryResponse {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub event_id: Uuid,
    pub payload: serde_json::Value,
    pub status: String,
    pub http_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempt_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<WebhookDelivery> for WebhookDeliveryResponse {
    fn from(d: WebhookDelivery) -> Self {
        Self {
            id: d.id,
            subscription_id: d.subscription_id,
            event_type: d.event_type,
            event_id: d.event_id,
            payload: d.payload,
            status: d.status,
            http_status: d.http_status,
            response_body: d.response_body,
            attempt_count: d.attempt_count,
            next_retry_at: d.next_retry_at,
            completed_at: d.completed_at,
            created_at: d.created_at,
        }
    }
}
