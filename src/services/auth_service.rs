use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    config::JwtConfig,
    error::{AppError, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
    pub role: crate::models::user::UserRole,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Clone)]
pub struct AuthService {
    pub jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(jwt_config: JwtConfig) -> Self {
        Self { jwt_config }
    }

    fn get_secret(&self) -> Result<&str> {
        self.jwt_config
            .secret
            .as_deref()
            .ok_or_else(|| AppError::Auth("JWT secret is not configured".to_string()))
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Auth(format!("密码哈希失败: {}", e)))?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Auth(format!("密码哈希解析失败: {}", e)))?;

        let argon2 = Argon2::default();
        let result = argon2.verify_password(password.as_bytes(), &parsed_hash);

        match result {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AppError::Auth(format!("密码验证失败: {}", e))),
        }
    }

    pub fn generate_token_pair(
        &self,
        user_id: Uuid,
        username: &str,
        role: crate::models::user::UserRole,
    ) -> Result<TokenPair> {
        let access_token = self.generate_access_token(user_id, username, role.clone())?;
        let refresh_token = self.generate_refresh_token(user_id, role)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.jwt_config.expiration_hours * 3600,
        })
    }

    fn generate_access_token(
        &self,
        user_id: Uuid,
        username: &str,
        role: crate::models::user::UserRole,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::Auth("系统时间错误".to_string()))?;

        let iat = now.as_secs() as usize;
        let exp = iat + (self.jwt_config.expiration_hours as usize * 3600);

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
            iat,
            token_type: "access".to_string(),
            role,
            username: Some(username.to_string()),
        };

        let secret = self.get_secret()?;
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| AppError::Auth(format!("Token生成失败: {}", e)))?;

        Ok(token)
    }

    fn generate_refresh_token(
        &self,
        user_id: Uuid,
        role: crate::models::user::UserRole,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::Auth("系统时间错误".to_string()))?;

        let iat = now.as_secs() as usize;
        let exp = iat + (self.jwt_config.expiration_hours as usize * 3600 * 7);

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
            iat,
            token_type: "refresh".to_string(),
            role,
            username: None,
        };

        let secret = self.get_secret()?;
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| AppError::Auth(format!("刷新Token生成失败: {}", e)))?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let secret = self.get_secret()?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AppError::Auth("Token已过期".to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                AppError::Auth("Token签名无效".to_string())
            }
            _ => AppError::Auth(format!("Token验证失败: {}", e)),
        })?;

        Ok(token_data.claims)
    }

    pub fn verify_access_token(&self, token: &str) -> Result<Claims> {
        let claims = self.verify_token(token)?;

        if claims.token_type != "access" {
            return Err(AppError::Auth("无效的访问令牌".to_string()));
        }

        Ok(claims)
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<Claims> {
        let claims = self.verify_token(token)?;

        if claims.token_type != "refresh" {
            return Err(AppError::Auth("无效的刷新令牌".to_string()));
        }

        Ok(claims)
    }

    pub fn extract_user_id(&self, claims: &Claims) -> Result<Uuid> {
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))
    }
}
