# Phase 3: OAuth 2.0 + 房间资源绑定 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement OAuth 2.0 authorization ecosystem (app registration, authorization code flow, token exchange, userinfo, identity mappings) and room-resource bindings for CapellaRoom v2.

**Architecture:** Extend existing OAuthService with authorization/token/userinfo/mapping/resource-binding methods. Add `handlers/oauth.rs` for HTTP endpoints. Add `oauth/templates/` for embedded login/consent pages. Register routes in `routes/mod.rs`. Add `020_room_resource_bindings.sql` migration.

**Tech Stack:** Rust, Axum 0.7, sqlx 0.8, jsonwebtoken 9, bcrypt, serde/serde_json, chrono, uuid

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `migrations/020_room_resource_bindings.sql` | Create | Room resource bindings table |
| `src/models/oauth.rs` | Modify | Add DTOs (CreateOAuthAppRequest, OAuthAppResponse, etc.) + RoomResourceBinding model |
| `src/config/mod.rs` | Modify | Add OAuthConfig struct |
| `src/services/oauth_service.rs` | Modify | Add authorization_code, token, userinfo, mapping, resource_binding methods |
| `src/handlers/oauth.rs` | Create | All OAuth + resource binding HTTP handlers |
| `src/handlers/mod.rs` | Modify | Add `pub mod oauth` |
| `src/oauth/mod.rs` | Create | Module declaration |
| `src/oauth/templates/login.html` | Create | Embedded login page |
| `src/oauth/templates/consent.html` | Create | Embedded consent page |
| `src/lib.rs` | Modify | Add `pub mod oauth` |
| `src/routes/mod.rs` | Modify | Register OAuth routes |
| `src/state/mod.rs` | Modify | Update OAuthService init with config |
| `tests/phase3_1_oauth_apps_test.rs` | Create | App CRUD tests |
| `tests/phase3_2_oauth_flow_test.rs` | Create | OAuth flow tests |
| `tests/phase3_5_resource_bindings_test.rs` | Create | Resource binding tests |

---

## Task 1: Database Migration — Room Resource Bindings

**Files:**
- Create: `migrations/020_room_resource_bindings.sql`

- [ ] **Step 1: Create migration file**

```sql
-- 020_room_resource_bindings.sql
CREATE TABLE room_resource_bindings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id         UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    resource_type   VARCHAR(64) NOT NULL,
    resource_id     VARCHAR(255) NOT NULL,
    resource_url    TEXT,
    resource_name   VARCHAR(255),
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, resource_type, resource_id)
);

CREATE INDEX idx_rrb_room_id ON room_resource_bindings(room_id);
CREATE INDEX idx_rrb_app_id ON room_resource_bindings(app_id);
CREATE INDEX idx_rrb_lookup ON room_resource_bindings(app_id, resource_type, resource_id);
```

- [ ] **Step 2: Verify migration compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors related to migration (sqlx embeds at compile time)

- [ ] **Step 3: Commit**

```bash
git add migrations/020_room_resource_bindings.sql
git commit -m "feat: add room_resource_bindings migration"
```

---

## Task 2: Models — OAuth DTOs + RoomResourceBinding

**Files:**
- Modify: `src/models/oauth.rs`

- [ ] **Step 1: Add Serialize derive to existing models and add DTOs**

Add `use serde::{Deserialize, Serialize};` and `use validator::Validate;` to imports.

Add `#[derive(Serialize)]` to `OAuthApp`, `AuthorizationCode`, `OAuthToken`, `UserIdentityMapping`.

Add new structs after existing models:

```rust
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
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthorizeFormRequest {
    pub email: String,
    pub password: String,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub state: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsentFormRequest {
    pub auth_session_token: String,
    pub client_id: Uuid,
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
    pub client_id: Uuid,
    pub client_secret: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
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
    pub expires_at: DateTime<Utc>,
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -30`
Expected: Compiles with possible unused warnings

- [ ] **Step 3: Commit**

```bash
git add src/models/oauth.rs
git commit -m "feat: add OAuth DTOs and RoomResourceBinding model"
```

---

## Task 3: Config — OAuthConfig

**Files:**
- Modify: `src/config/mod.rs`

- [ ] **Step 1: Add OAuthConfig struct and field to AppConfig**

Add after `MailConfig`:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    pub jwt_secret: Option<String>,
    #[serde(default = "default_access_token_ttl")]
    pub access_token_ttl: i64,
    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl: i64,
    #[serde(default = "default_auth_code_ttl")]
    pub authorization_code_ttl: i64,
}

fn default_access_token_ttl() -> i64 { 3600 }
fn default_refresh_token_ttl() -> i64 { 2592000 }
fn default_auth_code_ttl() -> i64 { 300 }

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: None,
            access_token_ttl: 3600,
            refresh_token_ttl: 2592000,
            authorization_code_ttl: 300,
        }
    }
}
```

Add to `AppConfig`:

```rust
#[serde(default)]
pub oauth: OAuthConfig,
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [ ] **Step 3: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat: add OAuthConfig for OAuth JWT and token TTLs"
```

---

## Task 4: Service — OAuthService Extensions (Authorization Code, Token, UserInfo, Mappings, Resources)

**Files:**
- Modify: `src/services/oauth_service.rs`

- [ ] **Step 1: Update OAuthService struct to hold config**

Replace the struct and constructor:

```rust
use crate::config::OAuthConfig;
use crate::models::oauth::*;
usechrono::Utc;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClaims {
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub scope: Option<String>,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Clone)]
pub struct OAuthService {
    db: Database,
    config: OAuthConfig,
}
```

Update `new()`:

```rust
pub fn new(db: Database, config: OAuthConfig) -> Self {
    Self { db, config }
}
```

- [ ] **Step 2: Add authorization code methods**

```rust
// ─── Authorization Code ───

pub async fn create_authorization_code(
    &self,
    app_id: Uuid,
    user_id: Uuid,
    redirect_uri: &str,
    scopes: &[String],
) -> Result<AuthorizationCode> {
    let code = Self::generate_authorization_code();
    let ttl = self.config.authorization_code_ttl;
    let auth_code = sqlx::query_as::<_, AuthorizationCode>(
        r#"INSERT INTO oauth_authorization_codes (app_id, user_id, code, redirect_uri, scopes, expires_at)
           VALUES ($1, $2, $3, $4, $5, NOW() + ($6 || ' seconds')::interval)
           RETURNING id, app_id, user_id, code, redirect_uri, scopes, expires_at, used_at, created_at"#,
    )
    .bind(app_id)
    .bind(user_id)
    .bind(&code)
    .bind(redirect_uri)
    .bind(scopes)
    .bind(ttl)
    .fetch_one(self.db.pool())
    .await?;

    Ok(auth_code)
}

