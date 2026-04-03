//! 阶段 6 扩展功能测试：房间更新时间
//!
//! 测试内容：
//! - 房间响应中包含 updated_at 字段
//! - 最近更新房间列表接口功能
//! - 房间更新时间排序正确性

use std::env;

use seredeli_room::{
    config::{DatabaseConfig, JwtConfig},
    db::Database,
    middleware::rate_limit::{RateLimitConfig, RateLimiter},
    services::{
        auth_service::AuthService,
        message_service::MessageService,
        room_service::RoomService,
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

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env.test or environment");

    let max_connections = env::var("APP_DATABASE__MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let config = DatabaseConfig {
        url: database_url,
        max_connections,
    };

    let db = Database::new(&config).await.expect("Failed to create database");

    // 运行数据库迁移
    db.migrate().await.expect("Failed to run migrations");

    db
}

/// 创建测试用户
async fn create_test_user(user_service: &UserService, username: &str) -> (Uuid, String) {
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let email = format!("{}{}@test.com", username, unique_id);
    let password = "TestPassword123!";

    let auth_service = AuthService::new(JwtConfig {
        secret: "test-secret-key-for-jwt-signing-in-tests-only".to_string(),
        expiration_hours: 24,
    });

    let password_hash = auth_service.hash_password(password).unwrap();
    let user = user_service.create_user(username, &email, &password_hash).await.unwrap();

    (user.id, password.to_string())
}

// ==================== 房间更新时间测试 ====================

#[tokio::test]
async fn test_room_response_has_updated_at() {
    // 测试房间响应包含 updated_at 字段
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testroom{}", unique_id)).await;

    // 创建房间
    let room_name = format!("Test Room {}", unique_id);
    let room = room_service
        .create_room(&room_name, Some("Test description"), user_id, false, 50)
        .await
        .unwrap();

    // 获取房间详情
    let room_detail = room_service
        .get_room_detail(room.id)
        .await
        .unwrap()
        .unwrap();

    // 验证响应包含 updated_at
    assert_eq!(room_detail.id, room.id);
    assert_eq!(room_detail.name, room_name);
    // updated_at 应该存在且大于等于 created_at
    assert!(room_detail.updated_at >= room_detail.created_at);
}

#[tokio::test]
async fn test_update_room_updates_updated_at() {
    // 测试更新房间会更新 updated_at 字段
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testupdate{}", unique_id)).await;

    // 创建房间
    let room_name = format!("Update Test Room {}", unique_id);
    let room = room_service
        .create_room(&room_name, Some("Original description"), user_id, false, 50)
        .await
        .unwrap();

    // 获取初始的 updated_at
    let room_detail_before = room_service
        .get_room_detail(room.id)
        .await
        .unwrap()
        .unwrap();
    let updated_at_before = room_detail_before.updated_at;

    // 等待一小段时间以确保时间戳有差异
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // 更新房间
    room_service
        .update_room(room.id, Some(&format!("Updated Room {}", unique_id)), Some("Updated description"), None, None)
        .await
        .unwrap();

    // 获取更新后的房间详情
    let room_detail_after = room_service
        .get_room_detail(room.id)
        .await
        .unwrap()
        .unwrap();

    // 验证 updated_at 已更新
    assert!(room_detail_after.updated_at > updated_at_before);
    assert_eq!(room_detail_after.name, format!("Updated Room {}", unique_id));
    assert_eq!(room_detail_after.description, Some("Updated description".to_string()));
}

#[tokio::test]
async fn test_list_recent_rooms() {
    // 测试按更新时间排序的房间列表
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testrecent{}", unique_id)).await;

    // 创建多个房间
    let room1 = room_service
        .create_room(&format!("Room 1 {}", unique_id), Some("First room"), user_id, false, 50)
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let room2 = room_service
        .create_room(&format!("Room 2 {}", unique_id), Some("Second room"), user_id, false, 50)
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // 更新第一个房间，使其 updated_at 最新
    room_service
        .update_room(room1.id, Some(&format!("Room 1 Updated {}", unique_id)), None, None, None)
        .await
        .unwrap();

    // 获取最近更新的房间列表
    let recent_rooms = room_service
        .list_recent_rooms(Some(user_id), 10, 0)
        .await
        .unwrap();

    // 验证房间按 updated_at 降序排序
    assert!(recent_rooms.len() >= 2);
    
    // 找到我们创建的两个房间
    let room1_index = recent_rooms.iter().position(|r| r.id == room1.id);
    let room2_index = recent_rooms.iter().position(|r| r.id == room2.id);
    
    assert!(room1_index.is_some());
    assert!(room2_index.is_some());
    
    // room1 应该排在 room2 前面（因为 room1 最后被更新）
    assert!(room1_index.unwrap() < room2_index.unwrap());
}

#[tokio::test]
async fn test_list_recent_rooms_pagination() {
    // 测试最近房间列表的分页功能
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testpagination{}", unique_id)).await;

    // 创建 3 个房间
    let mut created_room_ids = Vec::new();
    for i in 0..3 {
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        let room = room_service
            .create_room(&format!("Room {} {}", i, unique_id), Some(&format!("Room {}", i)), user_id, false, 50)
            .await
            .unwrap();
        created_room_ids.push(room.id);
    }

    // 测试分页：第一页（limit=2, offset=0）
    let page1 = room_service
        .list_recent_rooms(Some(user_id), 2, 0)
        .await
        .unwrap();
    
    // 验证第一页最多返回2个房间
    assert!(page1.len() <= 2, "第一页应该最多返回2个房间");
    assert!(page1.len() >= 1, "第一页应该至少返回1个房间");

    // 测试分页：第二页（limit=2, offset=2）
    let page2 = room_service
        .list_recent_rooms(Some(user_id), 2, 2)
        .await
        .unwrap();

    // 验证两页的房间不重复（只检查我们创建的房间）
    let page1_ids: Vec<_> = page1.iter().map(|r| r.id).collect();
    let page2_ids: Vec<_> = page2.iter().map(|r| r.id).collect();
    
    // 只验证我们创建的3个房间中，第一页和第二页没有重复
    let created_in_page1: Vec<_> = created_room_ids.iter().filter(|id| page1_ids.contains(id)).cloned().collect();
    let created_in_page2: Vec<_> = created_room_ids.iter().filter(|id| page2_ids.contains(id)).cloned().collect();
    
    for id in &created_in_page1 {
        assert!(!created_in_page2.contains(id), "同一个创建的房间不应该出现在两页中");
    }

    // 验证所有创建的房间都能在分页结果中找到
    let total_created_found = created_in_page1.len() + created_in_page2.len();
    assert!(
        total_created_found >= 2,
        "应该至少找到2个创建的房间（考虑到可能的数据库中已有数据）"
    );
}

#[tokio::test]
async fn test_recent_rooms_respects_privacy() {
    // 测试最近房间列表尊重房间隐私设置
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user1_id, _) = create_test_user(&user_service, &format!("owner{}", unique_id)).await;
    let (user2_id, _) = create_test_user(&user_service, &format!("other{}", unique_id)).await;

    // 用户 1 创建公开房间
    let public_room = room_service
        .create_room(&format!("Public Room {}", unique_id), Some("Public"), user1_id, false, 50)
        .await
        .unwrap();

    // 用户 1 创建私有房间
    let private_room = room_service
        .create_room(&format!("Private Room {}", unique_id), Some("Private"), user1_id, true, 50)
        .await
        .unwrap();

    // 用户 2 只能看到公开房间
    let recent_rooms_user2 = room_service
        .list_recent_rooms(Some(user2_id), 10, 0)
        .await
        .unwrap();
    
    let public_found = recent_rooms_user2.iter().any(|r| r.id == public_room.id);
    let private_found = recent_rooms_user2.iter().any(|r| r.id == private_room.id);
    
    assert!(public_found, "用户 2 应该能看到公开房间");
    assert!(!private_found, "用户 2 不应该能看到私有房间");

    // 用户 1 可以看到两个房间
    let recent_rooms_user1 = room_service
        .list_recent_rooms(Some(user1_id), 10, 0)
        .await
        .unwrap();
    
    let user1_public_found = recent_rooms_user1.iter().any(|r| r.id == public_room.id);
    let user1_private_found = recent_rooms_user1.iter().any(|r| r.id == private_room.id);
    
    assert!(user1_public_found, "用户 1 应该能看到公开房间");
    assert!(user1_private_found, "用户 1 应该能看到自己的私有房间");
}

