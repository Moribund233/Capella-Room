use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::OAuthConfig,
    db::Database,
    error::{AppError, Result},
    models::oauth::*,
};

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

impl OAuthService {
    pub fn new(db: Database, config: OAuthConfig) -> Self {
        Self { db, config }
    }

    // ═══════════════════════════════════════════════
    // App Management
    // ═══════════════════════════════════════════════

    pub async fn register_app(
        &self,
        owner_id: Uuid,
        name: &str,
        description: Option<&str>,
        redirect_uris: &[&str],
        scopes: &[&str],
    ) -> Result<OAuthApp> {
        let client_secret = Self::generate_client_secret();
        let secret_hash = hash(&client_secret, DEFAULT_COST).map_err(|e| {
            tracing::error!("bcrypt hash failed: {}", e);
            AppError::Internal
        })?;

        let redirect_uris_vec: Vec<String> = redirect_uris.iter().map(|s| s.to_string()).collect();
        let scopes_vec: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();

        let mut app = sqlx::query_as::<_, OAuthApp>(
            r#"INSERT INTO oauth_apps (name, description, client_secret, redirect_uris, scopes, owner_id)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, name, description, client_secret, redirect_uris, scopes, owner_id, is_active, created_at, updated_at"#,
        )
        .bind(name)
        .bind(description)
        .bind(&secret_hash)
        .bind(&redirect_uris_vec)
        .bind(&scopes_vec)
        .bind(owner_id)
        .fetch_one(self.db.pool())
        .await?;

        app.client_secret = client_secret;
        Ok(app)
    }

    pub async fn get_app(&self, app_id: Uuid) -> Result<OAuthApp> {
        sqlx::query_as::<_, OAuthApp>(
            r#"SELECT id, name, description, client_secret, redirect_uris, scopes, owner_id, is_active, created_at, updated_at
               FROM oauth_apps WHERE id = $1"#,
        )
        .bind(app_id)
        .fetch_optional(self.db.pool())
        .await?
        .ok_or(AppError::NotFound)
    }

    pub async fn list_apps(&self, owner_id: Uuid) -> Result<Vec<OAuthApp>> {
        let apps = sqlx::query_as::<_, OAuthApp>(
            r#"SELECT id, name, description, client_secret, redirect_uris, scopes, owner_id, is_active, created_at, updated_at
               FROM oauth_apps WHERE owner_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(owner_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(apps)
    }

    pub async fn update_app(
        &self,
        app_id: Uuid,
        owner_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        redirect_uris: Option<&[&str]>,
        scopes: Option<&[&str]>,
    ) -> Result<OAuthApp> {
        let app = self.get_owned_app(app_id, owner_id).await?;

        let new_name = name.unwrap_or(&app.name);
        let new_description = match description {
            Some(d) => Some(d.to_string()),
            None => app.description.clone(),
        };
        let new_redirect_uris: Vec<String> = redirect_uris
            .map(|r| r.iter().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| app.redirect_uris.clone());
        let new_scopes: Vec<String> = scopes
            .map(|s| s.iter().map(|x| x.to_string()).collect())
            .unwrap_or_else(|| app.scopes.clone());

        sqlx::query_as::<_, OAuthApp>(
            r#"UPDATE oauth_apps
               SET name = $1, description = $2, redirect_uris = $3, scopes = $4, updated_at = NOW()
               WHERE id = $5
               RETURNING id, name, description, client_secret, redirect_uris, scopes, owner_id, is_active, created_at, updated_at"#,
        )
        .bind(new_name)
        .bind(new_description)
        .bind(&new_redirect_uris)
        .bind(&new_scopes)
        .bind(app_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(Into::into)
    }

    pub async fn delete_app(&self, app_id: Uuid, owner_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM oauth_apps WHERE id = $1 AND owner_id = $2")
            .bind(app_id)
            .bind(owner_id)
            .execute(self.db.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn rotate_secret(&self, app_id: Uuid, owner_id: Uuid) -> Result<String> {
        let new_secret = Self::generate_client_secret();
        let secret_hash = hash(&new_secret, DEFAULT_COST).map_err(|e| {
            tracing::error!("bcrypt hash failed: {}", e);
            AppError::Internal
        })?;
        let result = sqlx::query(
            "UPDATE oauth_apps SET client_secret = $1, updated_at = NOW() WHERE id = $2 AND owner_id = $3",
        )
        .bind(&secret_hash)
        .bind(app_id)
        .bind(owner_id)
        .execute(self.db.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(new_secret)
    }

    async fn get_owned_app(&self, app_id: Uuid, owner_id: Uuid) -> Result<OAuthApp> {
        sqlx::query_as::<_, OAuthApp>(
            r#"SELECT id, name, description, client_secret, redirect_uris, scopes, owner_id, is_active, created_at, updated_at
               FROM oauth_apps WHERE id = $1 AND owner_id = $2"#,
        )
        .bind(app_id)
        .bind(owner_id)
        .fetch_optional(self.db.pool())
        .await?
        .ok_or(AppError::NotFound)
    }

    pub fn verify_client_secret(secret: &str, hash: &str) -> bool {
        verify(secret, hash).unwrap_or(false)
    }

    // ═══════════════════════════════════════════════
    // Authorization Code
    // ═══════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════
    // Token
    // ═══════════════════════════════════════════════

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

        let scopes = app.scopes.clone();
        self.generate_tokens(app.owner_id, client_id, &scopes).await
    }

    pub fn verify_access_token(&self, token: &str) -> Result<OAuthClaims> {
        let secret = self.config.jwt_secret.as_deref()
            .ok_or_else(|| AppError::Auth("OAuth JWT secret not configured".to_string()))?;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_aud = false;
        let token_data = decode::<OAuthClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        ).map_err(|e| AppError::Auth(format!("invalid_token: {}", e)))?;

        Ok(token_data.claims)
    }

    // ═══════════════════════════════════════════════
    // UserInfo
    // ═══════════════════════════════════════════════

    pub async fn get_user_info(&self, user_id: Uuid) -> Result<UserInfoResponse> {
        let user = sqlx::query_as::<_, crate::models::user::User>(
            r#"SELECT id, username, email, password_hash, role, status, is_active, avatar_url, created_at, updated_at, email_verified, email_verified_at
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

    // ═══════════════════════════════════════════════
    // Identity Mappings
    // ═══════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════
    // Room Resource Bindings
    // ═══════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════
    // Auth Session Token (for login flow)
    // ═══════════════════════════════════════════════

    pub fn create_auth_session_token(&self, user_id: Uuid, client_id: Uuid) -> Result<String> {
        let exp = (Utc::now() + chrono::Duration::minutes(5)).timestamp() as u64;
        let token = AuthSessionToken {
            user_id,
            client_id,
            exp,
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

        if token_data.claims.exp < Utc::now().timestamp() as u64 {
            return Err(AppError::Auth("Auth session expired".to_string()));
        }

        Ok(token_data.claims)
    }

    // ═══════════════════════════════════════════════
    // Internal Helpers
    // ═══════════════════════════════════════════════

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

    fn generate_client_secret() -> String {
        use rand::Rng;
        let bytes: [u8; 32] = rand::thread_rng().gen();
        format!("capella_sk_{}", hex::encode(bytes))
    }

    fn hash_secret(secret: &str) -> String {
        hash(secret, DEFAULT_COST).unwrap_or_default()
    }
}
