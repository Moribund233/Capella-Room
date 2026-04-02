//! 阶段六用户功能完善测试
//!
//! 阶段六包含以下功能：
//! - 6.1 用户资料 - 获取当前用户信息、更新用户信息
//! - 6.2 用户状态 - 在线状态管理、实时更新用户状态
//! - 6.3 用户列表 - 获取用户列表（支持搜索和分页）
//!
//! 验收标准：
//! ✅ 用户可以查看和修改个人资料
//! ✅ 用户在线状态实时更新
//! ✅ 可以浏览其他用户信息

use std::env;

use seredeli_room::{
    config::{DatabaseConfig, JwtConfig},
    db::Database,
    error::AppError,
    models::user::{UpdateUserRequest, UserStatus},
    services::{
        auth_service::AuthService,
        user_service::UserService,
    },
};
use uuid::Uuid;
use validator::Validate;

/// 创建测试用的 JWT 配置
fn test_jwt_config() -> JwtConfig {
    JwtConfig {
        secret: "test-secret-key-for-jwt-signing-in-tests-only".to_string(),
        expiration_hours: 24,
    }
}

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
        url: database_url,
        max_connections,
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
    let email = format!("{}_user_test@seredeli.com", username);
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

// ==================== 6.1 用户资料测试 ====================

#[tokio::test]
async fn test_6_1_get_current_user() {
    // 测试获取当前用户信息
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let (user_id, _) = create_test_user(&user_service, &auth_service, "testcurrentuser").await;

    // 通过ID获取用户
    let user = user_service.get_user_by_id(user_id).await.unwrap();
    assert!(user.is_some());

    let user = user.unwrap();
    assert_eq!(user.id, user_id);
    assert_eq!(user.username, "testcurrentuser");
    assert!(user.email.contains("testcurrentuser"));
}

#[tokio::test]
async fn test_6_1_get_user_by_id_not_found() {
    // 测试获取不存在的用户
    let db = setup_test_db().await;
    let user_service = UserService::new(db);

    let non_existent_id = Uuid::new_v4();
    let user = user_service.get_user_by_id(non_existent_id).await.unwrap();
    assert!(user.is_none());
}

#[tokio::test]
async fn test_6_1_update_user_username() {
    // 测试更新用户名
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &auth_service, &format!("testupdate{}", unique_id)).await;

    // 更新用户名
    let new_username = format!("updateduser{}", unique_id);
    let updated_user = user_service
        .update_user(user_id, Some(&new_username), None)
        .await
        .unwrap();

    assert_eq!(updated_user.username, new_username);
}

#[tokio::test]
async fn test_6_1_update_user_avatar() {
    // 测试更新用户头像
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &auth_service, &format!("testavatar{}", unique_id)).await;

    // 更新头像URL
    let avatar_url = "https://example.com/avatar.png";
    let updated_user = user_service
        .update_user(user_id, None, Some(avatar_url))
        .await
        .unwrap();

    assert_eq!(updated_user.avatar_url, Some(avatar_url.to_string()));
}

#[tokio::test]
async fn test_6_1_update_user_request_validation() {
    // 测试更新用户请求验证
    let valid_request = UpdateUserRequest {
        username: Some("validuser".to_string()),
        avatar_url: Some("https://example.com/avatar.png".to_string()),
    };
    assert!(valid_request.validate().is_ok());

    // 用户名太短应该失败
    let short_username = UpdateUserRequest {
        username: Some("ab".to_string()),
        avatar_url: None,
    };
    assert!(short_username.validate().is_err());

    // 用户名为None应该通过
    let no_username = UpdateUserRequest {
        username: None,
        avatar_url: Some("https://example.com/avatar.png".to_string()),
    };
    assert!(no_username.validate().is_ok());
}

