use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::models::user::UserInfo;

/// 聊天室数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub is_private: bool,
    pub max_members: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 聊天室成员模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RoomMember {
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

/// 成员角色
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "member_role", rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

/// 创建聊天室请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateRoomRequest {
    #[validate(length(min = 1, max = 50, message = "聊天室名称长度必须在1-50个字符之间"))]
    pub name: String,
    #[validate(length(max = 200, message = "聊天室描述不能超过200个字符"))]
    pub description: Option<String>,
    pub is_private: bool,
    #[validate(range(min = 2, max = 1000, message = "成员数量限制必须在2-1000之间"))]
    pub max_members: Option<i32>,
}

/// 更新聊天室请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateRoomRequest {
    #[validate(length(min = 1, max = 50, message = "聊天室名称长度必须在1-50个字符之间"))]
    pub name: Option<String>,
    #[validate(length(max = 200, message = "聊天室描述不能超过200个字符"))]
    pub description: Option<String>,
    pub is_private: Option<bool>,
    #[validate(range(min = 2, max = 1000, message = "成员数量限制必须在2-1000之间"))]
    pub max_members: Option<i32>,
}

/// 聊天室响应
#[derive(Debug, Clone, Serialize)]
pub struct RoomResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner: UserInfo,
    pub is_private: bool,
    pub max_members: i32,
    pub member_count: i64,
    pub last_message: Option<MessagePreview>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 消息预览（用于房间列表中的最后消息）
#[derive(Debug, Clone, Serialize)]
pub struct MessagePreview {
    pub id: Uuid,
    pub content: String,
    pub sender_name: String,
    pub created_at: DateTime<Utc>,
}

impl Room {
    /// 转换为响应DTO（需要传入所有者信息）
    pub fn to_response(&self, member_count: i64, owner: UserInfo) -> RoomResponse {
        RoomResponse {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            owner,
            is_private: self.is_private,
            max_members: self.max_members,
            member_count,
            last_message: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    /// 检查用户是否是房间成员
    pub fn is_member(&self, user_id: Uuid, members: &[RoomMember]) -> bool {
        members.iter().any(|m| m.room_id == self.id && m.user_id == user_id)
    }

    /// 检查用户是否是房间所有者
    pub fn is_owner(&self, user_id: Uuid) -> bool {
        self.owner_id == user_id
    }
}

impl RoomMember {
    /// 检查是否是管理员或所有者
    pub fn is_admin_or_owner(&self) -> bool {
        matches!(self.role, MemberRole::Owner | MemberRole::Admin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_role_serialization() {
        let role = MemberRole::Owner;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"owner\"");
    }

    #[test]
    fn test_create_room_request_validation() {
        let valid = CreateRoomRequest {
            name: "Test Room".to_string(),
            description: Some("A test room".to_string()),
            is_private: false,
            max_members: Some(50),
        };
        assert!(valid.validate().is_ok());

        let invalid_name = CreateRoomRequest {
            name: "".to_string(),
            description: None,
            is_private: false,
            max_members: None,
        };
        assert!(invalid_name.validate().is_err());
    }
}
