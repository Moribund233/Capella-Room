//! 房间邀请功能测试
//!
//! 测试功能：
//! - 创建房间邀请
//! - 获取邀请列表
//! - 撤销邀请
//! - 通过邀请码加入房间
//! - 验证邀请码

use std::env;

use capella_room::{
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

/// 创建测试房间
async fn create_test_room(room_service: &RoomService, owner_id: Uuid, name: &str) -> Uuid {
    let room = room_service
        .create_room(name, Some("Test room"), owner_id, false, 100)
        .await
        .expect("Failed to create test room");

    room.id
}

#[tokio::test]
async fn test_create_invitation() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (owner_id, _) = create_test_user(&user_service, "owner").await;
    let room_id = create_test_room(&room_service, owner_id, "Test Room").await;

    let invitation = room_service
        .create_invitation(room_id, owner_id, Some(24), Some(5))
        .await
        .expect("Failed to create invitation");

    assert_eq!(invitation.room_id, room_id);
    assert_eq!(invitation.inviter_id, owner_id);
    assert!(invitation.is_active);
    assert_eq!(invitation.used_count, 0);
    assert_eq!(invitation.max_uses, Some(5));
    assert!(!invitation.invite_code.is_empty());
    assert!(invitation.expires_at.is_some());
}

#[tokio::test]
async fn test_get_room_invitations() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (owner_id, _) = create_test_user(&user_service, "owner").await;
    let room_id = create_test_room(&room_service, owner_id, "Test Room").await;

    let _ = room_service
        .create_invitation(room_id, owner_id, Some(24), Some(5))
        .await
        .expect("Failed to create invitation 1");
    let _ = room_service
        .create_invitation(room_id, owner_id, Some(48), Some(10))
        .await
        .expect("Failed to create invitation 2");

    let invitations = room_service
        .get_room_invitations(room_id, owner_id)
        .await
        .expect("Failed to get invitations");

    assert_eq!(invitations.len(), 2);
}

#[tokio::test]
async fn test_revoke_invitation() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (owner_id, _) = create_test_user(&user_service, "owner").await;
    let room_id = create_test_room(&room_service, owner_id, "Test Room").await;

    let invitation = room_service
        .create_invitation(room_id, owner_id, Some(24), Some(5))
        .await
        .expect("Failed to create invitation");

    room_service
        .revoke_invitation(room_id, invitation.id, owner_id)
        .await
        .expect("Failed to revoke invitation");

    let invitations = room_service
        .get_room_invitations(room_id, owner_id)
        .await
        .expect("Failed to get invitations");

    assert!(invitations.is_empty() || !invitations[0].is_active);
}

#[tokio::test]
async fn test_join_by_invite_code() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (owner_id, _) = create_test_user(&user_service, "owner").await;
    let (joiner_id, _) = create_test_user(&user_service, "joiner").await;
    let room_id = create_test_room(&room_service, owner_id, "Test Room").await;

    let invitation = room_service
        .create_invitation(room_id, owner_id, Some(24), Some(5))
        .await
        .expect("Failed to create invitation");

    let joined_room_id = room_service
        .join_by_invite_code(&invitation.invite_code, joiner_id)
        .await
        .expect("Failed to join by invite code");

    assert_eq!(joined_room_id, room_id);

    let member = room_service
        .get_room_member(room_id, joiner_id)
        .await
        .expect("Failed to check membership");

    assert!(member.is_some());
}

#[tokio::test]
async fn test_validate_invite_code() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (owner_id, _) = create_test_user(&user_service, "owner").await;
    let room_id = create_test_room(&room_service, owner_id, "Test Room").await;

    let invitation = room_service
        .create_invitation(room_id, owner_id, Some(24), Some(5))
        .await
        .expect("Failed to create invitation");

    let valid_invitation = room_service
        .validate_invite_code(&invitation.invite_code)
        .await
        .expect("Failed to validate invite code");

    assert!(valid_invitation.is_some());

    let invalid_invitation = room_service
        .validate_invite_code("INVALIDCODE")
        .await
        .expect("Failed to validate invite code");

    assert!(invalid_invitation.is_none());
}

#[tokio::test]
async fn test_invitation_permission_denied() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (owner_id, _) = create_test_user(&user_service, "owner").await;
    let (other_id, _) = create_test_user(&user_service, "other").await;
    let room_id = create_test_room(&room_service, owner_id, "Test Room").await;

    let result = room_service
        .create_invitation(room_id, other_id, Some(24), Some(5))
        .await;

    assert!(matches!(result, Err(AppError::Forbidden)));
}

#[tokio::test]
async fn test_invitation_max_uses() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());
    let room_service = RoomService::new(db.clone());

    let (owner_id, _) = create_test_user(&user_service, "owner").await;
    let room_id = create_test_room(&room_service, owner_id, "Test Room").await;

    let invitation = room_service
        .create_invitation(room_id, owner_id, Some(24), Some(1))
        .await
        .expect("Failed to create invitation");

    let (joiner1_id, _) = create_test_user(&user_service, "joiner1").await;
    let _ = room_service
        .join_by_invite_code(&invitation.invite_code, joiner1_id)
        .await
        .expect("First user should join successfully");

    let (joiner2_id, _) = create_test_user(&user_service, "joiner2").await;
    let result = room_service
        .join_by_invite_code(&invitation.invite_code, joiner2_id)
        .await;

    assert!(result.is_err());
}
