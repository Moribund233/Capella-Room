use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::message::MessageType;
use crate::models::room::MessagePreview;

/// 被引用消息的信息（用于 WebSocket 传输）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyToInfo {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

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
    RoomJoined {
        room_id: Uuid,
        user_id: Uuid,
        username: String,
    },

    /// 房间离开结果
    RoomLeft {
        room_id: Uuid,
        user_id: Uuid,
        username: String,
    },

    /// 用户加入房间通知（广播给其他用户）
    UserJoined {
        room_id: Uuid,
        user_id: Uuid,
        username: String,
    },

    /// 用户离开房间通知（广播给其他用户）
    UserLeft {
        room_id: Uuid,
        user_id: Uuid,
        username: String,
    },

    /// 在线用户列表
    OnlineUsers { room_id: Uuid, users: Vec<UserInfo> },

    // ========== 消息通信 ==========
    /// 发送聊天消息
    ChatMessage {
        room_id: Uuid,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message_type: Option<MessageType>,
    },

    /// 收到聊天消息（广播）
    NewMessage {
        message_id: Uuid,
        room_id: Uuid,
        sender_id: Uuid,
        sender_name: String,
        content: String,
        message_type: MessageType,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to_message: Option<ReplyToInfo>,
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

    // ========== 消息反应 ==========
    /// 添加表情反应
    AddReaction {
        message_id: Uuid,
        emoji: String,
    },

    /// 移除表情反应
    RemoveReaction {
        message_id: Uuid,
        emoji: String,
    },

    /// 反应已添加（广播）
    ReactionAdded {
        message_id: Uuid,
        room_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    /// 反应已移除（广播）
    ReactionRemoved {
        message_id: Uuid,
        room_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    /// 编辑消息
    EditMessage {
        message_id: Uuid,
        new_content: String,
    },

    /// 消息已编辑通知
    MessageEdited {
        message_id: Uuid,
        new_content: String,
        edited_at: DateTime<Utc>,
    },

    /// 删除消息
    DeleteMessage { message_id: Uuid },

    /// 消息已删除通知
    MessageDeleted { message_id: Uuid },

    /// 置顶消息
    PinMessage {
        message_id: Uuid,
        room_id: Uuid,
    },

    /// 取消置顶消息
    UnpinMessage {
        message_id: Uuid,
        room_id: Uuid,
    },

    /// 消息已置顶（广播）
    MessagePinned {
        message_id: Uuid,
        room_id: Uuid,
        pinned_by: Uuid,
        pinned_by_name: String,
        content_preview: String,
        pinned_at: DateTime<Utc>,
    },

    /// 消息已取消置顶（广播）
    MessageUnpinned {
        message_id: Uuid,
        room_id: Uuid,
        unpinned_by: Uuid,
        unpinned_at: DateTime<Utc>,
    },

    // ========== 系统消息 ==========
    /// 系统广播
    SystemMessage { content: String },

    /// 房间信息更新
    RoomUpdated {
        room_id: Uuid,
        name: Option<String>,
        description: Option<String>,
    },

    // ========== 用户状态 ==========
    /// 更新用户状态
    UpdateStatus { status: UserStatus },

    /// 用户状态变更通知
    UserStatusChanged {
        user_id: Uuid,
        username: String,
        status: UserStatus,
    },

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

    // ========== 消息通知系统 ==========
    /// 私信通知
    PrivateMessage {
        message_id: Uuid,
        sender_id: Uuid,
        sender_name: String,
        content: String,
        created_at: DateTime<Utc>,
    },

    /// @提及通知
    Mentioned {
        message_id: Uuid,
        room_id: Uuid,
        mentioned_by: Uuid,
        mentioned_by_name: String,
        content_preview: String,
        created_at: DateTime<Utc>,
    },

    /// 房间邀请通知
    RoomInvitation {
        invitation_id: Uuid,
        room_id: Uuid,
        room_name: String,
        invited_by: Uuid,
        invited_by_name: String,
        created_at: DateTime<Utc>,
    },

    /// 系统通知
    SystemNotification {
        notification_type: NotificationType,
        title: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
    },

    /// 文件上传完成通知
    FileUploadComplete {
        file_id: Uuid,
        file_name: String,
        file_url: String,
        file_size: u64,
        uploaded_at: DateTime<Utc>,
    },

    // ========== 通知管理 ==========
    /// 获取离线通知
    GetOfflineNotifications {
        #[serde(skip_serializing_if = "Option::is_none")]
        last_notification_id: Option<Uuid>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<i64>,
    },

    /// 标记通知已读
    MarkNotificationRead { notification_id: Uuid },

    /// 标记所有通知已读
    MarkAllNotificationsRead,

    /// 离线通知列表
    OfflineNotifications {
        notifications: Vec<Notification>,
        has_more: bool,
    },

    /// 通知已读确认
    NotificationReadConfirm { notification_id: Uuid },

    // ========== 待办通知系统 ==========
    /// 待办通知
    PendingAction {
        notification_id: Uuid,
        action_type: String,
        title: String,
        description: String,
        deadline: Option<DateTime<Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
    },

    /// 响应待办通知
    RespondPendingAction {
        notification_id: Uuid,
        action: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        comment: Option<String>,
    },

    /// 待办响应确认
    PendingActionResponse {
        notification_id: Uuid,
        success: bool,
        message: String,
        new_status: String,
    },

    /// 获取待办列表
    GetPendingActions {
        #[serde(skip_serializing_if = "Option::is_none")]
        action_type: Option<String>,
    },

    /// 待办列表响应
    PendingActionsList {
        actions: Vec<PendingActionInfo>,
        total: usize,
    },

    // ========== 系统日志流 ==========
    /// 订阅系统日志
    SubscribeLogs {
        /// 日志级别过滤: error, warn, info, debug, all
        #[serde(skip_serializing_if = "Option::is_none")]
        level: Option<String>,
        /// 模块过滤: websocket, room, message, performance, all
        #[serde(skip_serializing_if = "Option::is_none")]
        module: Option<String>,
    },

    /// 取消订阅系统日志
    UnsubscribeLogs,

    /// 系统日志条目（实时推送）
    LogEntry {
        /// 日志级别
        level: String,
        /// 日志模块/目标
        target: String,
        /// 日志消息
        message: String,
        /// 时间戳
        timestamp: DateTime<Utc>,
        /// 相关字段（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        fields: Option<serde_json::Value>,
    },

    /// 订阅确认
    LogSubscriptionConfirmed { success: bool, message: String },

    // ========== 房间消息摘要 ==========
    /// 房间消息摘要（用于房间列表实时更新）
    RoomMessageSummary {
        room_id: Uuid,
        last_message: MessagePreview,
        unread_count: u32,
    },

    // ========== 外部服务自定义事件 ==========
    /// 外部服务发送自定义事件（需要 OAuth 身份）
    CustomEvent {
        event_name: String,
        room_id: Uuid,
        data: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        persistent: Option<bool>,
    },

    /// 转发给客户端的自定义事件
    CustomEventForward {
        event_name: String,
        room_id: Uuid,
        source_app: String,
        data: serde_json::Value,
        timestamp: DateTime<Utc>,
    },

    /// 客户端请求获取错过的自定义事件
    GetMissedCustomEvents {
        room_id: Uuid,
        since: DateTime<Utc>,
    },

    /// 服务端返回错过的自定义事件
    MissedCustomEvents {
        room_id: Uuid,
        events: Vec<CustomEventForwardPayload>,
        has_more: bool,
    },
}

/// 自定义事件转发 payload（用于 MissedCustomEvents）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEventForwardPayload {
    pub id: Uuid,
    pub event_name: String,
    pub source_app: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// 通知类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// 新功能、新版本
    New,
    /// 重要公告
    Important,
    /// 警告、维护通知
    Warning,
}

/// 待办操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PendingActionType {
    /// 确认执行
    Approve,
    /// 拒绝变更
    Reject,
    /// 稍后提醒
    Snooze,
}

