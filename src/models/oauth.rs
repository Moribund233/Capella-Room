use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct OAuthApp {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub owner_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuthorizationCode {
    pub id: Uuid,
    pub app_id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub redirect_uri: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct OAuthToken {
    pub id: Uuid,
    pub app_id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token_hash: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserIdentityMapping {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub external_user_id: String,
    pub external_username: Option<String>,
    pub mapped_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