pub async fn exchange_code(
    &self,
    code: &str,
    client_id: Uuid,
    client_secret: &str,
    redirect_uri: &str,
) -> Result<(AuthorizationCode, String)> {
    let auth_code = sqlx::query_as::<_, AuthorizationCode>(
        r#"SELECT id, app_id, user_id, code, redirect_uri, scopes, expires_at, used_at, created_at
           FROM oauth_authorization_codes
           WHERE code = $1 AND used_at IS NULL AND expires_at > NOW()"#,
    )
    .bind(code)
    .fetch_optional(self.db.pool())
    .await?
    .ok_or_else(|| AppError::Auth("invalid_grant".to_string()))?;

    // Verify client
    let app = self.get_app(auth_code.app_id).await?;
    if app.id != client_id {
        return Err(AppError::Auth("invalid_client".to_string()));
    }
    if !Self::verify_client_secret(client_secret, &app.client_secret) {
        return Err(AppError::Auth("invalid_client".to_string()));
    }
    if !app.is_active {
        return Err(AppError::Auth("unauthorized_client".to_string()));
    }

    // Verify redirect_uri
    if let Some(ref registered_uri) = auth_code.redirect_uri {
        if registered_uri != redirect_uri {
            return Err(AppError::Auth("invalid_grant".to_string()));
        }
    }

    // Mark code as used
    sqlx::query("UPDATE oauth_authorization_codes SET used_at = NOW() WHERE id = $1")
        .bind(auth_code.id)
        .execute(self.db.pool())
        .await?;

    Ok((auth_code, app.id.to_string()))
}

// ─── Token ───

pub async fn generate_tokens(
    &self,
    user_id: Uuid,
    app_id: Uuid,
    scopes: &[String],
) -> Result<TokenResponse> {
    let access_token = self.generate_jwt(user_id, app_id, scopes, self.config.access_token_ttl)?;
    let refresh_token_raw = Self::generate_refresh_token();
    let refresh_token_hash = Self::hash_secret(&refresh_token_raw);

    let refresh_expires_at = Utc::now() + chrono::Duration::seconds(self.config.refresh_token_ttl);

    sqlx::query(
        r#"INSERT INTO oauth_tokens (app_id, user_id, access_token, refresh_token_hash, scopes, expires_at, refresh_expires_at)
           VALUES ($1, $2, $3, $4, $5, NOW() + ($6 || ' seconds')::interval, $7)"#,
    )
    .bind(app_id)
    .bind(user_id)
    .bind(&access_token)
    .bind(&refresh_token_hash)
    .bind(scopes)
    .bind(self.config.access_token_ttl)
    .bind(refresh_expires_at)
    .execute(self.db.pool())
    .await?;

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: self.config.access_token_ttl,
        scope: Some(scopes.join(" ")),
    })
}

pub async fn exchange_refresh_token(
    &self,
    refresh_token: &str,
    client_id: Uuid,
    client_secret: &str,
) -> Result<TokenResponse> {
    // Find token by hash
    let refresh_hash = Self::hash_secret(refresh_token);
    let token = sqlx::query_as::<_, OAuthToken>(
        r#"SELECT id, app_id, user_id, access_token, refresh_token_hash, scopes, expires_at, refresh_expires_at, revoked_at, created_at
           FROM oauth_tokens
           WHERE refresh_token_hash = $1 AND revoked_at IS NULL AND refresh_expires_at > NOW()"#,
    )
    .bind(&refresh_hash)
    .fetch_optional(self.db.pool())
    .await?
    .ok_or_else(|| AppError::Auth("invalid_grant".to_string()))?;

    // Verify client
    let app = self.get_app(client_id).await?;
    if app.id != client_id || !Self::verify_client_secret(client_secret, &app.client_secret) {
        return Err(AppError::Auth("invalid_client".to_string()));
    }

    // Revoke old token
    sqlx::query("UPDATE oauth_tokens SET revoked_at = NOW() WHERE id = $1")
        .bind(token.id)
        .execute(self.db.pool())
        .await?;

    // Generate new tokens
    let scopes = token.scopes.unwrap_or_default();
    self.generate_tokens(token.user_id, client_id, &scopes).await
}

pub async fn client_credentials_grant(
    &self,
    client_id: Uuid,
    client_secret: &str,
) -> Result<TokenResponse> {
    let app = self.get_app(client_id).await?;
    if !Self::verify_client_secret(client_secret, &app.client_secret) {
        return Err(AppError::Auth("invalid_client".to_string()));
    }
    if !app.is_active {
        return Err(AppError::Auth("unauthorized_client".to_string()));
    }

    // Client credentials: no user context, use a nil UUID
    let nil_uuid = Uuid::nil();
    let scopes = app.scopes.clone();
    self.generate_tokens(nil_uuid, client_id, &scopes).await
}

pub async fn verify_access_token(&self, token: &str) -> Result<OAuthClaims> {
    let secret = self.config.jwt_secret.as_deref()
        .ok_or_else(|| AppError::Auth("OAuth JWT secret not configured".to_string()))?;

    let token_data = decode::<OAuthClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| AppError::Auth(format!("invalid_token: {}", e)))?;

    Ok(token_data.claims)
}

// ─── UserInfo ───

pub async fn get_user_info(&self, user_id: Uuid) -> Result<UserInfoResponse> {
    let user = sqlx::query_as::<_, crate::models::user::User>(
        r#"SELECT id, username, email, password_hash, role, status, avatar_url, created_at, updated_at, email_verified, email_verified_at
           FROM users WHERE id = $1"#,
    )
    .bind(user_id)
    .fetch_optional(self.db.pool())
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    Ok(UserInfoResponse {
        sub: user.id,
        username: user.username,
        email: user.email,
        email_verified: user.email_verified,
        avatar_url: user.avatar_url,
    })
}

// ─── Identity Mappings ───

pub async fn create_mapping(
    &self,
    app_id: Uuid,
    user_id: Uuid,
    external_user_id: &str,
    external_username: Option<&str>,
) -> Result<UserIdentityMapping> {
    let mapping = sqlx::query_as::<_, UserIdentityMapping>(
        r#"INSERT INTO user_identity_mappings (app_id, user_id, external_user_id, external_username)
           VALUES ($1, $2, $3, $4)
           RETURNING id, user_id, app_id, external_user_id, external_username, mapped_at, updated_at"#,
    )
    .bind(app_id)
    .bind(user_id)
    .bind(external_user_id)
    .bind(external_username)
    .fetch_one(self.db.pool())
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => {
            if db_err.constraint().map(|c| c.contains("unique")).unwrap_or(false) {
                AppError::Conflict("身份映射已存在".to_string())
            } else {
                AppError::Database(e)
            }
        }
        _ => AppError::Database(e),
    })?;

    Ok(mapping)
}

