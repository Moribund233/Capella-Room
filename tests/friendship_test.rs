//! 好友功能测试
//!
//! 测试功能：
//! - 发送好友申请
//! - 接受/拒绝好友申请
//! - 取消好友申请
//! - 获取好友列表
//! - 删除好友
//! - 搜索用户

use std::env;

use capella_room::{
    config::DatabaseConfig,
    db::Database,
    error::AppError,
    models::user::{FriendRequestStatus, SendFriendRequest},
    services::user_service::UserService,
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
async fn test_send_friend_request() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (sender_id, _) = create_test_user(&user_service, "sender").await;
    let (receiver_id, _) = create_test_user(&user_service, "receiver").await;

    let request = SendFriendRequest {
        target_user_id: receiver_id,
        message: Some("Hello, let's be friends!".to_string()),
    };

    let friend_request = user_service
        .send_friend_request(sender_id, request)
        .await
        .expect("Failed to send friend request");

    assert_eq!(friend_request.sender_id, sender_id);
    assert_eq!(friend_request.receiver_id, receiver_id);
    assert!(matches!(
        friend_request.status,
        FriendRequestStatus::Pending
    ));
    assert_eq!(
        friend_request.message,
        Some("Hello, let's be friends!".to_string())
    );
}

#[tokio::test]
async fn test_cannot_send_request_to_self() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (user_id, _) = create_test_user(&user_service, "self").await;

    let request = SendFriendRequest {
        target_user_id: user_id,
        message: None,
    };

    let result = user_service.send_friend_request(user_id, request).await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn test_cannot_send_duplicate_request() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (sender_id, _) = create_test_user(&user_service, "sender").await;
    let (receiver_id, _) = create_test_user(&user_service, "receiver").await;

    let request = SendFriendRequest {
        target_user_id: receiver_id,
        message: None,
    };

    let _ = user_service
        .send_friend_request(sender_id, request.clone())
        .await
        .expect("First request should succeed");

    let result = user_service.send_friend_request(sender_id, request).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn test_cannot_send_request_to_existing_friend() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (user_a_id, _) = create_test_user(&user_service, "a").await;
    let (user_b_id, _) = create_test_user(&user_service, "b").await;

    let request = SendFriendRequest {
        target_user_id: user_b_id,
        message: None,
    };
    let friend_request = user_service
        .send_friend_request(user_a_id, request)
        .await
        .expect("Failed to send request");

    user_service
        .handle_friend_request(user_b_id, friend_request.id, true)
        .await
        .expect("Failed to accept request");

    let request2 = SendFriendRequest {
        target_user_id: user_b_id,
        message: None,
    };
    let result = user_service.send_friend_request(user_a_id, request2).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn test_accept_friend_request() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (sender_id, _) = create_test_user(&user_service, "sender").await;
    let (receiver_id, _) = create_test_user(&user_service, "receiver").await;

    let request = SendFriendRequest {
        target_user_id: receiver_id,
        message: None,
    };

    let friend_request = user_service
        .send_friend_request(sender_id, request)
        .await
        .expect("Failed to send request");

    user_service
        .handle_friend_request(receiver_id, friend_request.id, true)
        .await
        .expect("Failed to accept request");

    let sender_friends = user_service
        .get_friends(sender_id)
        .await
        .expect("Failed to get friends");
    let receiver_friends = user_service
        .get_friends(receiver_id)
        .await
        .expect("Failed to get friends");

    assert_eq!(sender_friends.len(), 1);
    assert_eq!(receiver_friends.len(), 1);
    assert_eq!(sender_friends[0].friend.id, receiver_id);
    assert_eq!(receiver_friends[0].friend.id, sender_id);
}

#[tokio::test]
async fn test_reject_friend_request() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (sender_id, _) = create_test_user(&user_service, "sender").await;
    let (receiver_id, _) = create_test_user(&user_service, "receiver").await;

    let request = SendFriendRequest {
        target_user_id: receiver_id,
        message: None,
    };

    let friend_request = user_service
        .send_friend_request(sender_id, request)
        .await
        .expect("Failed to send request");

    user_service
        .handle_friend_request(receiver_id, friend_request.id, false)
        .await
        .expect("Failed to reject request");

    let sender_friends = user_service
        .get_friends(sender_id)
        .await
        .expect("Failed to get friends");

    assert!(sender_friends.is_empty());
}

