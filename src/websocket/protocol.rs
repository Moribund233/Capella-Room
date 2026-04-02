use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WebSocket 消息类型
/// 使用 tag 字段进行反序列化分发
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WebSocketMessage {
    // ========== 连接管理 ==========
    /// 客户端连接认证
    Auth { token: String },
    
    /// 认证结果
    AuthResult { success: bool, message: String },
    
    /// 心跳 ping
    Ping,
    
    /// 心跳 pong
    Pong,
    
    /// 错误消息
    Error { code: String, message: String },
    
    // ========== 房间管理 ==========
    /// 加入房间
    JoinRoom { room_id: Uuid },
    
    /// 离开房间
    LeaveRoom { room_id: Uuid },
    
    /// 房间加入结果
    RoomJoined { room_id: Uuid, user_id: Uuid, username: String },
    
    /// 房间离开结果
    RoomLeft { room_id: Uuid, user_id: Uuid, username: String },
    
    /// 用户加入房间通知（广播给其他用户）
    UserJoined { room_id: Uuid, user_id: Uuid, username: String },
    
    /// 用户离开房间通知（广播给其他用户）
    UserLeft { room_id: Uuid, user_id: Uuid, username: String },
    
    /// 在线用户列表
    OnlineUsers { room_id: Uuid, users: Vec<UserInfo> },
    
    // ========== 消息通信 ==========
    /// 发送聊天消息
    ChatMessage {
        room_id: Uuid,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to: Option<Uuid>,
    },
    
    /// 收到聊天消息（广播）
    NewMessage {
        message_id: Uuid,
        room_id: Uuid,
        sender_id: Uuid,
        sender_name: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to: Option<Uuid>,
        created_at: DateTime<Utc>,
    },
    
    /// 正在输入状态
    Typing { room_id: Uuid },
    
    /// 停止输入状态
    StopTyping { room_id: Uuid },
    
    /// 消息已读确认
    MessageRead { message_id: Uuid },
    
    /// 消息已读回执
    MessageReadReceipt { message_id: Uuid, user_id: Uuid },
    
    /// 编辑消息
    EditMessage { message_id: Uuid, new_content: String },
    
    /// 消息已编辑通知
    MessageEdited { message_id: Uuid, new_content: String, edited_at: DateTime<Utc> },
    
    /// 删除消息
    DeleteMessage { message_id: Uuid },
    
    /// 消息已删除通知
    MessageDeleted { message_id: Uuid },
    
    // ========== 系统消息 ==========
    /// 系统广播
    SystemMessage { content: String },
    
    /// 房间信息更新
    RoomUpdated { room_id: Uuid, name: Option<String>, description: Option<String> },
    
    // ========== 用户状态 ==========
    /// 更新用户状态
    UpdateStatus { status: UserStatus },
    
    /// 用户状态变更通知
    UserStatusChanged { user_id: Uuid, username: String, status: UserStatus },
    
    /// 获取在线用户列表（全局）
    GetOnlineUsers,
    
    /// 全局在线用户列表
    GlobalOnlineUsers { users: Vec<UserInfo>, total: usize },
    
    // ========== 断线重连 ==========
    /// 重连请求（携带上次断开时间）
    Reconnect { 
        token: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_disconnect_at: Option<DateTime<Utc>>,
    },
    
    /// 重连结果
    ReconnectResult { 
        success: bool, 
        message: String,
        /// 需要重新加入的房间列表
        #[serde(skip_serializing_if = "Option::is_none")]
        rooms_to_rejoin: Option<Vec<Uuid>>,
    },
    
    /// 请求离线期间的消息
    GetMissedMessages { 
        room_id: Uuid,
        /// 上次接收的消息ID
        last_message_id: Option<Uuid>,
    },
    
    /// 离线消息列表
    MissedMessages { 
        room_id: Uuid,
        messages: Vec<MissedMessage>,
        /// 是否有更多消息
        has_more: bool,
    },
    
    /// 会话恢复完成
    SessionRestored { 
        restored_at: DateTime<Utc>,
        /// 恢复的房间数
        rooms_restored: usize,
        /// 恢复的未读消息数
        total_unread: usize,
    },
}

