//! 私聊功能测试
//!
//! 测试功能：
//! - 创建私聊房间
//! - 获取或创建已存在的私聊房间
//! - 获取私聊房间列表
//! - 私聊房间权限控制

use std::env;

use seredeli_room::{
    config::DatabaseConfig,
    db::Database,
    error::AppError,
    services::{room_service::RoomService, user_service::UserService},
};
use uuid::Uuid;

/// 加载测试环境变量
fn load_test_env() {
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

/// 创建测试数据库连接
async fn setup_test_db() -> Database {
    load_test_env();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let config = DatabaseConfig {
        url: Some(database_url),
        max_connections: 5,
        acquire_timeout_secs: 30,
        idle_timeout_secs: 600,
    };

    Database::new(&config)
        .await
        .expect("Failed to connect to database")
}

/// 创建测试用户
async fn create_test_user(user_service: &UserService, suffix: &str) -> (Uuid, String) {
    let short_uuid = &Uuid::new_v4().to_string()[..8];
    let username = format!("t_{}_{}", suffix, short_uuid);
    let email = format!("{}@test.com", username);
    let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$hash";

    let user = user_service
        .create_user(&username, &email, password_hash)
        .await
        .expect("Failed to create test user");

    (user.id, username)
}

#[tokio::test]
async fn test_create_direct_room() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (user_a_id, _username_a) = create_test_user(&user_service, "a").await;
    let (user_b_id, username_b) = create_test_user(&user_service, "b").await;

    let room = room_service
        .get_or_create_direct_room(user_a_id, user_b_id)
        .await
        .expect("Failed to create direct room");

    assert_eq!(room.name, username_b);
    assert_eq!(room.target_user.id, user_b_id);
    assert_eq!(room.target_user.username, username_b);
}

#[tokio::test]
async fn test_get_existing_direct_room() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (user_a_id, _) = create_test_user(&user_service, "a").await;
    let (user_b_id, _) = create_test_user(&user_service, "b").await;

    let room1 = room_service
        .get_or_create_direct_room(user_a_id, user_b_id)
        .await
        .expect("Failed to create direct room");

    let room2 = room_service
        .get_or_create_direct_room(user_a_id, user_b_id)
        .await
        .expect("Failed to get existing direct room");

    assert_eq!(room1.id, room2.id);

    let room3 = room_service
        .get_or_create_direct_room(user_b_id, user_a_id)
        .await
        .expect("Failed to get existing direct room from other side");

    assert_eq!(room1.id, room3.id);
}

#[tokio::test]
async fn test_get_user_direct_rooms() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (user_a_id, _) = create_test_user(&user_service, "a").await;
    let (user_b_id, _) = create_test_user(&user_service, "b").await;
    let (user_c_id, _) = create_test_user(&user_service, "c").await;

    let _ = room_service
        .get_or_create_direct_room(user_a_id, user_b_id)
        .await
        .expect("Failed to create direct room with b");
    let _ = room_service
        .get_or_create_direct_room(user_a_id, user_c_id)
        .await
        .expect("Failed to create direct room with c");

    let rooms = room_service
        .get_user_direct_rooms(user_a_id)
        .await
        .expect("Failed to get direct rooms");

    assert_eq!(rooms.len(), 2);
}

#[tokio::test]
async fn test_cannot_create_direct_room_with_self() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (user_id, _) = create_test_user(&user_service, "self").await;

    let result = room_service
        .get_or_create_direct_room(user_id, user_id)
        .await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn test_direct_room_response_from_other_side() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (user_a_id, username_a) = create_test_user(&user_service, "a").await;
    let (user_b_id, _username_b) = create_test_user(&user_service, "b").await;

    let _ = room_service
        .get_or_create_direct_room(user_a_id, user_b_id)
        .await
        .expect("Failed to create direct room");

    let rooms = room_service
        .get_user_direct_rooms(user_b_id)
        .await
        .expect("Failed to get direct rooms");

    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0].target_user.id, user_a_id);
    assert_eq!(rooms[0].target_user.username, username_a);
    assert_eq!(rooms[0].name, username_a);
}

#[tokio::test]
async fn test_direct_room_is_private() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (user_a_id, _) = create_test_user(&user_service, "a").await;
    let (user_b_id, _) = create_test_user(&user_service, "b").await;
    let (user_c_id, _) = create_test_user(&user_service, "c").await;

    let room = room_service
        .get_or_create_direct_room(user_a_id, user_b_id)
        .await
        .expect("Failed to create direct room");

    let result = room_service.join_room(room.id, user_c_id).await;

    assert!(result.is_err());
}
