//! 阶段五消息系统功能测试
//!
//! 阶段五包含以下功能：
//! - 5.1 消息模型 - 完善 Message 模型，支持多种消息类型和回复功能
//! - 5.2 消息存储 - 实现消息持久化到数据库，软删除机制
//! - 5.3 消息查询 - 实现历史消息获取、游标分页、消息搜索
//! - 5.4 消息接口 - 实现 HTTP API 接口
//!
//! 验收标准：
//! ✅ 消息可以正确存储到数据库
//! ✅ 可以获取聊天室历史消息
//! ✅ 消息分页加载正常工作
//! ✅ 可以搜索消息内容
//! ✅ 消息可以软删除

use std::env;

use seredeli_room::{
    config::{DatabaseConfig, JwtConfig},
    db::Database,
    error::AppError,
    models::message::{MessageType, SendMessageRequest},
    services::{
        auth_service::AuthService,
        message_service::MessageService,
        room_service::RoomService,
        user_service::UserService,
    },
};
use uuid::Uuid;
use validator::Validate;

/// 测试辅助函数：加载测试环境变量
fn load_test_env() {
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

/// 测试辅助函数：创建测试数据库连接
async fn setup_test_db() -> Database {
    load_test_env();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env.test or environment");

    let max_connections = env::var("APP_DATABASE__MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let db_config = DatabaseConfig {
        url: Some(database_url),
        max_connections,
        acquire_timeout_secs: 30,
        idle_timeout_secs: 600,
    };

    let db = Database::new(&db_config)
        .await
        .expect("Failed to connect to test database");

    db.migrate().await.expect("Failed to run migrations");

    db
}

/// 测试辅助函数：创建测试用户
async fn create_test_user(
    user_service: &UserService,
    auth_service: &AuthService,
    username: &str,
) -> (Uuid, String) {
    let email = format!("{}_msg_test@seredeli.com", username);
    let password = "TestPassword123";

    // 检查用户是否已存在
    if let Ok(Some(user)) = user_service.get_user_by_email(&email).await {
        return (user.id, password.to_string());
    }

    let password_hash = auth_service.hash_password(password).unwrap();
    let user = user_service
        .create_user(username, &email, &password_hash)
        .await
        .unwrap();

    (user.id, password.to_string())
}

/// 测试辅助函数：创建测试聊天室
async fn create_test_room(
    room_service: &RoomService,
    owner_id: Uuid,
    name: &str,
) -> Uuid {
    let room = room_service
        .create_room(name, Some("Test room for messaging"), owner_id, false, 100)
        .await
        .unwrap();
    room.id
}

// ==================== 5.1 消息模型测试 ====================

#[tokio::test]
async fn test_5_1_message_type_serialization() {
    // 测试消息类型的序列化
    let msg_type = MessageType::Text;
    let json = serde_json::to_string(&msg_type).unwrap();
    assert_eq!(json, "\"text\"");

    let msg_type = MessageType::Image;
    let json = serde_json::to_string(&msg_type).unwrap();
    assert_eq!(json, "\"image\"");

    let msg_type = MessageType::File;
    let json = serde_json::to_string(&msg_type).unwrap();
    assert_eq!(json, "\"file\"");

    let msg_type = MessageType::System;
    let json = serde_json::to_string(&msg_type).unwrap();
    assert_eq!(json, "\"system\"");
}

#[tokio::test]
async fn test_5_1_send_message_request_validation() {
    // 测试发送消息请求验证
    let valid_request = SendMessageRequest {
        content: "Hello, World!".to_string(),
        message_type: MessageType::Text,
        reply_to: None,
    };
    assert!(valid_request.validate().is_ok());

    // 空内容应该失败
    let empty_content = SendMessageRequest {
        content: "".to_string(),
        message_type: MessageType::Text,
        reply_to: None,
    };
    assert!(empty_content.validate().is_err());

    // 内容过长应该失败
    let long_content = SendMessageRequest {
        content: "a".repeat(2001),
        message_type: MessageType::Text,
        reply_to: None,
    };
    assert!(long_content.validate().is_err());

    // 带回复 ID 的有效请求
    let with_reply = SendMessageRequest {
        content: "Reply to message".to_string(),
        message_type: MessageType::Text,
        reply_to: Some(Uuid::new_v4()),
    };
    assert!(with_reply.validate().is_ok());
}

// ==================== 5.2 消息存储测试 ====================

#[tokio::test]
async fn test_5_2_create_message() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    // 创建测试用户和聊天室
    let (user_id, _) = create_test_user(&user_service, &auth_service, "msg_creator").await;
    let room_id = create_test_room(&room_service, user_id, "Message Test Room").await;

    // 创建消息
    let content = "Test message content";
    let message = message_service
        .create_text_message(room_id, user_id, content, None)
        .await
        .unwrap();

    // 验证消息
    assert_eq!(message.room_id, room_id);
    assert_eq!(message.sender_id, user_id);
    assert_eq!(message.content, content);
    assert_eq!(message.message_type, MessageType::Text);
    assert!(!message.is_deleted);
    assert!(message.reply_to.is_none());
}

