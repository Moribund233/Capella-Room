//! 阶段4.6：消息通知系统测试
//!
//! 测试内容：
//! - @提及检测和通知
//! - 私信通知
//! - 房间邀请通知
//! - 系统通知
//! - 文件上传通知
//! - 离线通知存储和同步
//! - 通知已读状态管理

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use seredeli_room::services::notification_service::{
    FileInfo, MentionInfo, PrivateMessageInfo, RoomInvitationInfo, SystemNotificationInfo,
};
use seredeli_room::utils::mention;
use seredeli_room::websocket::manager::WebSocketManager;
use seredeli_room::websocket::protocol::{NotificationDbType, NotificationType, WebSocketMessage};

/// ==================== @提及检测测试 ====================

#[tokio::test]
async fn test_extract_mentions_single() {
    let content = "Hello @alice!";
    let mentions = mention::extract_mentions(content);
    assert_eq!(mentions, vec!["alice"]);
}

#[tokio::test]
async fn test_extract_mentions_multiple() {
    let content = "Hello @alice and @bob!";
    let mentions = mention::extract_mentions(content);
    assert_eq!(mentions, vec!["alice", "bob"]);
}

#[tokio::test]
async fn test_extract_mentions_duplicate() {
    let content = "@alice @alice @bob";
    let mentions = mention::extract_mentions(content);
    assert_eq!(mentions, vec!["alice", "bob"]);
}

#[tokio::test]
async fn test_extract_mentions_case_insensitive() {
    let content = "Hello @Alice and @BOB!";
    let mentions = mention::extract_mentions(content);
    assert_eq!(mentions, vec!["alice", "bob"]);
}

#[tokio::test]
async fn test_extract_mentions_no_mentions() {
    let content = "Hello everyone!";
    let mentions = mention::extract_mentions(content);
    assert!(mentions.is_empty());
}

#[tokio::test]
async fn test_extract_mentions_invalid_format() {
    // 太短的用户名（少于3个字符）
    let content = "Hello @ab!";
    let mentions = mention::extract_mentions(content);
    assert!(mentions.is_empty());

    // 太长的用户名（超过20个字符）
    let content = "Hello @verylongusernamethatistoolong!";
    let mentions = mention::extract_mentions(content);
    assert!(mentions.is_empty());
}

#[tokio::test]
async fn test_filter_self_mentions() {
    let mentions = vec![
        "alice".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
    ];
    let filtered = mention::filter_self_mentions(mentions, "alice");
    assert_eq!(filtered, vec!["bob", "charlie"]);
}

#[tokio::test]
async fn test_has_mentions() {
    assert!(mention::has_mentions("Hello @alice!"));
    assert!(!mention::has_mentions("Hello everyone!"));
}

#[tokio::test]
async fn test_mention_count() {
    assert_eq!(mention::mention_count("Hello @alice and @bob!"), 2);
    assert_eq!(mention::mention_count("Hello everyone!"), 0);
    assert_eq!(mention::mention_count("@alice @alice @bob"), 2); // 去重后
}

/// ==================== 通知服务单元测试 ====================

#[test]
fn test_notification_service_creation() {
    // 验证通知服务结构体可以创建
    // 注意：实际创建需要数据库连接，这里只做结构体验证
    let ws_manager = WebSocketManager::new();

    // 验证WebSocket管理器创建成功
    assert!(!ws_manager.is_user_online(uuid::Uuid::new_v4()));
}

#[tokio::test]
async fn test_private_message_info() {
    let info = PrivateMessageInfo {
        message_id: Uuid::new_v4(),
        sender_id: Uuid::new_v4(),
        sender_name: "test_user".to_string(),
        content: "Hello!".to_string(),
        created_at: Utc::now(),
    };
    assert_eq!(info.sender_name, "test_user");
    assert_eq!(info.content, "Hello!");
}

