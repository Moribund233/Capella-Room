//! 阶段 6 消息回复功能测试
//!
//! 测试内容：
//! - 正常回复消息
//! - 回复不存在的消息（应报错）
//! - 回复其他房间的消息（应报错）
//! - 回复已删除的消息（应报错）
//! - 获取消息列表时正确显示引用上下文
//! - 搜索消息时正确显示引用上下文

use std::env;

use seredeli_room::{
    config::{DatabaseConfig, JwtConfig},
    db::Database,
    error::AppError,
    services::{
        auth_service::AuthService, message_service::MessageService, room_service::RoomService,
        user_service::UserService,
    },
};
use uuid::Uuid;

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

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test or environment");

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
    let email = format!("{}_reply_test@seredeli.com", username);
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
async fn create_test_room(room_service: &RoomService, owner_id: Uuid, name: &str) -> Uuid {
    let room = room_service
        .create_room(name, Some("Test room for reply"), owner_id, false, 100)
        .await
        .unwrap();
    room.id
}

// ==================== 消息回复功能测试 ====================

#[tokio::test]
async fn test_reply_message_success() {
    // 测试正常回复消息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user1").await;
    let room_id = create_test_room(&room_service, user_id, "Reply Success Room").await;

    // 创建原始消息
    let original = message_service
        .create_text_message(room_id, user_id, "Original message", None)
        .await
        .unwrap();

    // 验证回复消息
    message_service
        .validate_reply_message(original.id, room_id)
        .await
        .expect("Should validate successfully");
}