#[tokio::test]
async fn test_5_2_create_message_with_reply() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "msg_replier").await;
    let room_id = create_test_room(&room_service, user_id, "Reply Test Room").await;

    // 创建原始消息
    let original = message_service
        .create_text_message(room_id, user_id, "Original message", None)
        .await
        .unwrap();

    // 创建回复消息
    let reply = message_service
        .create_text_message(room_id, user_id, "Reply message", Some(original.id))
        .await
        .unwrap();

    // 验证回复
    assert_eq!(reply.reply_to, Some(original.id));
}

#[tokio::test]
async fn test_5_2_soft_delete_message() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "msg_deleter").await;
    let room_id = create_test_room(&room_service, user_id, "Delete Test Room").await;

    // 创建消息
    let message = message_service
        .create_text_message(room_id, user_id, "Message to delete", None)
        .await
        .unwrap();

    // 软删除消息
    message_service
        .delete_message(message.id, user_id)
        .await
        .unwrap();

    // 验证消息已被标记为删除
    let deleted_message = message_service
        .get_message_by_id(message.id)
        .await
        .unwrap()
        .unwrap();
    assert!(deleted_message.is_deleted);
}

#[tokio::test]
async fn test_5_2_delete_message_permission() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    // 创建两个用户
    let (user1_id, _) = create_test_user(&user_service, &auth_service, "msg_owner").await;
    let (user2_id, _) = create_test_user(&user_service, &auth_service, "msg_other").await;
    let room_id = create_test_room(&room_service, user1_id, "Permission Test Room").await;

    // 用户 1 创建消息
    let message = message_service
        .create_text_message(room_id, user1_id, "Owner's message", None)
        .await
        .unwrap();

    // 用户 2 尝试删除用户 1 的消息，应该失败
    let result = message_service.delete_message(message.id, user2_id).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::Forbidden => (), // 预期错误
        _ => panic!("Expected Forbidden error"),
    }
}

// ==================== 5.3 消息查询测试 ====================

#[tokio::test]
async fn test_5_3_get_room_messages() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "msg_reader").await;
    let room_id = create_test_room(&room_service, user_id, "History Test Room").await;

    // 创建多条消息
    let mut message_ids = Vec::new();
    for i in 0..10 {
        let msg = message_service
            .create_text_message(room_id, user_id, &format!("Message {}", i), None)
            .await
            .unwrap();
        message_ids.push(msg.id);
    }

    // 获取消息历史
    let messages = message_service
        .get_room_messages(room_id, 50, None)
        .await
        .unwrap();

    // 验证消息数量和顺序
    assert_eq!(messages.len(), 10);
    // 消息应该按创建时间倒序排列
    assert_eq!(messages[0].content, "Message 9");
    assert_eq!(messages[9].content, "Message 0");
}

#[tokio::test]
async fn test_5_3_cursor_pagination() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "pagination_user").await;
    let room_id = create_test_room(&room_service, user_id, "Pagination Test Room").await;

    // 创建 20 条消息
    let mut message_ids = Vec::new();
    for i in 0..20 {
        let msg = message_service
            .create_text_message(room_id, user_id, &format!("Message {}", i), None)
            .await
            .unwrap();
        message_ids.push(msg.id);
    }

    // 第一页：获取最新的 10 条
    let page1 = message_service
        .get_room_messages(room_id, 10, None)
        .await
        .unwrap();
    assert_eq!(page1.len(), 10);
    assert_eq!(page1[0].content, "Message 19");
    assert_eq!(page1[9].content, "Message 10");

    // 第二页：获取 before 参数之前的消息
    let before_id = page1.last().unwrap().id;
    let page2 = message_service
        .get_room_messages(room_id, 10, Some(before_id))
        .await
        .unwrap();
    assert_eq!(page2.len(), 10);
    assert_eq!(page2[0].content, "Message 9");
    assert_eq!(page2[9].content, "Message 0");
}

#[tokio::test]
async fn test_5_3_search_messages() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "search_user").await;
    let room_id = create_test_room(&room_service, user_id, "Search Test Room").await;

    // 创建包含不同关键词的消息
    message_service
        .create_text_message(room_id, user_id, "Hello world", None)
        .await
        .unwrap();
    message_service
        .create_text_message(room_id, user_id, "Rust programming is great", None)
        .await
        .unwrap();
    message_service
        .create_text_message(room_id, user_id, "Hello Rust community", None)
        .await
        .unwrap();
    message_service
        .create_text_message(room_id, user_id, "Goodbye world", None)
        .await
        .unwrap();

    // 搜索包含 "Rust" 的消息
    let results = message_service
        .search_messages(Some(room_id), "Rust", 50)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|m| m.content.contains("Rust")));

    // 搜索包含 "Hello" 的消息
    let results = message_service
        .search_messages(Some(room_id), "Hello", 50)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);

    // 搜索不存在的关键词
    let results = message_service
        .search_messages(Some(room_id), "NonExistent", 50)
        .await
        .unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_5_3_search_excludes_deleted() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "search_del_user").await;
    let room_id = create_test_room(&room_service, user_id, "Search Exclude Deleted").await;

    // 创建消息
    let msg1 = message_service
        .create_text_message(room_id, user_id, "Searchable message", None)
        .await
        .unwrap();
    let msg2 = message_service
        .create_text_message(room_id, user_id, "Another searchable message", None)
        .await
        .unwrap();

    // 删除其中一条
    message_service.delete_message(msg2.id, user_id).await.unwrap();

    // 搜索应该只返回未删除的消息
    let results = message_service
        .search_messages(Some(room_id), "searchable", 50)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, msg1.id);
}