#[tokio::test]
async fn test_mention_info() {
    let info = MentionInfo {
        message_id: Uuid::new_v4(),
        room_id: Uuid::new_v4(),
        mentioned_by: Uuid::new_v4(),
        mentioned_by_name: "test_user".to_string(),
        content_preview: "Hello @user".to_string(),
        created_at: Utc::now(),
    };
    assert_eq!(info.mentioned_by_name, "test_user");
    assert_eq!(info.content_preview, "Hello @user");
}

#[tokio::test]
async fn test_room_invitation_info() {
    let info = RoomInvitationInfo {
        invitation_id: Uuid::new_v4(),
        room_id: Uuid::new_v4(),
        room_name: "Test Room".to_string(),
        invited_by: Uuid::new_v4(),
        invited_by_name: "admin".to_string(),
        created_at: Utc::now(),
    };
    assert_eq!(info.room_name, "Test Room");
    assert_eq!(info.invited_by_name, "admin");
}

#[tokio::test]
async fn test_system_notification_info() {
    let info = SystemNotificationInfo {
        notification_type: NotificationType::New,
        title: "New Feature".to_string(),
        content: "Check out our new feature!".to_string(),
        data: None,
        created_at: Utc::now(),
    };
    assert_eq!(info.title, "New Feature");
    assert_eq!(info.notification_type, NotificationType::New);
}

#[tokio::test]
async fn test_file_info() {
    let info = FileInfo {
        file_id: Uuid::new_v4(),
        file_name: "test.txt".to_string(),
        file_url: "/uploads/test.txt".to_string(),
        file_size: 1024,
        uploaded_at: Utc::now(),
    };
    assert_eq!(info.file_name, "test.txt");
    assert_eq!(info.file_size, 1024);
}

/// ==================== WebSocket 通知协议测试 ====================

#[test]
fn test_websocket_private_message() {
    let msg = WebSocketMessage::PrivateMessage {
        message_id: Uuid::new_v4(),
        sender_id: Uuid::new_v4(),
        sender_name: "alice".to_string(),
        content: "Hello!".to_string(),
        created_at: Utc::now(),
    };

    let json = msg.to_json().unwrap();
    assert!(json.contains("PrivateMessage"));
    assert!(json.contains("alice"));
    assert!(json.contains("Hello!"));
}

#[test]
fn test_websocket_mentioned() {
    let msg = WebSocketMessage::Mentioned {
        message_id: Uuid::new_v4(),
        room_id: Uuid::new_v4(),
        mentioned_by: Uuid::new_v4(),
        mentioned_by_name: "bob".to_string(),
        content_preview: "Hey @alice".to_string(),
        created_at: Utc::now(),
    };

    let json = msg.to_json().unwrap();
    assert!(json.contains("Mentioned"));
    assert!(json.contains("bob"));
    assert!(json.contains("Hey @alice"));
}

#[test]
fn test_websocket_room_invitation() {
    let msg = WebSocketMessage::RoomInvitation {
        invitation_id: Uuid::new_v4(),
        room_id: Uuid::new_v4(),
        room_name: "Test Room".to_string(),
        invited_by: Uuid::new_v4(),
        invited_by_name: "admin".to_string(),
        created_at: Utc::now(),
    };

    let json = msg.to_json().unwrap();
    assert!(json.contains("RoomInvitation"));
    assert!(json.contains("Test Room"));
    assert!(json.contains("admin"));
}

#[test]
fn test_websocket_system_notification() {
    let msg = WebSocketMessage::SystemNotification {
        notification_type: NotificationType::Important,
        title: "System Maintenance".to_string(),
        content: "System will be down for maintenance".to_string(),
        data: Some(json!({"duration": "2 hours"})),
        created_at: Utc::now(),
    };

    let json = msg.to_json().unwrap();
    assert!(json.contains("SystemNotification"));
    assert!(json.contains("System Maintenance"));
    assert!(json.contains("important"));
}