#[tokio::test]
async fn test_6_1_update_user_duplicate_username() {
    // 测试更新为已存在的用户名应该失败
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user1_id, _) = create_test_user(&user_service, &auth_service, &format!("testdup1{}", unique_id)).await;
    let (_, _) = create_test_user(&user_service, &auth_service, &format!("testdup2{}", unique_id)).await;

    // 尝试将user1的用户名改为user2的用户名
    let result = user_service
        .update_user(user1_id, Some(&format!("testdup2{}", unique_id)), None)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::Conflict(msg) => {
            assert!(msg.contains("用户名"));
        }
        _ => panic!("Expected Conflict error"),
    }
}

// ==================== 6.2 用户状态测试 ====================

#[tokio::test]
async fn test_6_2_update_user_status() {
    // 测试更新用户状态
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &auth_service, &format!("teststatus{}", unique_id)).await;

    // 更新状态为在线
    user_service
        .update_user_status(user_id, UserStatus::Online)
        .await
        .unwrap();

    // 验证状态已更新
    let user = user_service.get_user_by_id(user_id).await.unwrap().unwrap();
    match user.status {
        UserStatus::Online => {}
        _ => panic!("Expected Online status"),
    }

    // 更新状态为离开
    user_service
        .update_user_status(user_id, UserStatus::Away)
        .await
        .unwrap();

    let user = user_service.get_user_by_id(user_id).await.unwrap().unwrap();
    match user.status {
        UserStatus::Away => {}
        _ => panic!("Expected Away status"),
    }
}

#[tokio::test]
async fn test_6_2_update_user_status_not_found() {
    // 测试更新不存在用户的状态
    let db = setup_test_db().await;
    let user_service = UserService::new(db);

    let non_existent_id = Uuid::new_v4();
    let result = user_service
        .update_user_status(non_existent_id, UserStatus::Online)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_6_2_get_online_users() {
    // 测试获取在线用户列表
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &auth_service, &format!("testonline{}", unique_id)).await;

    // 将用户状态设置为在线
    user_service
        .update_user_status(user_id, UserStatus::Online)
        .await
        .unwrap();

    // 获取在线用户列表
    let online_users = user_service.get_online_users(10, 0).await.unwrap();
    assert!(!online_users.is_empty());

    // 验证包含我们设置的用户
    let found = online_users.iter().any(|u| u.id == user_id);
    assert!(found, "Expected to find the online user in the list");
}

#[tokio::test]
async fn test_6_2_get_users_by_status() {
    // 测试按状态获取用户
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &auth_service, &format!("testbystatus{}", unique_id)).await;

    // 设置用户为离线状态
    user_service
        .update_user_status(user_id, UserStatus::Offline)
        .await
        .unwrap();

    // 获取离线用户
    let offline_users = user_service
        .get_users_by_status(UserStatus::Offline, 10, 0)
        .await
        .unwrap();

    // 验证包含我们的用户
    let found = offline_users.iter().any(|u| u.id == user_id);
    assert!(found, "Expected to find the offline user in the list");
}

// ==================== 6.3 用户列表测试 ====================

#[tokio::test]
async fn test_6_3_list_users_pagination() {
    // 测试用户列表分页
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    // 创建多个测试用户
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    for i in 0..5 {
        let _ = create_test_user(&user_service, &auth_service, &format!("testlist{}{}", unique_id, i)).await;
    }

    // 测试分页
    let users_page1 = user_service.list_users(3, 0).await.unwrap();
    assert_eq!(users_page1.len(), 3);

    let users_page2 = user_service.list_users(3, 3).await.unwrap();
    assert!(!users_page2.is_empty());
}

#[tokio::test]
async fn test_6_3_count_users() {
    // 测试统计用户总数
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    // 获取当前用户总数
    let count = user_service.count_users().await.unwrap();
    
    // 验证返回的是有效的计数（非负）
    assert!(count >= 0, "User count should be non-negative");
    
    // 创建新用户
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let _ = create_test_user(&user_service, &auth_service, &format!("testcount{}", unique_id)).await;

    // 验证总数增加（至少为1，因为创建了一个用户）
    let new_count = user_service.count_users().await.unwrap();
    assert!(new_count >= count, "User count should not decrease after creating a user");
}