#[tokio::test]
async fn test_recent_rooms_anonymous_user() {
    // 测试匿名用户只能看到公开房间
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("anon{}", unique_id)).await;

    // 创建公开和私有房间
    let public_room = room_service
        .create_room(&format!("Public {}", unique_id), Some("Public"), user_id, false, 50)
        .await
        .unwrap();
    
    let private_room = room_service
        .create_room(&format!("Private {}", unique_id), Some("Private"), user_id, true, 50)
        .await
        .unwrap();

    // 匿名用户（user_id = None）只能看到公开房间
    let recent_rooms_anon = room_service
        .list_recent_rooms(None, 10, 0)
        .await
        .unwrap();
    
    let public_found = recent_rooms_anon.iter().any(|r| r.id == public_room.id);
    let private_found = recent_rooms_anon.iter().any(|r| r.id == private_room.id);
    
    assert!(public_found, "匿名用户应该能看到公开房间");
    assert!(!private_found, "匿名用户不应该能看到私有房间");
}

// ==================== 速率限制中间件测试 ====================

#[tokio::test]
async fn test_rate_limiter_ip_limit() {
    // 测试IP级别的速率限制
    let limiter = RateLimiter::default();

    // 在限制范围内应该通过
    for i in 0..5 {
        let allowed = limiter.check_ip_limit("127.0.0.1", 10, 60).await;
        assert!(allowed, "请求 {} 应该被允许", i);
    }
}