#[test]
fn test_websocket_file_upload_complete() {
    let msg = WebSocketMessage::FileUploadComplete {
        file_id: Uuid::new_v4(),
        file_name: "document.pdf".to_string(),
        file_url: "/uploads/document.pdf".to_string(),
        file_size: 1024000,
        uploaded_at: Utc::now(),
    };

    let json = msg.to_json().unwrap();
    assert!(json.contains("FileUploadComplete"));
    assert!(json.contains("document.pdf"));
}

#[test]
fn test_websocket_get_offline_notifications_message() {
    let msg = WebSocketMessage::GetOfflineNotifications {
        last_notification_id: None,
        limit: Some(50),
    };

    let json = msg.to_json().unwrap();
    assert!(json.contains("GetOfflineNotifications"));
    assert!(json.contains("50"));
}

#[test]
fn test_websocket_mark_notification_read_message() {
    let notification_id = Uuid::new_v4();
    let msg = WebSocketMessage::MarkNotificationRead { notification_id };

    let json = msg.to_json().unwrap();
    assert!(json.contains("MarkNotificationRead"));
    assert!(json.contains(&notification_id.to_string()));
}

#[test]
fn test_websocket_mark_all_notifications_read_message() {
    let msg = WebSocketMessage::MarkAllNotificationsRead;

    let json = msg.to_json().unwrap();
    assert!(json.contains("MarkAllNotificationsRead"));
}

#[test]
fn test_notification_type_enum() {
    assert_eq!(NotificationType::New, NotificationType::New);
    assert_eq!(NotificationType::Important, NotificationType::Important);
    assert_eq!(NotificationType::Warning, NotificationType::Warning);
    assert_ne!(NotificationType::New, NotificationType::Important);
}

#[test]
fn test_notification_db_type_enum() {
    assert_eq!(
        NotificationDbType::PrivateMessage,
        NotificationDbType::PrivateMessage
    );
    assert_eq!(NotificationDbType::Mentioned, NotificationDbType::Mentioned);
    assert_eq!(
        NotificationDbType::RoomInvitation,
        NotificationDbType::RoomInvitation
    );
    assert_eq!(
        NotificationDbType::SystemNotification,
        NotificationDbType::SystemNotification
    );
    assert_eq!(
        NotificationDbType::FileUploadComplete,
        NotificationDbType::FileUploadComplete
    );
}

/// ==================== 边界情况测试 ====================

#[tokio::test]
async fn test_mention_in_empty_message() {
    let content = "";
    let mentions = mention::extract_mentions(content);
    assert!(mentions.is_empty());
}

#[tokio::test]
async fn test_mention_with_special_characters() {
    // 测试包含特殊字符的用户名
    let content = "Hello @user_123 and @test-user!";
    let mentions = mention::extract_mentions(content);
    // @test-user 中的 test 符合长度要求（4个字符），会被匹配
    // @user_123 也会被匹配
    assert_eq!(mentions, vec!["test", "user_123"]);
}

#[tokio::test]
async fn test_mention_at_start_and_end() {
    let content = "@alice Hello @bob";
    let mentions = mention::extract_mentions(content);
    assert_eq!(mentions, vec!["alice", "bob"]);
}

#[tokio::test]
async fn test_mention_with_numbers() {
    let content = "Hello @user123 and @123user!";
    let mentions = mention::extract_mentions(content);
    assert_eq!(mentions, vec!["123user", "user123"]);
}

#[tokio::test]
async fn test_filter_self_mentions_empty() {
    let mentions: Vec<String> = vec![];
    let filtered = mention::filter_self_mentions(mentions, "alice");
    assert!(filtered.is_empty());
}

#[tokio::test]
async fn test_filter_self_mentions_all_self() {
    let mentions = vec!["alice".to_string(), "ALICE".to_string()];
    let filtered = mention::filter_self_mentions(mentions, "alice");
    assert!(filtered.is_empty());
}

#[tokio::test]
async fn test_mention_count_zero() {
    assert_eq!(mention::mention_count(""), 0);
    assert_eq!(mention::mention_count("Hello world"), 0);
}
