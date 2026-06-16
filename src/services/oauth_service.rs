use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::{
    db::Database,
    error::{AppError, Result},
    models::oauth::OAuthApp,
};

#[derive(Clone)]
pub struct OAuthService {
    db: Database,
    _jwt_secret: String,
}

impl OAuthService {
    pub fn new(db: Database, jwt_secret: &str) -> Self {
        Self {
            db,
            _jwt_secret: jwt_secret.to_string(),
        }
    }

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

    /// Fetch app and verify ownership in one query
    async fn get_owned_app(&self, app_id: Uuid, owner_id: Uuid) -> Result<OAuthApp> {
        sqlx::query_as::<_, OAuthApp>(
            r#"SELECT id, name, description, client_secret, redirect_uris, scopes, owner_id, is_active, created_at, updated_at
               FROM oauth_apps WHERE id = $1 AND owner_id = $2"#
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

    fn generate_client_secret() -> String {
        use rand::Rng;
        let bytes: [u8; 32] = rand::thread_rng().gen();
        format!("capella_sk_{}", hex::encode(bytes))
    }

    fn _generate_authorization_code() -> String {
        use rand::Rng;
        let bytes: [u8; 32] = rand::thread_rng().gen();
        hex::encode(bytes)
    }
}
