use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ─── DB Models ───

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct OAuthApp {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing)]
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub owner_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
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

#[derive(Debug, Clone, FromRow, Serialize)]
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

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserIdentityMapping {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub external_user_id: String,
    pub external_username: Option<String>,
    pub mapped_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Request DTOs ───

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOAuthAppRequest {
    #[validate(length(min = 1, max = 128, message = "应用名称长度必须在 1-128 之间"))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, message = "至少需要一个回调地址"))]
    pub redirect_uris: Vec<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateOAuthAppRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub redirect_uris: Option<Vec<String>>,
    pub scopes: Option<Vec<String>>,
}

// ─── Response DTOs ───

#[derive(Debug, Clone, Serialize)]
pub struct OAuthAppCreatedResponse {
    pub id: Uuid,
    pub name: String,
    pub client_id: Uuid,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthAppResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthAppWithSecretResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── OAuth Flow DTOs ───

#[derive(Debug, Clone, Deserialize)]
pub struct AuthorizeRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthorizeFormRequest {
    pub email: String,
    pub password: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub state: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsentFormRequest {
    pub auth_session_token: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub state: Option<String>,
    pub scope: Option<String>,
    pub approve: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_expires_in: i64,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfoResponse {
    pub sub: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
}

// ─── Identity Mapping DTOs ───

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateMappingRequest {
    pub app_id: Uuid,
    pub user_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub external_user_id: String,
    pub external_username: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MappingLookupQuery {
    pub app_id: Uuid,
    pub external_user_id: String,
}

// ─── Room Resource Binding DTOs ───

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RoomResourceBinding {
    pub id: Uuid,
    pub room_id: Uuid,
    pub app_id: Uuid,
    pub resource_type: String,
    pub resource_id: String,
    pub resource_url: Option<String>,
    pub resource_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct AutoCreateResourceRequest {
    #[validate(length(min = 1, max = 64))]
    pub resource_type: String,
    #[validate(length(min = 1, max = 255))]
    pub resource_id: String,
    pub resource_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateResourceBindingRequest {
    pub app_id: Uuid,
    #[validate(length(min = 1, max = 64))]
    pub resource_type: String,
    #[validate(length(min = 1, max = 255))]
    pub resource_id: String,
    pub resource_url: Option<String>,
    pub resource_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateResourceBindingRequest {
    pub resource_url: Option<String>,
    pub resource_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceLookupQuery {
    pub app_id: Uuid,
    pub resource_type: String,
    pub resource_id: String,
}

// ─── Auth Session Token (for login flow) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSessionToken {
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub exp: u64,
}
