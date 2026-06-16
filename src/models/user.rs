use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use uuid::Uuid;
use validator::Validate;

use crate::utils::validation::{
    validate_email_format, validate_password_strength, validate_username,
};

/// 用户角色
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    #[default]
    User,
    Admin,
    SuperAdmin,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UserRole::User => "user",
            UserRole::Admin => "admin",
            UserRole::SuperAdmin => "super_admin",
        };
        write!(f, "{}", s)
    }
}

impl UserRole {
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::SuperAdmin)
    }

    pub fn is_super_admin(&self) -> bool {
        matches!(self, UserRole::SuperAdmin)
    }

    pub fn can_manage_user(&self, target_role: &UserRole) -> bool {
        match self {
            UserRole::SuperAdmin => true,
            UserRole::Admin => !target_role.is_admin(),
            UserRole::User => false,
        }
    }
}

/// 用户数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub avatar_url: Option<String>,
    /// 在线状态：online/offline/away
    pub status: UserStatus,
    /// 账号状态：true=启用, false=禁用
    pub is_active: bool,
    pub role: UserRole,
    #[sqlx(default)]
    pub email_verified: bool,
    #[sqlx(default)]
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户在线状态
#[derive(Debug, Clone, Serialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Online,
    Offline,
    Away,
}

impl UserStatus {
    /// 检查用户是否在线
    pub fn is_online(&self) -> bool {
        matches!(self, UserStatus::Online)
    }
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
    /// 设备名称（如 "iPhone 15", "Windows PC"）
    pub device_name: Option<String>,
    /// 设备类型（mobile, tablet, desktop, unknown）
    pub device_type: Option<String>,
    /// User-Agent 字符串
    pub user_agent: Option<String>,
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

/// 修改密码请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    #[validate(custom(function = "validate_password_strength", message = "密码强度不足"))]
    pub new_password: String,
}

/// 用户信息响应
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    /// 在线状态：online/offline/away
    pub status: UserStatus,
    /// 账号状态：true=启用, false=禁用
    pub is_active: bool,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

/// 简化用户信息（用于嵌套在其他响应中）
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
}

impl UserInfo {
    /// 创建新的 UserInfo
    pub fn new(id: Uuid, username: String, avatar_url: Option<String>) -> Self {
        Self {
            id,
            username,
            avatar_url,
        }
    }

    /// 从 User 创建 UserInfo
    pub fn from_user(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            avatar_url: user.avatar_url.clone(),
        }
    }
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
            is_active: self.is_active,
            role: self.role.clone(),
            created_at: self.created_at,
        }
    }

    /// 检查用户账号是否可用（未被禁用）
    pub fn is_account_active(&self) -> bool {
        self.is_active
    }

    /// 检查用户账号是否被禁用
    pub fn is_account_disabled(&self) -> bool {
        !self.is_active
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

// ==================== 好友功能模型 ====================

/// 好友申请状态
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "friend_request_status", rename_all = "lowercase")]
pub enum FriendRequestStatus {
    Pending,   // 待处理
    Accepted,  // 已接受
    Rejected,  // 已拒绝
    Cancelled, // 已取消
}

/// 好友关系
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Friendship {
    pub id: Uuid,
    pub user_id_a: Uuid,
    pub user_id_b: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 好友申请
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct FriendRequest {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub status: FriendRequestStatus,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 好友申请响应（包含发送者信息）
#[derive(Debug, Clone, Serialize)]
pub struct FriendRequestResponse {
    pub id: Uuid,
    pub sender: UserInfo,
    pub status: FriendRequestStatus,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 好友响应（包含好友信息）
#[derive(Debug, Clone, Serialize)]
pub struct FriendResponse {
    pub id: Uuid,
    pub friend: UserInfo,
    pub created_at: DateTime<Utc>,
}

/// 发送好友申请请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SendFriendRequest {
    /// 目标用户ID
    pub target_user_id: Uuid,
    /// 附加消息（可选）
    #[validate(length(max = 200, message = "附加消息不能超过200个字符"))]
    pub message: Option<String>,
}

/// 处理好友申请请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct HandleFriendRequest {
    /// 申请ID
    pub request_id: Uuid,
    /// 是否接受
    pub accept: bool,
}

/// 搜索用户请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SearchUserRequest {
    #[validate(length(min = 1, max = 50, message = "搜索关键词长度必须在1-50个字符之间"))]
    pub keyword: String,
}