#[tokio::test]
async fn test_reply_nonexistent_message() {
    // 测试回复不存在的消息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user2").await;
    let room_id = create_test_room(&room_service, user_id, "Reply Nonexistent Room").await;

    // 使用随机的消息 ID
    let fake_message_id = Uuid::new_v4();

    // 验证应该失败
    let result = message_service
        .validate_reply_message(fake_message_id, room_id)
        .await;

    assert!(result.is_err());
    match result {
        Err(AppError::NotFound) => (), // 期望的错误类型
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_reply_message_from_different_room() {
    // 测试回复其他房间的消息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user3").await;
    let room1_id = create_test_room(&room_service, user_id, "Room 1").await;
    let room2_id = create_test_room(&room_service, user_id, "Room 2").await;

    // 在房间1创建消息
    let original = message_service
        .create_text_message(room1_id, user_id, "Message in room 1", None)
        .await
        .unwrap();

    // 尝试在房间2回复房间1的消息
    let result = message_service
        .validate_reply_message(original.id, room2_id)
        .await;

    assert!(result.is_err());
    match result {
        Err(AppError::Validation(msg)) => {
            assert!(msg.contains("同一会话"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_reply_deleted_message() {
    // 测试回复已删除的消息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user4").await;
    let room_id = create_test_room(&room_service, user_id, "Reply Deleted Room").await;

    // 创建消息
    let original = message_service
        .create_text_message(room_id, user_id, "Message to be deleted", None)
        .await
        .unwrap();

    // 删除消息
    message_service
        .delete_message(original.id, user_id)
        .await
        .unwrap();

    // 尝试回复已删除的消息
    let result = message_service
        .validate_reply_message(original.id, room_id)
        .await;

    assert!(result.is_err());
    match result {
        Err(AppError::Validation(msg)) => {
            assert!(msg.contains("已删除"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_get_reply_to_info() {
    // 测试获取被引用消息的信息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user5").await;
    let room_id = create_test_room(&room_service, user_id, "Get Reply Info Room").await;

    // 创建原始消息
    let original = message_service
        .create_text_message(room_id, user_id, "Original content", None)
        .await
        .unwrap();

    // 获取被引用消息的信息
    let reply_info = message_service
        .get_reply_to_info(original.id)
        .await
        .unwrap()
        .expect("Should get reply info");

    assert_eq!(reply_info.id, original.id);
    assert_eq!(reply_info.content, "Original content");
    assert_eq!(reply_info.sender.id, user_id);
}

#[tokio::test]
async fn test_get_reply_to_info_for_deleted_message() {
    // 测试获取已删除消息的引用信息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user6").await;
    let room_id = create_test_room(&room_service, user_id, "Deleted Reply Info Room").await;

    // 创建消息
    let original = message_service
        .create_text_message(room_id, user_id, "Content before deletion", None)
        .await
        .unwrap();

    // 删除消息
    message_service
        .delete_message(original.id, user_id)
        .await
        .unwrap();

    // 获取已删除消息的引用信息
    let reply_info = message_service
        .get_reply_to_info(original.id)
        .await
        .unwrap()
        .expect("Should get reply info for deleted message");

    // 已删除消息的内容应该被替换
    assert_eq!(reply_info.content, "[此消息已被删除]");
}

#[tokio::test]
async fn test_room_messages_include_reply_context() {
    // 测试获取消息历史时包含引用上下文
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user7").await;
    let room_id = create_test_room(&room_service, user_id, "Reply Context Room").await;

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

    // 获取消息历史
    let messages = message_service
        .get_room_messages(room_id, 10, None)
        .await
        .unwrap();

    // 找到回复消息
    let reply_response = messages
        .iter()
        .find(|m| m.id == reply.id)
        .expect("Should find reply message");

    // 验证回复消息包含引用上下文
    assert_eq!(reply_response.reply_to, Some(original.id));
    assert!(
        reply_response.reply_to_message.is_some(),
        "Reply message should include reply_to_message"
    );

    let reply_to_info = reply_response.reply_to_message.as_ref().unwrap();
    assert_eq!(reply_to_info.id, original.id);
    assert_eq!(reply_to_info.content, "Original message");
}

#[tokio::test]
async fn test_search_messages_include_reply_context() {
    // 测试搜索消息时包含引用上下文
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user8").await;
    let room_id = create_test_room(&room_service, user_id, "Search Reply Context Room").await;

    // 创建原始消息
    let original = message_service
        .create_text_message(room_id, user_id, "Searchable original", None)
        .await
        .unwrap();

    // 创建回复消息
    let _reply = message_service
        .create_text_message(room_id, user_id, "Searchable reply", Some(original.id))
        .await
        .unwrap();

    // 搜索消息
    let results = message_service
        .search_messages(Some(room_id), "reply", 10)
        .await
        .unwrap();

    assert!(!results.is_empty(), "Should find messages");

    // 验证搜索结果包含引用上下文
    let found_reply = results
        .iter()
        .find(|m| m.content == "Searchable reply")
        .expect("Should find reply message");

    assert_eq!(found_reply.reply_to, Some(original.id));
    assert!(
        found_reply.reply_to_message.is_some(),
        "Search result should include reply_to_message"
    );
}

#[tokio::test]
async fn test_message_without_reply_has_no_reply_context() {
    // 测试普通消息不包含引用上下文
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user9").await;
    let room_id = create_test_room(&room_service, user_id, "No Reply Room").await;

    // 创建普通消息（不带回复）
    let _message = message_service
        .create_text_message(room_id, user_id, "Normal message", None)
        .await
        .unwrap();

    // 获取消息历史
    let messages = message_service
        .get_room_messages(room_id, 10, None)
        .await
        .unwrap();

    let normal_msg = messages
        .iter()
        .find(|m| m.content == "Normal message")
        .expect("Should find normal message");

    // 普通消息不应该有 reply_to 和 reply_to_message
    assert!(normal_msg.reply_to.is_none());
    assert!(normal_msg.reply_to_message.is_none());
}

#[tokio::test]
async fn test_batch_get_reply_to_infos() {
    // 测试批量获取被引用消息信息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret".to_string()),
        expiration_hours: 24,
    });

    let (user_id, _) = create_test_user(&user_service, &auth_service, "reply_user10").await;
    let room_id = create_test_room(&room_service, user_id, "Batch Reply Info Room").await;

    // 创建多条消息
    let msg1 = message_service
        .create_text_message(room_id, user_id, "Message 1", None)
        .await
        .unwrap();

    let msg2 = message_service
        .create_text_message(room_id, user_id, "Message 2", None)
        .await
        .unwrap();

    // 批量获取消息信息
    let infos = message_service
        .get_reply_to_infos(&[msg1.id, msg2.id])
        .await
        .unwrap();

    assert_eq!(infos.len(), 2);
    assert!(infos.contains_key(&msg1.id));
    assert!(infos.contains_key(&msg2.id));
    assert_eq!(infos.get(&msg1.id).unwrap().content, "Message 1");
    assert_eq!(infos.get(&msg2.id).unwrap().content, "Message 2");
}