pub async fn lookup_mapping(
    &self,
    app_id: Uuid,
    external_user_id: &str,
) -> Result<Option<UserIdentityMapping>> {
    let mapping = sqlx::query_as::<_, UserIdentityMapping>(
        r#"SELECT id, user_id, app_id, external_user_id, external_username, mapped_at, updated_at
           FROM user_identity_mappings
           WHERE app_id = $1 AND external_user_id = $2"#,
    )
    .bind(app_id)
    .bind(external_user_id)
    .fetch_optional(self.db.pool())
    .await?;

    Ok(mapping)
}

pub async fn delete_mapping(&self, mapping_id: Uuid) -> Result<()> {
    let result = sqlx::query("DELETE FROM user_identity_mappings WHERE id = $1")
        .bind(mapping_id)
        .execute(self.db.pool())
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

// ─── Room Resource Bindings ───

pub async fn create_resource_binding(
    &self,
    room_id: Uuid,
    request: CreateResourceBindingRequest,
) -> Result<RoomResourceBinding> {
    let binding = sqlx::query_as::<_, RoomResourceBinding>(
        r#"INSERT INTO room_resource_bindings (room_id, app_id, resource_type, resource_id, resource_url, resource_name, metadata)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, room_id, app_id, resource_type, resource_id, resource_url, resource_name, metadata, created_at, updated_at"#,
    )
    .bind(room_id)
    .bind(request.app_id)
    .bind(&request.resource_type)
    .bind(&request.resource_id)
    .bind(&request.resource_url)
    .bind(&request.resource_name)
    .bind(request.metadata.unwrap_or(serde_json::json!({})))
    .fetch_one(self.db.pool())
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => {
            if db_err.constraint().map(|c| c.contains("unique")).unwrap_or(false) {
                AppError::Conflict("资源绑定已存在".to_string())
            } else {
                AppError::Database(e)
            }
        }
        _ => AppError::Database(e),
    })?;

    Ok(binding)
}

pub async fn list_resource_bindings(&self, room_id: Uuid) -> Result<Vec<RoomResourceBinding>> {
    let bindings = sqlx::query_as::<_, RoomResourceBinding>(
        r#"SELECT id, room_id, app_id, resource_type, resource_id, resource_url, resource_name, metadata, created_at, updated_at
           FROM room_resource_bindings WHERE room_id = $1 ORDER BY created_at DESC"#,
    )
    .bind(room_id)
    .fetch_all(self.db.pool())
    .await?;

    Ok(bindings)
}

pub async fn lookup_resource(
    &self,
    app_id: Uuid,
    resource_type: &str,
    resource_id: &str,
) -> Result<Option<RoomResourceBinding>> {
    let binding = sqlx::query_as::<_, RoomResourceBinding>(
        r#"SELECT id, room_id, app_id, resource_type, resource_id, resource_url, resource_name, metadata, created_at, updated_at
           FROM room_resource_bindings
           WHERE app_id = $1 AND resource_type = $2 AND resource_id = $3"#,
    )
    .bind(app_id)
    .bind(resource_type)
    .bind(resource_id)
    .fetch_optional(self.db.pool())
    .await?;

    Ok(binding)
}

pub async fn update_resource_binding(
    &self,
    binding_id: Uuid,
    resource_url: Option<&str>,
    resource_name: Option<&str>,
    metadata: Option<serde_json::Value>,
) -> Result<RoomResourceBinding> {
    let binding = sqlx::query_as::<_, RoomResourceBinding>(
        r#"UPDATE room_resource_bindings
           SET resource_url = COALESCE($2, resource_url),
               resource_name = COALESCE($3, resource_name),
               metadata = COALESCE($4, metadata),
               updated_at = NOW()
           WHERE id = $1
           RETURNING id, room_id, app_id, resource_type, resource_id, resource_url, resource_name, metadata, created_at, updated_at"#,
    )
    .bind(binding_id)
    .bind(resource_url)
    .bind(resource_name)
    .bind(metadata)
    .fetch_optional(self.db.pool())
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(binding)
}

pub async fn delete_resource_binding(&self, binding_id: Uuid) -> Result<()> {
    let result = sqlx::query("DELETE FROM room_resource_bindings WHERE id = $1")
        .bind(binding_id)
        .execute(self.db.pool())
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

// ─── Auth Session Token (for login flow) ───

pub fn create_auth_session_token(&self, user_id: Uuid, client_id: Uuid) -> Result<String> {
    let token = AuthSessionToken {
        user_id,
        client_id,
        expires_at: Utc::now() + chrono::Duration::minutes(5),
    };
    let secret = self.config.jwt_secret.as_deref()
        .ok_or_else(|| AppError::Auth("OAuth JWT secret not configured".to_string()))?;
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &token, &encoding_key)
        .map_err(|e| AppError::Auth(format!("Failed to create auth session: {}", e)))
}

pub fn verify_auth_session_token(&self, token_str: &str) -> Result<AuthSessionToken> {
    let secret = self.config.jwt_secret.as_deref()
        .ok_or_else(|| AppError::Auth("OAuth JWT secret not configured".to_string()))?;
    let token_data = decode::<AuthSessionToken>(
        token_str,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| AppError::Auth(format!("Invalid auth session: {}", e)))?;

    if token_data.claims.expires_at < Utc::now() {
        return Err(AppError::Auth("Auth session expired".to_string()));
    }

    Ok(token_data.claims)
}

// ─── Internal Helpers ───

fn generate_jwt(&self, user_id: Uuid, app_id: Uuid, scopes: &[String], ttl: i64) -> Result<String> {
    let secret = self.config.jwt_secret.as_deref()
        .ok_or_else(|| AppError::Auth("OAuth JWT secret not configured".to_string()))?;

    let now = Utc::now();
    let exp = (now + chrono::Duration::seconds(ttl)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = OAuthClaims {
        sub: user_id.to_string(),
        aud: app_id.to_string(),
        iss: "capella-room".to_string(),
        scope: Some(scopes.join(" ")),
        iat,
        exp,
    };

    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &encoding_key)
        .map_err(|e| AppError::Auth(format!("Failed to generate JWT: {}", e)))
}

fn generate_authorization_code() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

fn generate_refresh_token() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

