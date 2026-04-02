use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::utils::validation::{validate_username, validate_password_strength, validate_email_format};

/// 用户数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户状态
#[derive(Debug, Clone, Serialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Online,
    Offline,
    Away,
}

/// 用户注册请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(custom(function = "validate_username", message = "用户名格式不正确"))]
    pub username: String,

    #[validate(custom(function = "validate_email_format", message = "邮箱格式不正确"))]
    pub email: String,

    #[validate(custom(function = "validate_password_strength", message = "密码强度不足"))]
    pub password: String,
}

/// 用户登录请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
    pub password: String,
}

/// 创建用户请求（用于服务层）
#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

/// 更新用户请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 20))]
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

/// 用户信息响应
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
}

impl User {
    /// 转换为响应DTO
    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            avatar_url: self.avatar_url.clone(),
            status: self.status.clone(),
        }
    }
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        user.to_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_status_serialization() {
        let status = UserStatus::Online;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"online\"");
    }

    #[test]
    fn test_register_request_validation() {
        let valid = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "Password123".to_string(), // 包含大写、小写和数字
        };
        assert!(valid.validate().is_ok());

        let invalid_email = RegisterRequest {
            username: "test".to_string(),
            email: "invalid-email".to_string(),
            password: "Password123".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        // 测试密码强度不足
        let weak_password = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(), // 缺少大写字母
        };
        assert!(weak_password.validate().is_err());
    }
}
