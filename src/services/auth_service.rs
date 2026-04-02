use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    config::JwtConfig,
    error::{AppError, Result},
};

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,  // 用户ID
    pub exp: usize, // 过期时间
    pub iat: usize, // 签发时间
}

/// 认证服务
pub struct AuthService {
    jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(jwt_config: JwtConfig) -> Self {
        Self { jwt_config }
    }
    
    /// 密码哈希
    /// TODO: 实现密码哈希
    pub fn hash_password(&self, password: &str) -> Result<String> {
        todo!("实现密码哈希")
    }
    
    /// 验证密码
    /// TODO: 实现密码验证
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        todo!("实现密码验证")
    }
    
    /// 生成JWT Token
    /// TODO: 实现Token生成
    pub fn generate_token(&self, user_id: Uuid) -> Result<String> {
        todo!("实现Token生成")
    }
    
    /// 验证JWT Token
    /// TODO: 实现Token验证
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        todo!("实现Token验证")
    }
}
