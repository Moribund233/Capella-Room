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
    pub room_type: RoomType,
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

/// 房间类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "room_type", rename_all = "lowercase")]
pub enum RoomType {
    Group,  // 群聊
    Direct, // 私聊（1对1）
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        members
            .iter()
            .any(|m| m.room_id == self.id && m.user_id == user_id)
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

/// 房间邀请
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RoomInvitation {
    pub id: Uuid,
    pub room_id: Uuid,
    pub inviter_id: Uuid,
    pub invite_code: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// 房间邀请响应（包含邀请者信息）
#[derive(Debug, Clone, Serialize)]
pub struct RoomInvitationResponse {
    pub id: Uuid,
    pub room_id: Uuid,
    pub inviter: UserInfo,
    pub invite_code: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// 创建房间邀请请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateInvitationRequest {
    /// 邀请码有效期（小时），null表示永不过期
    pub expires_in_hours: Option<i32>,
    /// 最大使用次数，null表示无限制
    #[validate(range(min = 1, max = 1000, message = "最大使用次数必须在1-1000之间"))]
    pub max_uses: Option<i32>,
}

/// 通过邀请码加入房间请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct JoinByInviteRequest {
    #[validate(length(min = 1, max = 20, message = "邀请码长度必须在1-20个字符之间"))]
    pub invite_code: String,
}

/// 创建私聊房间请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateDirectRoomRequest {
    /// 对方用户ID
    pub target_user_id: Uuid,
}

/// 私聊房间响应（包含对方用户信息）
/// 
/// 注意：name 字段动态生成，始终为目标用户的最新用户名
/// 不依赖数据库中存储的房间名称，确保用户改名后房间名同步更新
#[derive(Debug, Clone, Serialize)]
pub struct DirectRoomResponse {
    pub id: Uuid,
    pub name: String,
    pub target_user: UserInfo,
    pub created_at: DateTime<Utc>,
}

impl RoomInvitation {
    /// 检查邀请是否有效
    pub fn is_valid(&self) -> bool {
        if !self.is_active {
            return false;
        }

        // 检查是否过期
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        // 检查使用次数
        if let Some(max_uses) = self.max_uses {
            if self.used_count >= max_uses {
                return false;
            }
        }

        true
    }

    /// 转换为响应DTO
    pub fn to_response(&self, inviter: UserInfo) -> RoomInvitationResponse {
        RoomInvitationResponse {
            id: self.id,
            room_id: self.room_id,
            inviter,
            invite_code: self.invite_code.clone(),
            expires_at: self.expires_at,
            max_uses: self.max_uses,
            used_count: self.used_count,
            is_active: self.is_active,
            created_at: self.created_at,
        }
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

    #[test]
    fn test_room_invitation_valid() {
        let invitation = RoomInvitation {
            id: Uuid::new_v4(),
            room_id: Uuid::new_v4(),
            inviter_id: Uuid::new_v4(),
            invite_code: "ABC123".to_string(),
            expires_at: None,
            max_uses: None,
            used_count: 0,
            is_active: true,
            created_at: Utc::now(),
        };
        assert!(invitation.is_valid());

        // 测试已停用
        let mut inactive = invitation.clone();
        inactive.is_active = false;
        assert!(!inactive.is_valid());

        // 测试已过期
        let mut expired = invitation.clone();
        expired.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!expired.is_valid());

        // 测试使用次数已满
        let mut max_used = invitation.clone();
        max_used.max_uses = Some(5);
        max_used.used_count = 5;
        assert!(!max_used.is_valid());
    }

    #[test]
    fn test_room_type_serialization() {
        let group = RoomType::Group;
        let direct = RoomType::Direct;
        assert_eq!(serde_json::to_string(&group).unwrap(), "\"group\"");
        assert_eq!(serde_json::to_string(&direct).unwrap(), "\"direct\"");
    }
}
