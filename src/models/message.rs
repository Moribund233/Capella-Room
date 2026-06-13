use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::models::message_reaction::ReactionSummary;

/// 消息数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Message {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<Uuid>,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub edit_count: i32,
    pub edited_at: Option<DateTime<Utc>>,
}

/// 消息类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "message_type", rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    File,
    System,
}

/// 创建消息请求（用于服务层）
#[derive(Debug, Clone)]
pub struct CreateMessageRequest {
    pub room_id: Uuid,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<Uuid>,
}

/// 发送消息请求（HTTP API）
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SendMessageRequest {
    #[validate(length(min = 1, max = 2000, message = "消息内容长度必须在1-2000个字符之间"))]
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<Uuid>,
}

/// 编辑消息请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct EditMessageRequest {
    #[validate(length(min = 1, max = 2000, message = "消息内容长度必须在1-2000个字符之间"))]
    pub content: String,
}

/// 消息编辑记录
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct MessageEdit {
    pub id: Uuid,
    pub message_id: Uuid,
    pub editor_id: Uuid,
    pub old_content: String,
    pub new_content: String,
    pub created_at: DateTime<Utc>,
}

/// 消息编辑记录响应
#[derive(Debug, Clone, Serialize)]
pub struct MessageEditResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub editor: SenderInfo,
    pub old_content: String,
    pub new_content: String,
    pub created_at: DateTime<Utc>,
}

/// 被引用消息的信息
#[derive(Debug, Clone, Serialize)]
pub struct ReplyToInfo {
    pub id: Uuid,
    pub sender: SenderInfo,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// 消息响应
#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender: SenderInfo,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message: Option<ReplyToInfo>,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub edit_count: i32,
    pub edited_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reactions: Option<Vec<ReactionSummary>>,
}

/// 发送者信息
#[derive(Debug, Clone, Serialize)]
pub struct SenderInfo {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
}

impl Message {
    /// 转换为响应DTO
    pub fn to_response(&self, sender: SenderInfo) -> MessageResponse {
        MessageResponse {
            id: self.id,
            room_id: self.room_id,
            sender,
            content: self.content.clone(),
            message_type: self.message_type.clone(),
            reply_to: self.reply_to,
            reply_to_message: None,
            is_deleted: self.is_deleted,
            created_at: self.created_at,
            edit_count: self.edit_count,
            edited_at: self.edited_at,
            reactions: None,
        }
    }

    /// 转换为响应DTO，包含引用消息信息
    pub fn to_response_with_reply(
        &self,
        sender: SenderInfo,
        reply_to_message: Option<ReplyToInfo>,
    ) -> MessageResponse {
        MessageResponse {
            id: self.id,
            room_id: self.room_id,
            sender,
            content: self.content.clone(),
            message_type: self.message_type.clone(),
            reply_to: self.reply_to,
            reply_to_message,
            is_deleted: self.is_deleted,
            created_at: self.created_at,
            edit_count: self.edit_count,
            edited_at: self.edited_at,
            reactions: None,
        }
    }

    /// 转换为响应DTO，包含反应信息
    pub fn to_response_with_reactions(
        &self,
        sender: SenderInfo,
        reactions: Option<Vec<ReactionSummary>>,
    ) -> MessageResponse {
        MessageResponse {
            id: self.id,
            room_id: self.room_id,
            sender,
            content: self.content.clone(),
            message_type: self.message_type.clone(),
            reply_to: self.reply_to,
            reply_to_message: None,
            is_deleted: self.is_deleted,
            created_at: self.created_at,
            edit_count: self.edit_count,
            edited_at: self.edited_at,
            reactions,
        }
    }

    /// 检查是否是系统消息
    pub fn is_system(&self) -> bool {
        matches!(self.message_type, MessageType::System)
    }

    /// 检查是否已被删除
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// 获取显示内容（如果已删除则返回提示）
    pub fn display_content(&self) -> String {
        if self.is_deleted {
            "[此消息已被删除]".to_string()
        } else {
            self.content.clone()
        }
    }
}

impl SenderInfo {
    /// 创建发送者信息
    pub fn new(id: Uuid, username: String, avatar_url: Option<String>) -> Self {
        Self {
            id,
            username,
            avatar_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_serialization() {
        let msg_type = MessageType::Text;
        let json = serde_json::to_string(&msg_type).unwrap();
        assert_eq!(json, "\"text\"");
    }

    #[test]
    fn test_send_message_request_validation() {
        let valid = SendMessageRequest {
            content: "Hello, World!".to_string(),
            message_type: MessageType::Text,
            reply_to: None,
        };
        assert!(valid.validate().is_ok());

        let empty_content = SendMessageRequest {
            content: "".to_string(),
            message_type: MessageType::Text,
            reply_to: None,
        };
        assert!(empty_content.validate().is_err());
    }

    #[test]
    fn test_message_display_content() {
        let msg = Message {
            id: Uuid::new_v4(),
            room_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            content: "Test content".to_string(),
            message_type: MessageType::Text,
            reply_to: None,
            is_deleted: false,
            created_at: Utc::now(),
            edit_count: 0,
            edited_at: None,
        };
        assert_eq!(msg.display_content(), "Test content");

        let deleted_msg = Message {
            is_deleted: true,
            ..msg
        };
        assert_eq!(deleted_msg.display_content(), "[此消息已被删除]");
    }
}