#[tokio::test]
async fn test_cancel_friend_request() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (sender_id, _) = create_test_user(&user_service, "sender").await;
    let (receiver_id, _) = create_test_user(&user_service, "receiver").await;

    let request = SendFriendRequest {
        target_user_id: receiver_id,
        message: None,
    };

    let friend_request = user_service
        .send_friend_request(sender_id, request)
        .await
        .expect("Failed to send request");

    user_service
        .cancel_friend_request(sender_id, friend_request.id)
        .await
        .expect("Failed to cancel request");

    let received_requests = user_service
        .get_received_friend_requests(receiver_id)
        .await
        .expect("Failed to get received requests");

    assert!(received_requests.is_empty());
}

#[tokio::test]
async fn test_get_received_friend_requests() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (receiver_id, _) = create_test_user(&user_service, "receiver").await;
    let (sender1_id, _) = create_test_user(&user_service, "sender1").await;
    let (sender2_id, _) = create_test_user(&user_service, "sender2").await;

    let request1 = SendFriendRequest {
        target_user_id: receiver_id,
        message: Some("From sender1".to_string()),
    };
    let request2 = SendFriendRequest {
        target_user_id: receiver_id,
        message: Some("From sender2".to_string()),
    };

    user_service
        .send_friend_request(sender1_id, request1)
        .await
        .expect("Failed to send request 1");
    user_service
        .send_friend_request(sender2_id, request2)
        .await
        .expect("Failed to send request 2");

    let received = user_service
        .get_received_friend_requests(receiver_id)
        .await
        .expect("Failed to get received requests");

    assert_eq!(received.len(), 2);
}

#[tokio::test]
async fn test_remove_friend() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (user_a_id, _) = create_test_user(&user_service, "a").await;
    let (user_b_id, _) = create_test_user(&user_service, "b").await;

    let request = SendFriendRequest {
        target_user_id: user_b_id,
        message: None,
    };
    let friend_request = user_service
        .send_friend_request(user_a_id, request)
        .await
        .expect("Failed to send request");

    user_service
        .handle_friend_request(user_b_id, friend_request.id, true)
        .await
        .expect("Failed to accept request");

    let friends_before = user_service
        .get_friends(user_a_id)
        .await
        .expect("Failed to get friends");
    assert_eq!(friends_before.len(), 1);

    user_service
        .remove_friend(user_a_id, user_b_id)
        .await
        .expect("Failed to remove friend");

    let friends_after = user_service
        .get_friends(user_a_id)
        .await
        .expect("Failed to get friends");
    assert!(friends_after.is_empty());
}

#[tokio::test]
async fn test_search_users() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (user1_id, _username1) = create_test_user(&user_service, "searchable_alpha").await;
    let (user2_id, _username2) = create_test_user(&user_service, "searchable_beta").await;
    let _ = create_test_user(&user_service, "other_user").await;

    let results = user_service
        .search_users_by_username("searchable", 10)
        .await
        .expect("Failed to search users");

    assert_eq!(results.len(), 2);
    let ids: Vec<_> = results.iter().map(|u| u.id).collect();
    assert!(ids.contains(&user1_id));
    assert!(ids.contains(&user2_id));
}

#[tokio::test]
async fn test_are_friends() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (user_a_id, _) = create_test_user(&user_service, "a").await;
    let (user_b_id, _) = create_test_user(&user_service, "b").await;
    let (user_c_id, _) = create_test_user(&user_service, "c").await;

    let request = SendFriendRequest {
        target_user_id: user_b_id,
        message: None,
    };
    let friend_request = user_service
        .send_friend_request(user_a_id, request)
        .await
        .expect("Failed to send request");

    user_service
        .handle_friend_request(user_b_id, friend_request.id, true)
        .await
        .expect("Failed to accept request");

    assert!(user_service
        .are_friends(user_a_id, user_b_id)
        .await
        .unwrap());
    assert!(!user_service
        .are_friends(user_a_id, user_c_id)
        .await
        .unwrap());
}

#[tokio::test]
async fn test_only_receiver_can_handle_request() {
    let db = setup_test_db().await;
    let user_service = UserService::new(db.clone());

    let (sender_id, _) = create_test_user(&user_service, "sender").await;
    let (receiver_id, _) = create_test_user(&user_service, "receiver").await;
    let (other_id, _) = create_test_user(&user_service, "other").await;

    let request = SendFriendRequest {
        target_user_id: receiver_id,
        message: None,
    };

    let friend_request = user_service
        .send_friend_request(sender_id, request)
        .await
        .expect("Failed to send request");

    let result = user_service
        .handle_friend_request(other_id, friend_request.id, true)
        .await;

    assert!(matches!(result, Err(AppError::Forbidden)));
}