fn hash_secret(secret: &str) -> String {
    hash(secret, DEFAULT_COST).unwrap_or_default()
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -30`

- [ ] **Step 4: Commit**

```bash
git add src/services/oauth_service.rs
git commit -m "feat: extend OAuthService with auth code, token, userinfo, mapping, resource methods"
```

---

## Task 5: Handlers — OAuth HTTP Endpoints

**Files:**
- Create: `src/handlers/oauth.rs`
- Modify: `src/handlers/mod.rs`

- [ ] **Step 1: Add module declaration to handlers/mod.rs**

Add `pub mod oauth;` to `src/handlers/mod.rs`.

- [ ] **Step 2: Create handlers/oauth.rs — App CRUD handlers**

```rust
use axum::{
    extract::{Extension, Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::{
        oauth::*,
        response::ApiResponse,
        room::MemberRole,
    },
    services::auth_service::Claims,
    state::AppState,
};

// ─── Helper ───

fn extract_user_id(claims: &Claims) -> Result<Uuid> {
    Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))
}

// ═══════════════════════════════════════════════
// 3.1 OAuth App CRUD
// ═══════════════════════════════════════════════

pub async fn create_app(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateOAuthAppRequest>,
) -> Result<(StatusCode, Json<ApiResponse<OAuthAppCreatedResponse>>)> {
    let user_id = extract_user_id(&claims)?;
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let scopes: Vec<String> = request.scopes.unwrap_or_default();
    let redirect_uri_refs: Vec<&str> = request.redirect_uris.iter().map(|s| s.as_str()).collect();
    let scope_refs: Vec<&str> = scopes.iter().map(|s| s.as_str()).collect();

    let app = state.oauth_service().register_app(
        user_id,
        &request.name,
        request.description.as_deref(),
        &redirect_uri_refs,
        &scope_refs,
    ).await?;

    let response = OAuthAppCreatedResponse {
        id: app.id,
        name: app.name,
        client_id: app.id,
        client_secret: app.client_secret,
        redirect_uris: app.redirect_uris,
        scopes: app.scopes,
        is_active: app.is_active,
        created_at: app.created_at,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

pub async fn list_apps(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<OAuthAppResponse>>>> {
    let user_id = extract_user_id(&claims)?;
    let apps = state.oauth_service().list_apps(user_id).await?;

    let responses: Vec<OAuthAppResponse> = apps.into_iter().map(|a| OAuthAppResponse {
        id: a.id,
        name: a.name,
        description: a.description,
        redirect_uris: a.redirect_uris,
        scopes: a.scopes,
        is_active: a.is_active,
        created_at: a.created_at,
        updated_at: a.updated_at,
    }).collect();

    Ok(Json(ApiResponse::success(responses)))
}

pub async fn get_app(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApiResponse<OAuthAppResponse>>> {
    let user_id = extract_user_id(&claims)?;
    let app = state.oauth_service().get_app(app_id).await?;
    if app.owner_id != user_id {
        return Err(AppError::Forbidden);
    }

    let response = OAuthAppResponse {
        id: app.id,
        name: app.name,
        description: app.description,
        redirect_uris: app.redirect_uris,
        scopes: app.scopes,
        is_active: app.is_active,
        created_at: app.created_at,
        updated_at: app.updated_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn update_app(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
    Json(request): Json<UpdateOAuthAppRequest>,
) -> Result<Json<ApiResponse<OAuthAppResponse>>> {
    let user_id = extract_user_id(&claims)?;

    let name = request.name.as_deref();
    let description = request.description.as_deref();
    let redirect_uris = request.redirect_uris.as_deref().map(|v| v.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let scopes = request.scopes.as_deref().map(|v| v.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let app = state.oauth_service().update_app(
        app_id,
        user_id,
        name,
        description,
        redirect_uris.as_deref(),
        scopes.as_deref(),
    ).await?;

    let response = OAuthAppResponse {
        id: app.id,
        name: app.name,
        description: app.description,
        redirect_uris: app.redirect_uris,
        scopes: app.scopes,
        is_active: app.is_active,
        created_at: app.created_at,
        updated_at: app.updated_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn delete_app(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id = extract_user_id(&claims)?;
    state.oauth_service().delete_app(app_id, user_id).await?;
    Ok(Json(ApiResponse::success_with_message("应用已删除")))
}

pub async fn rotate_secret(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApiResponse<OAuthAppWithSecretResponse>>> {
    let user_id = extract_user_id(&claims)?;
    let new_secret = state.oauth_service().rotate_secret(app_id, user_id).await?;
    let app = state.oauth_service().get_app(app_id).await?;

    let response = OAuthAppWithSecretResponse {
        id: app.id,
        name: app.name,
        description: app.description,
        client_secret: new_secret,
        redirect_uris: app.redirect_uris,
        scopes: app.scopes,
        is_active: app.is_active,
        created_at: app.created_at,
        updated_at: app.updated_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

// ═══════════════════════════════════════════════
// 3.2 OAuth Flow — Authorize endpoint (browser)
// ═══════════════════════════════════════════════

pub async fn authorize_get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuthorizeRequest>,
) -> Result<Response> {
    // Validate app exists
    let app = state.oauth_service().get_app(params.client_id).await?;
    if !app.is_active {
        return Err(AppError::Auth("应用未激活".to_string()));
    }

    // Validate redirect_uri
    if !app.redirect_uris.contains(&params.redirect_uri) {
        return Err(AppError::Validation("redirect_uri 不匹配".to_string()));
    }

    if params.response_type != "code" {
        return Err(AppError::Validation("仅支持 response_type=code".to_string()));
    }

    // Render login page
    let login_html = include_str!("../oauth/templates/login.html");
    let html = login_html
        .replace("{{client_id}}", &params.client_id.to_string())
        .replace("{{redirect_uri}}", &params.redirect_uri)
        .replace("{{state}}", &params.state.unwrap_or_default())
        .replace("{{scope}}", &params.scope.unwrap_or_default())
        .replace("{{app_name}}", &app.name)
        .replace("{{error}}", "");

    Ok(Html(html).into_response())
}

pub async fn authorize_post(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AuthorizeFormRequest>,
) -> Result<Response> {
    // Validate app
    let app = state.oauth_service().get_app(form.client_id).await?;
    if !app.is_active {
        return Err(AppError::Auth("应用未激活".to_string()));
    }
    if !app.redirect_uris.contains(&form.redirect_uri) {
        return Err(AppError::Validation("redirect_uri 不匹配".to_string()));
    }

    // Authenticate user (same pattern as auth_v2::login_with_password)
    let email = form.email.trim().to_lowercase();
    let user = state.user_service().get_user_by_email(&email).await?
        .ok_or_else(|| AppError::Auth("邮箱或密码错误".to_string()))?;

    if user.is_account_disabled() {
        return Err(AppError::Auth("账号已被禁用".to_string()));
    }

    let password_valid = state.auth_service().verify_password(&form.password, &user.password_hash)?;
    if !password_valid {
        return Err(AppError::Auth("邮箱或密码错误".to_string()));
    }

    // Create auth session token
    let session_token = state.oauth_service().create_auth_session_token(user.id, form.client_id)?;

    // Render consent page
    let consent_html = include_str!("../oauth/templates/consent.html");
    let app_name_short = app.name.chars().next().unwrap_or('?').to_string();
    let html = consent_html
        .replace("{{auth_session_token}}", &session_token)
        .replace("{{client_id}}", &form.client_id.to_string())
        .replace("{{redirect_uri}}", &form.redirect_uri)
        .replace("{{response_type}}", &form.response_type)
        .replace("{{state}}", &form.state.unwrap_or_default())
        .replace("{{scope}}", &form.scope.unwrap_or_default())
        .replace("{{app_name}}", &app.name)
        .replace("{{app_name_short}}", &app_name_short)
        .replace("{{username}}", &user.username);

    Ok(Html(html).into_response())
}

pub async fn authorize_consent(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ConsentFormRequest>,
) -> Result<Response> {
    // Verify auth session
    let session = state.oauth_service().verify_auth_session_token(&form.auth_session_token)?;

    if session.client_id != form.client_id {
        return Err(AppError::Auth("client_id 不匹配".to_string()));
    }

    let scopes: Vec<String> = form.scope
        .unwrap_or_default()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if form.approve.as_deref() == Some("true") {
        // Generate authorization code
        let auth_code = state.oauth_service().create_authorization_code(
            form.client_id,
            session.user_id,
            &form.redirect_uri,
            &scopes,
        ).await?;

        let state_param = form.state.unwrap_or_default();
        let redirect_url = format!("{}?code={}&state={}", form.redirect_uri, auth_code.code, state_param);

        Ok(Redirect::to(&redirect_url).into_response())
    } else {
        let state_param = form.state.unwrap_or_default();
        let redirect_url = format!("{}?error=access_denied&state={}", form.redirect_uri, state_param);
        Ok(Redirect::to(&redirect_url).into_response())
    }
}

// ═══════════════════════════════════════════════
// 3.2 OAuth Flow — Token endpoint
// ═══════════════════════════════════════════════

pub async fn token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TokenRequest>,
) -> Result<Json<TokenResponse>> {
    match request.grant_type.as_str() {
        "authorization_code" => {
            let code = request.code.ok_or_else(|| AppError::Validation("缺少 code".to_string()))?;
            let redirect_uri = request.redirect_uri.ok_or_else(|| AppError::Validation("缺少 redirect_uri".to_string()))?;

            let (auth_code, _) = state.oauth_service().exchange_code(
                &code,
                request.client_id,
                &request.client_secret,
                &redirect_uri,
            ).await?;

            let scopes = auth_code.scopes.unwrap_or_default();
            let response = state.oauth_service().generate_tokens(
                auth_code.user_id,
                request.client_id,
                &scopes,
            ).await?;

            Ok(Json(response))
        }
        "refresh_token" => {
            let refresh_token = request.refresh_token.ok_or_else(|| AppError::Validation("缺少 refresh_token".to_string()))?;

            let response = state.oauth_service().exchange_refresh_token(
                &refresh_token,
                request.client_id,
                &request.client_secret,
            ).await?;

            Ok(Json(response))
        }
        "client_credentials" => {
            let response = state.oauth_service().client_credentials_grant(
                request.client_id,
                &request.client_secret,
            ).await?;

            Ok(Json(response))
        }
        _ => Err(AppError::Auth("unsupported_grant_type".to_string())),
    }
}

// ═══════════════════════════════════════════════
// 3.2 UserInfo endpoint
// ═══════════════════════════════════════════════

pub async fn userinfo(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserInfoResponse>> {
    let user_id = extract_user_id(&claims)?;
    let user_info = state.oauth_service().get_user_info(user_id).await?;
    Ok(Json(user_info))
}

// ═══════════════════════════════════════════════
// 3.2 Identity Mapping endpoints
// ═══════════════════════════════════════════════

pub async fn create_mapping(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateMappingRequest>,
) -> Result<(StatusCode, Json<ApiResponse<UserIdentityMapping>>)> {
    let _user_id = extract_user_id(&claims)?;
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let mapping = state.oauth_service().create_mapping(
        request.app_id,
        request.user_id,
        &request.external_user_id,
        request.external_username.as_deref(),
    ).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(mapping))))
}

pub async fn lookup_mapping(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<MappingLookupQuery>,
) -> Result<Json<ApiResponse<Option<UserIdentityMapping>>>> {
    let mapping = state.oauth_service().lookup_mapping(query.app_id, &query.external_user_id).await?;
    Ok(Json(ApiResponse::success(mapping)))
}

pub async fn delete_mapping_handler(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(mapping_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    state.oauth_service().delete_mapping(mapping_id).await?;
    Ok(Json(ApiResponse::success_with_message("映射已解除")))
}

// ═══════════════════════════════════════════════
// 3.5 Room Resource Binding endpoints
// ═══════════════════════════════════════════════

pub async fn bind_resource(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<Uuid>,
    Json(request): Json<CreateResourceBindingRequest>,
) -> Result<(StatusCode, Json<ApiResponse<RoomResourceBinding>>)> {
    let user_id = extract_user_id(&claims)?;
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Check user is room admin/owner
    let member = state.room_service().get_room_member(room_id, user_id).await?;
    match member {
        Some(m) if m.role == MemberRole::Owner || m.role == MemberRole::Admin => {}
        _ => return Err(AppError::Forbidden),
    }

    let binding = state.oauth_service().create_resource_binding(room_id, request).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::success(binding))))
}

pub async fn list_bindings(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<RoomResourceBinding>>>> {
    let bindings = state.oauth_service().list_resource_bindings(room_id).await?;
    Ok(Json(ApiResponse::success(bindings)))
}

pub async fn lookup_resource(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<ResourceLookupQuery>,
) -> Result<Json<ApiResponse<Option<RoomResourceBinding>>>> {
    let binding = state.oauth_service().lookup_resource(query.app_id, &query.resource_type, &query.resource_id).await?;
    Ok(Json(ApiResponse::success(binding)))
}

pub async fn update_binding(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((room_id, binding_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateResourceBindingRequest>,
) -> Result<Json<ApiResponse<RoomResourceBinding>>> {
    let user_id = extract_user_id(&claims)?;

    // Check user is room admin/owner
    let member = state.room_service().get_room_member(room_id, user_id).await?;
    match member {
        Some(m) if m.role == MemberRole::Owner || m.role == MemberRole::Admin => {}
        _ => return Err(AppError::Forbidden),
    }

    let binding = state.oauth_service().update_resource_binding(
        binding_id,
        request.resource_url.as_deref(),
        request.resource_name.as_deref(),
        request.metadata,
    ).await?;

    Ok(Json(ApiResponse::success(binding)))
}

pub async fn unbind_resource(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((room_id, binding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id = extract_user_id(&claims)?;

    // Check user is room admin/owner
    let member = state.room_service().get_room_member(room_id, user_id).await?;
    match member {
        Some(m) if m.role == MemberRole::Owner || m.role == MemberRole::Admin => {}
        _ => return Err(AppError::Forbidden),
    }

    state.oauth_service().delete_resource_binding(binding_id).await?;
    Ok(Json(ApiResponse::success_with_message("资源已解绑")))
}

```

- [ ] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -40`

- [ ] **Step 4: Commit**

```bash
git add src/handlers/oauth.rs src/handlers/mod.rs
git commit -m "feat: add OAuth handlers for app CRUD, authorize, token, userinfo, mappings, resources"
```

---

## Task 6: OAuth Templates — Login + Consent HTML

**Files:**
- Create: `src/oauth/mod.rs`
- Create: `src/oauth/templates/login.html`
- Create: `src/oauth/templates/consent.html`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create src/oauth/mod.rs**

```rust
pub mod templates;
```

- [ ] **Step 2: Create src/oauth/templates/mod.rs**

```rust
// Templates are loaded via include_str! in handlers
```

Actually, we don't need templates mod.rs since handlers load HTML directly. Just create the HTML files.

- [ ] **Step 3: Create login.html**

Create directory `src/oauth/templates/` and file `login.html`:

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>登录 - CapellaRoom</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #0f0f14; color: #e4e4e7; display: flex; justify-content: center; align-items: center; min-height: 100vh; }
        .card { background: #1a1a24; border-radius: 12px; padding: 40px; width: 100%; max-width: 400px; box-shadow: 0 8px 32px rgba(0,0,0,0.4); }
        .app-name { text-align: center; margin-bottom: 8px; font-size: 14px; color: #7c5cfc; }
        h1 { text-align: center; margin-bottom: 24px; font-size: 20px; font-weight: 600; }
        .form-group { margin-bottom: 16px; }
        label { display: block; margin-bottom: 6px; font-size: 13px; color: #a1a1aa; }
        input { width: 100%; padding: 10px 14px; border: 1px solid #27272a; border-radius: 8px; background: #0f0f14; color: #e4e4e7; font-size: 14px; outline: none; transition: border-color 0.15s; }
        input:focus { border-color: #7c5cfc; }
        button { width: 100%; padding: 10px; border: none; border-radius: 8px; background: #7c5cfc; color: #fff; font-size: 14px; font-weight: 500; cursor: pointer; transition: background 0.15s; }
        button:hover { background: #6a4fe0; }
        .error { background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.3); color: #fca5a5; padding: 10px; border-radius: 8px; margin-bottom: 16px; font-size: 13px; text-align: center; }
    </style>
</head>
<body>
    <div class="card">
        <div class="app-name">{{app_name}} 请求访问</div>
        <h1>登录 CapellaRoom</h1>
        {{error}}
        <form method="post" action="/oauth/authorize">
            <input type="hidden" name="client_id" value="{{client_id}}">
            <input type="hidden" name="redirect_uri" value="{{redirect_uri}}">
            <input type="hidden" name="response_type" value="code">
            <input type="hidden" name="state" value="{{state}}">
            <input type="hidden" name="scope" value="{{scope}}">
            <div class="form-group">
                <label for="email">邮箱</label>
                <input type="email" id="email" name="email" placeholder="your@email.com" required autofocus>
            </div>
            <div class="form-group">
                <label for="password">密码</label>
                <input type="password" id="password" name="password" placeholder="••••••••" required>
            </div>
            <button type="submit">登录并授权</button>
        </form>
    </div>
</body>
</html>
```

- [ ] **Step 4: Create consent.html**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>授权确认 - CapellaRoom</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #0f0f14; color: #e4e4e7; display: flex; justify-content: center; align-items: center; min-height: 100vh; }
        .card { background: #1a1a24; border-radius: 12px; padding: 40px; width: 100%; max-width: 400px; box-shadow: 0 8px 32px rgba(0,0,0,0.4); }
        .app-icon { text-align: center; margin-bottom: 16px; }
        .app-icon span { display: inline-flex; align-items: center; justify-content: center; width: 56px; height: 56px; border-radius: 14px; background: linear-gradient(135deg, #7c5cfc, #ec4899); font-size: 24px; font-weight: 700; color: #fff; }
        h1 { text-align: center; margin-bottom: 8px; font-size: 18px; font-weight: 600; }
        .subtitle { text-align: center; color: #a1a1aa; font-size: 13px; margin-bottom: 24px; }
        .user-info { text-align: center; margin-bottom: 20px; padding: 10px; background: #0f0f14; border-radius: 8px; font-size: 13px; color: #a1a1aa; }
        .scopes { margin-bottom: 24px; }
        .scopes h3 { font-size: 13px; color: #a1a1aa; margin-bottom: 8px; }
        .scope-item { display: flex; align-items: center; gap: 8px; padding: 8px 12px; background: #0f0f14; border-radius: 6px; margin-bottom: 4px; font-size: 13px; }
        .scope-item::before { content: "✓"; color: #22c55e; font-weight: 700; }
        .actions { display: flex; gap: 12px; }
        .btn { flex: 1; padding: 10px; border: none; border-radius: 8px; font-size: 14px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
        .btn-allow { background: #7c5cfc; color: #fff; }
        .btn-allow:hover { background: #6a4fe0; }
        .btn-deny { background: transparent; border: 1px solid #27272a; color: #a1a1aa; }
        .btn-deny:hover { border-color: #52525b; color: #e4e4e7; }
    </style>
</head>
<body>
    <div class="card">
        <div class="app-icon"><span>{{app_name_short}}</span></div>
        <h1>{{app_name}} 请求访问</h1>
        <p class="subtitle">授权后，该应用将获取以下权限</p>
        <div class="user-info">以 <strong>{{username}}</strong> 的身份继续</div>
        <form method="post" action="/oauth/authorize/consent">
            <input type="hidden" name="auth_session_token" value="{{auth_session_token}}">
            <input type="hidden" name="client_id" value="{{client_id}}">
            <input type="hidden" name="redirect_uri" value="{{redirect_uri}}">
            <input type="hidden" name="response_type" value="{{response_type}}">
            <input type="hidden" name="state" value="{{state}}">
            <input type="hidden" name="scope" value="{{scope}}">
            <div class="scopes">
                <h3>将被授予的权限：</h3>
                <div class="scope-item" id="scope-list"></div>
            </div>
            <div class="actions">
                <button type="submit" name="deny" class="btn btn-deny">拒绝</button>
                <button type="submit" name="approve" value="true" class="btn btn-allow">授权</button>
            </div>
        </form>
    </div>
    <script>
        const scope = '{{scope}}';
        const scopeList = document.getElementById('scope-list');
        if (scope) {
            scopeList.innerHTML = scope.split(' ').map(s => {
                const labels = { openid: '身份标识', profile: '用户资料', email: '邮箱地址' };
                return '<div class="scope-item">' + (labels[s] || s) + '</div>';
            }).join('');
        } else {
            scopeList.innerHTML = '<div class="scope-item">基本访问权限</div>';
        }
        // Set app_name_short from first char
        document.querySelector('.app-icon span').textContent = '{{app_name}}'[0] || '?';
    </script>
</body>
</html>
```

- [ ] **Step 5: Add oauth module to lib.rs**

Add `pub mod oauth;` to `src/lib.rs`.

- [ ] **Step 6: Verify compilation**

Run: `cargo check 2>&1 | head -30`

- [ ] **Step 7: Commit**

```bash
git add src/oauth/ src/lib.rs
git commit -m "feat: add OAuth login and consent HTML templates"
```

---

## Task 7: Routes — Register OAuth Routes

**Files:**
- Modify: `src/routes/mod.rs`

- [ ] **Step 1: Add oauth handler import**

Add to imports: `handlers::oauth`

- [ ] **Step 2: Add OAuth browser routes (public, no auth_middleware)**

Add to `create_router` function, before `auth_routes_router`:

```rust
// OAuth 浏览器端点（不使用 auth_middleware，内部处理认证）
let oauth_browser_routes = Router::new()
    .route("/oauth/authorize", get(oauth::authorize_get).post(oauth::authorize_post))
    .route("/oauth/authorize/consent", post(oauth::authorize_consent))
    .route("/oauth/token", post(oauth::token));
```

- [ ] **Step 3: Add OAuth API routes (protected)**

Add to `protected_routes`:

```rust
// OAuth API 路由
.nest("/api/v2/oauth", oauth_api_routes())
```

Add new function:

```rust
fn oauth_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/apps", post(oauth::create_app).get(oauth::list_apps))
        .route("/apps/:id", get(oauth::get_app).put(oauth::update_app).delete(oauth::delete_app))
        .route("/apps/:id/secret", patch(oauth::rotate_secret))
        .route("/userinfo", get(oauth::userinfo))
        .route("/mappings", post(oauth::create_mapping))
        .route("/mappings/lookup", get(oauth::lookup_mapping))
        .route("/mappings/:id", delete(oauth::delete_mapping_handler))
}
```

- [ ] **Step 4: Add resource binding routes**

Add to `protected_routes`:

```rust
// 房间资源绑定
.nest("/api/v2/rooms", resource_routes())
```

Add new function:

```rust
fn resource_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:room_id/resources", post(oauth::bind_resource).get(oauth::list_bindings))
        .route("/:room_id/resources/:id", patch(oauth::update_binding).delete(oauth::unbind_resource))
        .route("/resources/lookup", get(oauth::lookup_resource))
}
```

- [ ] **Step 5: Merge oauth_browser_routes**

Update the merge at end of `create_router`:

```rust
public_routes
    .merge(oauth_browser_routes)
    .merge(auth_routes_router)
    .merge(register_admin_router)
    .merge(protected_routes)
    .merge(admin_routes)
    .with_state(state)
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check 2>&1 | head -40`

- [ ] **Step 7: Commit**

```bash
git add src/routes/mod.rs
git commit -m "feat: register OAuth and resource binding routes"
```

---

## Task 8: State — Update OAuthService Initialization

**Files:**
- Modify: `src/state/mod.rs`

- [ ] **Step 1: Update OAuthService construction to use OAuthConfig**

Find the `oauth_service` initialization and update:

```rust
let oauth_config = {
    let config = shared_config.read().await;
    config.oauth.clone()
};
let oauth_service = OAuthService::new(db.clone(), oauth_config);
```

Update the `Clone` impl similarly.

- [ ] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -30`

- [ ] **Step 3: Commit**

```bash
git add src/state/mod.rs
git commit -m "feat: update AppState to initialize OAuthService with OAuthConfig"
```

---

## Task 9: Full Build Verification

- [ ] **Step 1: Run full cargo check**

Run: `cargo check 2>&1`
Expected: Compiles successfully (warnings OK)

- [ ] **Step 2: Run cargo test (existing tests)**

Run: `cargo test 2>&1 | tail -20`
Expected: All existing tests pass

- [ ] **Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: resolve compilation issues for Phase 3 OAuth"
```

---

## Task 10: Tests — Phase 3.1 OAuth App CRUD

**Files:**
- Create: `tests/phase3_1_oauth_apps_test.rs`

- [ ] **Step 1: Create test file**

```rust
use reqwest::Client;
use serde_json::{json, Value};

const BASE_URL: &str = "http://localhost:3000";

async fn register_and_login(client: &Client, email: &str, password: &str) -> String {
    // Register
    let _ = client.post(&format!("{}/api/v2/auth/register/send-code", BASE_URL))
        .json(&json!({"email": email}))
        .send().await;

    // Login with password (v1)
    let resp = client.post(&format!("{}/api/v1/auth/login", BASE_URL))
        .json(&json!({"email": email, "password": password}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    body["data"]["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_oauth_app() {
    let client = Client::new();
    let token = register_and_login(&client, "oauth_test1@example.com", "Test1234!").await;

    let resp = client.post(&format!("{}/api/v2/oauth/apps", BASE_URL))
        .bearer_auth(&token)
        .json(&json!({
            "name": "TestApp",
            "redirect_uris": ["https://example.com/callback"],
            "scopes": ["openid", "profile"]
        }))
        .send().await.unwrap();

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["name"].as_str().unwrap(), "TestApp");
    assert!(body["data"]["client_secret"].as_str().unwrap().starts_with("capella_sk_"));
}

#[tokio::test]
async fn test_list_oauth_apps() {
    let client = Client::new();
    let token = register_and_login(&client, "oauth_test2@example.com", "Test1234!").await;

    let resp = client.get(&format!("{}/api/v2/oauth/apps", BASE_URL))
        .bearer_auth(&token)
        .send().await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_non_owner_cannot_update_app() {
    let client = Client::new();
    let token1 = register_and_login(&client, "oauth_test3a@example.com", "Test1234!").await;
    let token2 = register_and_login(&client, "oauth_test3b@example.com", "Test1234!").await;

    // Create app with token1
    let resp = client.post(&format!("{}/api/v2/oauth/apps", BASE_URL))
        .bearer_auth(&token1)
        .json(&json!({"name": "App1", "redirect_uris": ["https://example.com"]}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    let app_id = body["data"]["id"].as_str().unwrap();

    // Try to update with token2
    let resp = client.put(&format!("{}/api/v2/oauth/apps/{}", BASE_URL, app_id))
        .bearer_auth(&token2)
        .json(&json!({"name": "Hacked"}))
        .send().await.unwrap();

    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn test_rotate_secret() {
    let client = Client::new();
    let token = register_and_login(&client, "oauth_test4@example.com", "Test1234!").await;

    // Create app
    let resp = client.post(&format!("{}/api/v2/oauth/apps", BASE_URL))
        .bearer_auth(&token)
        .json(&json!({"name": "SecretTest", "redirect_uris": ["https://example.com"]}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    let app_id = body["data"]["id"].as_str().unwrap();
    let old_secret = body["data"]["client_secret"].as_str().unwrap().to_string();

    // Rotate
    let resp = client.patch(&format!("{}/api/v2/oauth/apps/{}/secret", BASE_URL, app_id))
        .bearer_auth(&token)
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    let new_secret = body["data"]["client_secret"].as_str().unwrap().to_string();

    assert_ne!(old_secret, new_secret);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test phase3_1 -- --nocapture 2>&1 | tail -20`

- [ ] **Step 3: Commit**

```bash
git add tests/phase3_1_oauth_apps_test.rs
git commit -m "test: add Phase 3.1 OAuth app CRUD tests"
```

---

## Task 11: Tests — Phase 3.2 OAuth Flow

**Files:**
- Create: `tests/phase3_2_oauth_flow_test.rs`

- [ ] **Step 1: Create test file**

```rust
use reqwest::Client;
use serde_json::{json, Value};

const BASE_URL: &str = "http://localhost:3000";

#[tokio::test]
async fn test_complete_oauth_flow() {
    let client = Client::new();

    // 1. Register user
    let _ = client.post(&format!("{}/api/v2/auth/register/send-code", BASE_URL))
        .json(&json!({"email": "oauth_flow@example.com"}))
        .send().await;

    // 2. Create OAuth app
    let token = register_and_login(&client, "oauth_flow@example.com", "Test1234!").await;
    let resp = client.post(&format!("{}/api/v2/oauth/apps", BASE_URL))
        .bearer_auth(&token)
        .json(&json!({
            "name": "FlowTest",
            "redirect_uris": ["https://example.com/callback"],
            "scopes": ["openid", "profile", "email"]
        }))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    let app_id = body["data"]["id"].as_str().unwrap();
    let client_secret = body["data"]["client_secret"].as_str().unwrap();

    // 3. Simulate authorization (in real flow, this is browser-based)
    // For testing, we'll use client_credentials grant
    let resp = client.post(&format!("{}/oauth/token", BASE_URL))
        .json(&json!({
            "grant_type": "client_credentials",
            "client_id": app_id,
            "client_secret": client_secret
        }))
        .send().await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["token_type"].as_str().unwrap(), "Bearer");
    assert!(body["access_token"].as_str().is_some());
}

async fn register_and_login(client: &Client, email: &str, password: &str) -> String {
    let _ = client.post(&format!("{}/api/v2/auth/register/send-code", BASE_URL))
        .json(&json!({"email": email}))
        .send().await;

    let resp = client.post(&format!("{}/api/v1/auth/login", BASE_URL))
        .json(&json!({"email": email, "password": password}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    body["data"]["access_token"].as_str().unwrap().to_string()
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test phase3_2 -- --nocapture 2>&1 | tail -20`

- [ ] **Step 3: Commit**

```bash
git add tests/phase3_2_oauth_flow_test.rs
git commit -m "test: add Phase 3.2 OAuth flow tests"
```

---

## Task 12: Tests — Phase 3.5 Resource Bindings

**Files:**
- Create: `tests/phase3_5_resource_bindings_test.rs`

- [ ] **Step 1: Create test file**

```rust
use reqwest::Client;
use serde_json::{json, Value};

const BASE_URL: &str = "http://localhost:3000";

#[tokio::test]
async fn test_bind_and_lookup_resource() {
    let client = Client::new();
    let token = register_and_login(&client, "resource_test@example.com", "Test1234!").await;

    // Create room
    let resp = client.post(&format!("{}/api/v1/rooms", BASE_URL))
        .bearer_auth(&token)
        .json(&json!({"name": "TestRoom", "is_public": true}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    let room_id = body["data"]["id"].as_str().unwrap();

    // Create OAuth app
    let resp = client.post(&format!("{}/api/v2/oauth/apps", BASE_URL))
        .bearer_auth(&token)
        .json(&json!({"name": "ResTest", "redirect_uris": ["https://example.com"]}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    let app_id = body["data"]["id"].as_str().unwrap();

    // Bind resource
    let resp = client.post(&format!("{}/api/v2/rooms/{}/resources", BASE_URL, room_id))
        .bearer_auth(&token)
        .json(&json!({
            "app_id": app_id,
            "resource_type": "project",
            "resource_id": "proj-123",
            "resource_url": "https://example.com/projects/123",
            "resource_name": "My Project"
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);

    // Lookup
    let resp = client.get(&format!("{}/api/v2/rooms/resources/lookup?app_id={}&resource_type=project&resource_id=proj-123", BASE_URL, app_id))
        .bearer_auth(&token)
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["room_id"].as_str().unwrap(), room_id);
}

async fn register_and_login(client: &Client, email: &str, password: &str) -> String {
    let _ = client.post(&format!("{}/api/v2/auth/register/send-code", BASE_URL))
        .json(&json!({"email": email}))
        .send().await;

    let resp = client.post(&format!("{}/api/v1/auth/login", BASE_URL))
        .json(&json!({"email": email, "password": password}))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    body["data"]["access_token"].as_str().unwrap().to_string()
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test phase3_5 -- --nocapture 2>&1 | tail -20`

- [ ] **Step 3: Commit**

```bash
git add tests/phase3_5_resource_bindings_test.rs
git commit -m "test: add Phase 3.5 resource binding tests"
```

---

## Task 13: Final Verification

- [ ] **Step 1: Run full test suite**

Run: `cargo test 2>&1 | tail -30`
Expected: All tests pass

- [ ] **Step 2: Run cargo clippy for lint**

Run: `cargo clippy 2>&1 | head -40`
Expected: No errors (warnings acceptable)

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete Phase 3.1+3.2+3.5 OAuth and resource bindings"
```

---

*Plan version: 1.0.0*
*Created: 2026-06-19*