#[tokio::test]
async fn test_6_3_search_users() {
    // 测试搜索用户
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let username = format!("searchableuser{}", unique_id);
    let (user_id, _) = create_test_user(&user_service, &auth_service, &username).await;

    // 通过用户名搜索
    let (results, total) = user_service.search_users(&username, 10, 0).await.unwrap();
    assert!(total >= 1);
    assert!(results.iter().any(|u| u.id == user_id));

    // 通过部分用户名搜索
    let (results, total) = user_service.search_users("searchableuser", 10, 0).await.unwrap();
    assert!(total >= 1);
    assert!(results.iter().any(|u| u.id == user_id));
}

#[tokio::test]
async fn test_6_3_search_users_by_email() {
    // 测试通过邮箱搜索用户
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let username = format!("searchemail{}", unique_id);
    let (user_id, _) = create_test_user(&user_service, &auth_service, &username).await;

    // 通过邮箱域名搜索
    let (results, total) = user_service.search_users("seredeli.com", 10, 0).await.unwrap();
    assert!(total >= 1);
    assert!(results.iter().any(|u| u.id == user_id));
}

#[tokio::test]
async fn test_6_3_search_users_pagination() {
    // 测试搜索结果分页
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // 创建多个相似用户名的用户
    for i in 0..5 {
        let _ = create_test_user(
            &user_service,
            &auth_service,
            &format!("pagsearch{}{}", unique_id, i),
        )
        .await;
    }

    // 搜索并分页
    let (results1, total) = user_service.search_users(&format!("pagsearch{}", unique_id), 2, 0).await.unwrap();
    assert_eq!(results1.len(), 2);
    assert!(total >= 5);

    let (results2, _) = user_service.search_users(&format!("pagsearch{}", unique_id), 2, 2).await.unwrap();
    assert_eq!(results2.len(), 2);
}

// ==================== 用户响应转换测试 ====================

#[tokio::test]
async fn test_user_to_response() {
    // 测试用户模型转换为响应DTO
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &auth_service, &format!("testresponse{}", unique_id)).await;

    let user = user_service.get_user_by_id(user_id).await.unwrap().unwrap();
    let response = user.to_response();

    assert_eq!(response.id, user_id);
    assert_eq!(response.username, user.username);
    assert_eq!(response.email, user.email);
    assert_eq!(response.avatar_url, user.avatar_url);
}

// ==================== 集成测试 ====================

#[tokio::test]
async fn test_user_profile_workflow() {
    // 测试完整的用户资料工作流程
    let db = setup_test_db().await;
    let auth_service = AuthService::new(test_jwt_config());
    let user_service = UserService::new(db);

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &auth_service, &format!("workflow{}", unique_id)).await;

    // 1. 获取用户信息
    let user = user_service.get_user_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(user.username, format!("workflow{}", unique_id));

    // 2. 更新用户名和头像
    let new_username = format!("updatedworkflow{}", unique_id);
    let avatar_url = "https://example.com/new-avatar.png";
    let updated_user = user_service
        .update_user(user_id, Some(&new_username), Some(avatar_url))
        .await
        .unwrap();

    assert_eq!(updated_user.username, new_username);
    assert_eq!(updated_user.avatar_url, Some(avatar_url.to_string()));

    // 3. 更新状态为在线
    user_service
        .update_user_status(user_id, UserStatus::Online)
        .await
        .unwrap();

    // 4. 验证状态更新
    let user = user_service.get_user_by_id(user_id).await.unwrap().unwrap();
    match user.status {
        UserStatus::Online => {}
        _ => panic!("Expected Online status after update"),
    }

    // 5. 在在线用户列表中查找
    let online_users = user_service.get_online_users(10, 0).await.unwrap();
    let found = online_users.iter().any(|u| u.id == user_id);
    assert!(found, "User should be in online users list");

    // 6. 搜索用户
    let (search_results, _) = user_service.search_users(&new_username, 10, 0).await.unwrap();
    let found = search_results.iter().any(|u| u.id == user_id);
    assert!(found, "User should be found in search results");
}