// ==================== 5.4 消息接口测试 ====================

#[tokio::test]
async fn test_5_4_get_message_by_id() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "get_by_id_user").await;
    let room_id = create_test_room(&room_service, user_id, "Get By ID Room").await;

    // 创建消息
    let message = message_service
        .create_text_message(room_id, user_id, "Test message", None)
        .await
        .unwrap();

    // 通过 ID 获取消息
    let retrieved = message_service
        .get_message_by_id(message.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.id, message.id);
    assert_eq!(retrieved.content, "Test message");

    // 获取不存在的消息
    let not_found = message_service
        .get_message_by_id(Uuid::new_v4())
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_5_4_get_latest_messages() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "latest_user").await;
    let room_id = create_test_room(&room_service, user_id, "Latest Messages Room").await;

    // 创建 15 条消息
    for i in 0..15 {
        message_service
            .create_text_message(room_id, user_id, &format!("Latest {}", i), None)
            .await
            .unwrap();
    }

    // 获取最新的 10 条
    let latest = message_service
        .get_latest_messages(room_id, 10)
        .await
        .unwrap();

    assert_eq!(latest.len(), 10);
    // 最新的消息应该在前面
    assert!(latest[0].content.contains("Latest"));
}

#[tokio::test]
async fn test_5_4_get_missed_messages() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "missed_user").await;
    let room_id = create_test_room(&room_service, user_id, "Missed Messages Room").await;

    // 创建 10 条消息
    let mut message_ids = Vec::new();
    for i in 0..10 {
        let msg = message_service
            .create_text_message(room_id, user_id, &format!("Missed {}", i), None)
            .await
            .unwrap();
        message_ids.push(msg.id);
    }

    // 获取最后一条消息之后的消息（应该没有）
    let missed = message_service
        .get_missed_messages(room_id, Some(message_ids[9]), 50)
        .await
        .unwrap();
    assert_eq!(missed.len(), 0);

    // 获取第 5 条消息之后的消息
    let missed = message_service
        .get_missed_messages(room_id, Some(message_ids[4]), 50)
        .await
        .unwrap();
    assert_eq!(missed.len(), 5);
    assert_eq!(missed[0].content, "Missed 5");
}

// ==================== 综合测试 ====================

#[tokio::test]
async fn test_5_full_message_lifecycle() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    // 创建用户和聊天室（使用唯一名称避免冲突）
    let unique_id = Uuid::new_v4();
    let (user_id, _) = create_test_user(&user_service, &auth_service, &format!("lifecycle_{}", unique_id)).await;
    let room_id = create_test_room(&room_service, user_id, &format!("Lifecycle Test Room {}", unique_id)).await;

    // 1. 创建消息
    let msg1 = message_service
        .create_text_message(room_id, user_id, "First message", None)
        .await
        .unwrap();

    // 2. 创建回复（短暂等待以确保时间戳不同）
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let _msg2 = message_service
        .create_text_message(room_id, user_id, "Reply to first", Some(msg1.id))
        .await
        .unwrap();

    // 3. 获取消息历史
    let messages = message_service
        .get_room_messages(room_id, 50, None)
        .await
        .unwrap();
    assert_eq!(messages.len(), 2, "Should have 2 messages initially");

    // 4. 搜索消息 - 验证搜索功能正常
    let search_results = message_service
        .search_messages(Some(room_id), "First message", 50)
        .await
        .unwrap();
    assert_eq!(search_results.len(), 1, "Should find exactly 1 message");

    // 5. 软删除消息
    message_service.delete_message(msg1.id, user_id).await.unwrap();

    // 验证消息确实被标记为删除
    let deleted_msg_check = message_service.get_message_by_id(msg1.id).await.unwrap().unwrap();
    assert!(deleted_msg_check.is_deleted, "Message should be marked as deleted");

    // 6. 验证删除后的搜索结果
    let search_after_delete = message_service
        .search_messages(Some(room_id), "First message", 50)
        .await
        .unwrap();
    
    eprintln!("Search results after delete: {}", search_after_delete.len());
    for result in &search_after_delete {
        eprintln!("  - {} (is_deleted: {})", result.content, result.is_deleted);
    }
    
    assert_eq!(search_after_delete.len(), 0, "Deleted message should not appear in search");
}
