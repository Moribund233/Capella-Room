use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    config::{JwtConfig, SharedConfig},
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
    jwt_config: JwtConfig,
    shared_config: Option<SharedConfig>,
}

impl AuthService {
    pub fn new(jwt_config: JwtConfig) -> Self {
        Self {
            jwt_config,
            shared_config: None,
        }
    }

    /// 使用共享配置创建 AuthService
    /// 注意：不要在异步运行时中调用 blocking_read，这里直接存储配置引用
    pub fn with_shared_config(config: SharedConfig) -> Self {
        // 先使用默认配置，后续通过 shared_config 动态读取
        let jwt_config = JwtConfig::default();
        Self {
            jwt_config,
            shared_config: Some(config),
        }
    }

    /// 获取 JWT secret
    /// 优先从 shared_config 读取（支持热加载），否则使用静态配置
    fn get_secret(&self) -> Result<String> {
        if let Some(config) = &self.shared_config {
            // 使用 try_read 避免阻塞，如果获取不到锁则使用默认配置
            match config.try_read() {
                Ok(cfg) => cfg
                    .jwt
                    .secret
                    .clone()
                    .ok_or_else(|| AppError::Auth("JWT secret is not configured".to_string())),
                Err(_) => self
                    .jwt_config
                    .secret
                    .clone()
                    .ok_or_else(|| AppError::Auth("JWT secret is not configured".to_string())),
            }
        } else {
            self.jwt_config
                .secret
                .clone()
                .ok_or_else(|| AppError::Auth("JWT secret is not configured".to_string()))
        }
    }

    /// 获取过期时间（小时）
    fn get_expiration_hours(&self) -> i64 {
        if let Some(config) = &self.shared_config {
            // 使用 try_read 避免阻塞
            match config.try_read() {
                Ok(cfg) => cfg.jwt.expiration_hours,
                Err(_) => self.jwt_config.expiration_hours,
            }
        } else {
            self.jwt_config.expiration_hours
        }
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
            expires_in: self.get_expiration_hours() * 3600,
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
        let exp = iat + (self.get_expiration_hours() as usize * 3600);

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
        let exp = iat + (self.get_expiration_hours() as usize * 3600 * 7);

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

    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<String> {
        let claims = self.verify_token(refresh_token)?;

        if claims.token_type != "refresh" {
            return Err(AppError::Auth("无效的刷新Token".to_string()));
        }

        let user_id =
            Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

        self.generate_access_token(
            user_id,
            claims.username.as_deref().unwrap_or(""),
            claims.role,
        )
    }

    /// 验证访问令牌
    /// 与 verify_token 相同，但专门用于访问令牌验证
    pub fn verify_access_token(&self, token: &str) -> Result<Claims> {
        let claims = self.verify_token(token)?;
        if claims.token_type != "access" {
            return Err(AppError::Auth("无效的访问令牌类型".to_string()));
        }
        Ok(claims)
    }

    /// 验证刷新令牌
    pub fn verify_refresh_token(&self, token: &str) -> Result<Claims> {
        let claims = self.verify_token(token)?;
        if claims.token_type != "refresh" {
            return Err(AppError::Auth("无效的刷新令牌类型".to_string()));
        }
        Ok(claims)
    }

    /// 从 Claims 中提取用户 ID
    pub fn extract_user_id(&self, claims: &Claims) -> Result<Uuid> {
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_auth_service() -> AuthService {
        let jwt_config = JwtConfig {
            secret: Some("test_secret_key_for_testing_purposes_only".to_string()),
            expiration_hours: 24,
        };
        AuthService::new(jwt_config)
    }

    #[test]
    fn test_password_hash_and_verify() {
        let auth = create_test_auth_service();
        let password = "test_password123";

        let hash = auth.hash_password(password).unwrap();
        assert!(!hash.is_empty());

        let is_valid = auth.verify_password(password, &hash).unwrap();
        assert!(is_valid);

        let is_invalid = auth.verify_password("wrong_password", &hash).unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_token_generation_and_verification() {
        let auth = create_test_auth_service();
        let user_id = Uuid::new_v4();
        let username = "test_user";
        let role = crate::models::user::UserRole::User;

        let token_pair = auth
            .generate_token_pair(user_id, username, role.clone())
            .unwrap();

        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.expires_in, 24 * 3600);

        let claims = auth.verify_token(&token_pair.access_token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, Some(username.to_string()));
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_refresh_token() {
        let auth = create_test_auth_service();
        let user_id = Uuid::new_v4();
        let username = "test_user";
        let role = crate::models::user::UserRole::User;

        let token_pair = auth.generate_token_pair(user_id, username, role).unwrap();
        let new_access_token = auth
            .refresh_access_token(&token_pair.refresh_token)
            .unwrap();

        assert!(!new_access_token.is_empty());

        let claims = auth.verify_token(&new_access_token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "access");
    }
}