#[tokio::test]
async fn test_rate_limiter_ip_limit_exceeded() {
    // 测试IP超过限制后被拒绝
    let limiter = RateLimiter::default();

    // 发送超过限制的请求
    for _ in 0..10 {
        limiter.check_ip_limit("192.168.1.1", 10, 60).await;
    }

    // 第11个请求应该被拒绝
    let allowed = limiter.check_ip_limit("192.168.1.1", 10, 60).await;
    assert!(!allowed, "超过限制后请求应该被拒绝");
}

#[tokio::test]
async fn test_rate_limiter_user_limit() {
    // 测试用户级别的速率限制
    let limiter = RateLimiter::default();

    // 用户级别限制
    for i in 0..5 {
        let allowed = limiter.check_user_limit("user123", 10, 60).await;
        assert!(allowed, "请求 {} 应该被允许", i);
    }

    // 不同用户互不影响
    let allowed = limiter.check_user_limit("user456", 10, 60).await;
    assert!(allowed, "不同用户不应该受影响");
}

#[tokio::test]
async fn test_rate_limiter_different_paths() {
    // 测试不同路径的限制策略
    let limiter = RateLimiter::default();

    // 认证接口限制
    let (limit, window) = limiter.get_ip_limit("/api/v1/auth/login");
    assert_eq!(limit, 5);
    assert_eq!(window, 60);

    // 消息接口限制
    let (limit, window) = limiter.get_ip_limit("/api/v1/messages");
    assert_eq!(limit, 30);
    assert_eq!(window, 60);

    // 房间接口限制
    let (limit, window) = limiter.get_ip_limit("/api/v1/rooms");
    assert_eq!(limit, 20);
    assert_eq!(window, 60);

    // 默认接口限制
    let (limit, window) = limiter.get_ip_limit("/api/v1/users");
    assert_eq!(limit, 100);
    assert_eq!(window, 60);
}

// ==================== 消息编辑功能测试 ====================

#[tokio::test]
async fn test_edit_message() {
    // 测试编辑消息功能
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testedit{}", unique_id)).await;

    // 创建房间
    let room = room_service
        .create_room(&format!("Edit Test Room {}", unique_id), Some("Test"), user_id, false, 50)
        .await
        .unwrap();

    // 创建消息
    let message = message_service
        .create_text_message(room.id, user_id, "原始消息内容", None)
        .await
        .unwrap();

    assert_eq!(message.content, "原始消息内容");
    assert_eq!(message.edit_count, 0);
    assert!(message.edited_at.is_none());

    // 编辑消息
    let edited_message = message_service
        .edit_message(message.id, user_id, "编辑后的消息内容")
        .await
        .unwrap();

    assert_eq!(edited_message.content, "编辑后的消息内容");
    assert_eq!(edited_message.edit_count, 1);
    assert!(edited_message.edited_at.is_some());
}