impl std::fmt::Display for PendingActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PendingActionType::Approve => write!(f, "approve"),
            PendingActionType::Reject => write!(f, "reject"),
            PendingActionType::Snooze => write!(f, "snooze"),
        }
    }
}

impl std::str::FromStr for PendingActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "approve" => Ok(PendingActionType::Approve),
            "reject" => Ok(PendingActionType::Reject),
            "snooze" => Ok(PendingActionType::Snooze),
            _ => Err(format!("Unknown pending action type: {}", s)),
        }
    }
}

/// 待办操作状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "action_status", rename_all = "snake_case")]
pub enum PendingActionStatus {
    /// 待处理
    Pending,
    /// 已确认
    Approved,
    /// 已拒绝
    Rejected,
    /// 已延迟
    Snoozed,
}

/// 待办通知信息
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PendingActionInfo {
    pub notification_id: Uuid,
    pub action_type: String,
    pub title: String,
    pub description: String,
    pub deadline: Option<DateTime<Utc>>,
    pub action_status: PendingActionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_config_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_config_value: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 待办响应请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingActionResponseRequest {
    pub notification_id: Uuid,
    pub action: PendingActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// 通知结构（用于离线通知同步）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub notification_type: NotificationDbType,
    pub title: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    pub is_read: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 数据库通知类型枚举（对应数据库中的 notification_type）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationDbType {
    PrivateMessage,
    Mentioned,
    RoomInvitation,
    SystemNotification,
    FileUploadComplete,
    ConfigReloadRequired,
    PendingAction,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message: Option<ReplyToInfo>,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        serde_json::to_string(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize message: {}", e))
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize message: {}", e))
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
        message_type: MessageType,
        reply_to: Option<Uuid>,
        reply_to_message: Option<ReplyToInfo>,
    ) -> Self {
        Self::NewMessage {
            message_id,
            room_id,
            sender_id,
            sender_name: sender_name.to_string(),
            content: content.to_string(),
            message_type,
            reply_to,
            reply_to_message,
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
            WebSocketMessage::Ping => {}
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
            message_type: None,
        };

        let json = original.to_json().unwrap();
        let deserialized = WebSocketMessage::from_json(&json).unwrap();

        match deserialized {
            WebSocketMessage::ChatMessage {
                room_id: r,
                content: c,
                reply_to,
                message_type,
            } => {
                assert_eq!(r, room_id);
                assert_eq!(c, "Hello, World!");
                assert!(reply_to.is_none());
                assert!(message_type.is_none());
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

    #[test]
    fn test_message_edited_roundtrip() {
        let message_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let msg = WebSocketMessage::MessageEdited {
            message_id,
            new_content: "Edited content".to_string(),
            edited_at: now,
        };

        let json = msg.to_json().unwrap();
        let decoded = WebSocketMessage::from_json(&json).unwrap();

        match decoded {
            WebSocketMessage::MessageEdited {
                message_id: id,
                new_content,
                edited_at,
            } => {
                assert_eq!(id, message_id);
                assert_eq!(new_content, "Edited content");
                assert!((edited_at - now).num_seconds().abs() <= 1);
            }
            _ => panic!("Expected MessageEdited"),
        }
    }

    #[test]
    fn test_message_deleted_roundtrip() {
        let message_id = Uuid::new_v4();

        let msg = WebSocketMessage::MessageDeleted { message_id };

        let json = msg.to_json().unwrap();
        let decoded = WebSocketMessage::from_json(&json).unwrap();

        match decoded {
            WebSocketMessage::MessageDeleted { message_id: id } => {
                assert_eq!(id, message_id);
            }
            _ => panic!("Expected MessageDeleted"),
        }
    }

    #[test]
    fn test_message_read_receipt_roundtrip() {
        let message_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let msg = WebSocketMessage::MessageReadReceipt {
            message_id,
            user_id,
        };

        let json = msg.to_json().unwrap();
        let decoded = WebSocketMessage::from_json(&json).unwrap();

        match decoded {
            WebSocketMessage::MessageReadReceipt {
                message_id: mid,
                user_id: uid,
            } => {
                assert_eq!(mid, message_id);
                assert_eq!(uid, user_id);
            }
            _ => panic!("Expected MessageReadReceipt"),
        }
    }

    #[test]
    fn test_edit_message_request_roundtrip() {
        let message_id = Uuid::new_v4();

        let msg = WebSocketMessage::EditMessage {
            message_id,
            new_content: "Updated content".to_string(),
        };

        let json = msg.to_json().unwrap();
        let decoded = WebSocketMessage::from_json(&json).unwrap();

        match decoded {
            WebSocketMessage::EditMessage {
                message_id: id,
                new_content,
            } => {
                assert_eq!(id, message_id);
                assert_eq!(new_content, "Updated content");
            }
            _ => panic!("Expected EditMessage"),
        }
    }

    #[test]
    fn test_delete_message_request_roundtrip() {
        let message_id = Uuid::new_v4();

        let msg = WebSocketMessage::DeleteMessage { message_id };

        let json = msg.to_json().unwrap();
        let decoded = WebSocketMessage::from_json(&json).unwrap();

        match decoded {
            WebSocketMessage::DeleteMessage { message_id: id } => {
                assert_eq!(id, message_id);
            }
            _ => panic!("Expected DeleteMessage"),
        }
    }

    #[test]
    fn test_message_read_request_roundtrip() {
        let message_id = Uuid::new_v4();

        let msg = WebSocketMessage::MessageRead { message_id };

        let json = msg.to_json().unwrap();
        let decoded = WebSocketMessage::from_json(&json).unwrap();

        match decoded {
            WebSocketMessage::MessageRead { message_id: id } => {
                assert_eq!(id, message_id);
            }
            _ => panic!("Expected MessageRead"),
        }
    }
}
