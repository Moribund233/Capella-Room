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
    services::{
        auth_service::AuthService,
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

    Database::new(&config).await.expect("Failed to create database")
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

    // 创建 5 个房间
    let mut rooms = Vec::new();
    for i in 0..5 {
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        let room = room_service
            .create_room(&format!("Room {} {}", i, unique_id), Some(&format!("Room {}", i)), user_id, false, 50)
            .await
            .unwrap();
        rooms.push(room);
    }

    // 测试分页：第一页（limit=2, offset=0）
    let page1 = room_service
        .list_recent_rooms(Some(user_id), 2, 0)
        .await
        .unwrap();
    assert_eq!(page1.len(), 2);

    // 测试分页：第二页（limit=2, offset=2）
    let page2 = room_service
        .list_recent_rooms(Some(user_id), 2, 2)
        .await
        .unwrap();
    assert_eq!(page2.len(), 2);

    // 验证两页的房间不重复
    let page1_ids: Vec<_> = page1.iter().map(|r| r.id).collect();
    let page2_ids: Vec<_> = page2.iter().map(|r| r.id).collect();
    
    for id in &page1_ids {
        assert!(!page2_ids.contains(id));
    }
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