/// 离线消息结构（用于断线重连后同步消息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissedMessage {
    pub message_id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// 用户信息（用于在线列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub status: UserStatus,
}

/// 用户在线状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Online,
    Away,
    Busy,
    Offline,
}

impl WebSocketMessage {
    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string(self).map_err(|e| anyhow::anyhow!("Failed to serialize message: {}", e))
    }
    
    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).map_err(|e| anyhow::anyhow!("Failed to deserialize message: {}", e))
    }
    
    /// 创建成功认证响应
    pub fn auth_success() -> Self {
        Self::AuthResult {
            success: true,
            message: "Authentication successful".to_string(),
        }
    }
    
    /// 创建失败认证响应
    pub fn auth_failed(reason: &str) -> Self {
        Self::AuthResult {
            success: false,
            message: reason.to_string(),
        }
    }
    
    /// 创建错误消息
    pub fn error(code: &str, message: &str) -> Self {
        Self::Error {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
    
    /// 创建聊天消息
    pub fn new_chat_message(
        message_id: Uuid,
        room_id: Uuid,
        sender_id: Uuid,
        sender_name: &str,
        content: &str,
        reply_to: Option<Uuid>,
    ) -> Self {
        Self::NewMessage {
            message_id,
            room_id,
            sender_id,
            sender_name: sender_name.to_string(),
            content: content.to_string(),
            reply_to,
            created_at: Utc::now(),
        }
    }
    
    /// 创建重连成功响应
    pub fn reconnect_success(rooms_to_rejoin: Vec<Uuid>) -> Self {
        Self::ReconnectResult {
            success: true,
            message: "Reconnected successfully".to_string(),
            rooms_to_rejoin: Some(rooms_to_rejoin),
        }
    }
    
    /// 创建重连失败响应
    pub fn reconnect_failed(reason: &str) -> Self {
        Self::ReconnectResult {
            success: false,
            message: reason.to_string(),
            rooms_to_rejoin: None,
        }
    }
    
    /// 创建会话恢复完成消息
    pub fn session_restored(rooms_restored: usize, total_unread: usize) -> Self {
        Self::SessionRestored {
            restored_at: Utc::now(),
            rooms_restored,
            total_unread,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = WebSocketMessage::Ping;
        let json = msg.to_json().unwrap();
        assert_eq!(json, r#"{"type":"Ping"}"#);
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{"type":"Ping"}"#;
        let msg = WebSocketMessage::from_json(json).unwrap();
        match msg {
            WebSocketMessage::Ping => {},
            _ => panic!("Expected Ping message"),
        }
    }

    #[test]
    fn test_chat_message_roundtrip() {
        let room_id = Uuid::new_v4();
        let original = WebSocketMessage::ChatMessage {
            room_id,
            content: "Hello, World!".to_string(),
            reply_to: None,
        };
        
        let json = original.to_json().unwrap();
        let deserialized = WebSocketMessage::from_json(&json).unwrap();
        
        match deserialized {
            WebSocketMessage::ChatMessage { room_id: r, content: c, reply_to } => {
                assert_eq!(r, room_id);
                assert_eq!(c, "Hello, World!");
                assert!(reply_to.is_none());
            }
            _ => panic!("Expected ChatMessage"),
        }
    }

    #[test]
    fn test_auth_result() {
        let success = WebSocketMessage::auth_success();
        match success {
            WebSocketMessage::AuthResult { success, message } => {
                assert!(success);
                assert_eq!(message, "Authentication successful");
            }
            _ => panic!("Expected AuthResult"),
        }

        let failed = WebSocketMessage::auth_failed("Invalid token");
        match failed {
            WebSocketMessage::AuthResult { success, message } => {
                assert!(!success);
                assert_eq!(message, "Invalid token");
            }
            _ => panic!("Expected AuthResult"),
        }
    }
}