#[tokio::test]
async fn test_edit_message_permission() {
    // 测试只有消息发送者才能编辑
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user1_id, _) = create_test_user(&user_service, &format!("owner{}", unique_id)).await;
    let (user2_id, _) = create_test_user(&user_service, &format!("other{}", unique_id)).await;

    // 创建房间
    let room = room_service
        .create_room(&format!("Permission Room {}", unique_id), Some("Test"), user1_id, false, 50)
        .await
        .unwrap();

    // 用户1创建消息
    let message = message_service
        .create_text_message(room.id, user1_id, "用户1的消息", None)
        .await
        .unwrap();

    // 用户2尝试编辑应该失败
    let result = message_service
        .edit_message(message.id, user2_id, "恶意修改")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_edit_message_history() {
    // 测试消息编辑历史记录
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testhistory{}", unique_id)).await;

    // 创建房间
    let room = room_service
        .create_room(&format!("History Room {}", unique_id), Some("Test"), user_id, false, 50)
        .await
        .unwrap();

    // 创建消息
    let message = message_service
        .create_text_message(room.id, user_id, "版本1", None)
        .await
        .unwrap();

    // 编辑多次
    message_service
        .edit_message(message.id, user_id, "版本2")
        .await
        .unwrap();

    message_service
        .edit_message(message.id, user_id, "版本3")
        .await
        .unwrap();

    // 获取编辑历史
    let history = message_service
        .get_message_edit_history(message.id, 10)
        .await
        .unwrap();

    assert_eq!(history.len(), 2);
    // 最新的编辑应该在前面
    assert_eq!(history[0].old_content, "版本2");
    assert_eq!(history[0].new_content, "版本3");
    assert_eq!(history[1].old_content, "版本1");
    assert_eq!(history[1].new_content, "版本2");
}

#[tokio::test]
async fn test_edit_system_message_forbidden() {
    // 测试不能编辑系统消息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testsys{}", unique_id)).await;

    // 创建房间
    let room = room_service
        .create_room(&format!("System Room {}", unique_id), Some("Test"), user_id, false, 50)
        .await
        .unwrap();

    // 创建系统消息
    let message = message_service
        .create_message(
            room.id,
            user_id,
            "系统消息",
            seredeli_room::models::message::MessageType::System,
            None,
        )
        .await
        .unwrap();

    // 尝试编辑系统消息应该失败
    let result = message_service
        .edit_message(message.id, user_id, "修改系统消息")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_messages_fulltext() {
    // 测试全文搜索功能
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testsearch{}", unique_id)).await;

    // 创建房间
    let room = room_service
        .create_room(&format!("Search Room {}", unique_id), Some("Test"), user_id, false, 50)
        .await
        .unwrap();

    // 创建多条消息 - 使用简单英文单词便于tsvector索引
    message_service
        .create_text_message(room.id, user_id, "hello world from rust", None)
        .await
        .unwrap();
    message_service
        .create_text_message(room.id, user_id, "rust is awesome", None)
        .await
        .unwrap();
    message_service
        .create_text_message(room.id, user_id, "python programming", None)
        .await
        .unwrap();

    // 全文搜索 - 使用小写关键词
    let results = message_service
        .search_messages_fulltext(Some(room.id), "rust", 10)
        .await
        .unwrap();

    // 验证全文搜索能找到包含"rust"的消息
    // 注意：触发器会在插入时自动更新content_tsv字段
    assert!(
        results.len() >= 2,
        "全文搜索应该找到至少2条包含rust的消息，实际找到{}条",
        results.len()
    );
    assert!(results.iter().any(|m| m.content.contains("rust")));

    // 同时测试普通搜索作为对比
    let normal_results = message_service
        .search_messages(Some(room.id), "rust", 10)
        .await
        .unwrap();

    assert!(
        normal_results.len() >= 2,
        "普通搜索应该找到至少2条包含rust的消息"
    );
    assert!(normal_results.iter().any(|m| m.content.contains("rust")));

    // 测试搜索"hello" - 应该只找到一条
    let hello_results = message_service
        .search_messages_fulltext(Some(room.id), "hello", 10)
        .await
        .unwrap();

    assert!(
        hello_results.len() >= 1,
        "搜索hello应该至少找到1条消息"
    );
    assert!(hello_results.iter().any(|m| m.content.contains("hello")));
}

#[tokio::test]
async fn test_edit_message_multiple_times() {
    // 测试多次编辑消息
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    let message_service = MessageService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testmulti{}", unique_id)).await;

    // 创建房间
    let room = room_service
        .create_room(&format!("Multi Edit Room {}", unique_id), Some("Test"), user_id, false, 50)
        .await
        .unwrap();

    // 创建消息
    let message = message_service
        .create_text_message(room.id, user_id, "初始内容", None)
        .await
        .unwrap();

    // 编辑5次
    for i in 1..=5 {
        let edited = message_service
            .edit_message(message.id, user_id, &format!("编辑版本{}", i))
            .await
            .unwrap();
        assert_eq!(edited.edit_count, i);
    }

    // 获取最终消息
    let final_message = message_service
        .get_message_by_id(message.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(final_message.edit_count, 5);
    assert_eq!(final_message.content, "编辑版本5");
}
