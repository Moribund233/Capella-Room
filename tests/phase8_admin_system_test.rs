use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

use seredeli_room::{
    config::{AppConfig, ConfigManager, DatabaseConfig, JwtConfig, UploadConfig},
    db::Database,
    routes::create_router,
    state::AppState,
    utils::logging::MetricsCollector,
    websocket::manager::WebSocketManager,
};

async fn cleanup_database(db: &Database) {
    sqlx::query("DELETE FROM messages")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM room_members")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM rooms")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM users")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM system_configs")
        .execute(db.pool())
        .await
        .ok();
}

async fn create_test_app() -> (Router, Arc<AppState>) {
    dotenvy::from_filename(".env.test").ok();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://test:test123@localhost:5432/seredeli_room_test".to_string()
    });

    let db = Database::new(&DatabaseConfig {
        url: Some(database_url.clone()),
        max_connections: 5,
        acquire_timeout_secs: 30,
        idle_timeout_secs: 600,
    })
    .await
    .expect("Failed to connect to test database");

    db.migrate().await.expect("Failed to run migrations");
    cleanup_database(&db).await;

    let ws_manager = WebSocketManager::new();
    let metrics_collector = Arc::new(MetricsCollector::new());

    let config = AppConfig {
        app: Default::default(),
        server: Default::default(),
        database: DatabaseConfig {
            url: Some(database_url),
            max_connections: 5,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,
        },
        jwt: JwtConfig {
            secret: Some("test_secret_key_for_testing_purposes_only".to_string()),
            expiration_hours: 24,
        },
        upload: UploadConfig {
            max_file_size: 10 * 1024 * 1024,
            base_url: "/uploads".to_string(),
        },
        rate_limit: Default::default(),
        websocket: Default::default(),
        reconnect: Default::default(),
        logging: Default::default(),
        cors: Default::default(),
        system: Default::default(),
        admin: Default::default(),
    };
    let config_manager = ConfigManager::new(db.clone(), config.clone());

    let state = AppState::new(
        db,
        ws_manager,
        config,
        metrics_collector,
        Arc::new(config_manager),
    )
    .expect("Failed to create app state");

    let app = create_router(Arc::clone(&state));

    (app, state)
}

async fn create_test_user(state: &AppState, username: &str, email: &str) -> (Uuid, String) {
    let password = "TestPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();

    let user = state
        .user_service()
        .create_user(username, email, &password_hash)
        .await
        .unwrap();

    let token = state.auth_service().generate_token_pair(user.id).unwrap();

    (user.id, token.access_token)
}

async fn create_test_admin(state: &AppState, username: &str, email: &str) -> (Uuid, String) {
    let password = "AdminPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();

    let user = state
        .user_service()
        .create_admin(username, email, &password_hash)
        .await
        .unwrap();

    let token = state.auth_service().generate_token_pair(user.id).unwrap();

    (user.id, token.access_token)
}

async fn create_test_super_admin(state: &AppState, username: &str, email: &str) -> (Uuid, String) {
    let password = "SuperAdminPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();

    let user = state
        .user_service()
        .create_super_admin(username, email, &password_hash)
        .await
        .unwrap();

    let token = state.auth_service().generate_token_pair(user.id).unwrap();

    (user.id, token.access_token)
}

#[tokio::test]
async fn test_admin_can_list_users() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) = create_test_admin(&state, "admin1", "admin1@test.com").await;
    let (_user_id, _user_token) = create_test_user(&state, "user1", "user1@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/users")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_non_admin_cannot_access_admin_routes() {
    let (app, state) = create_test_app().await;

    let (_user_id, user_token) = create_test_user(&state, "user2", "user2@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/users")
                .header("Authorization", format!("Bearer {}", user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_can_update_user_role() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) = create_test_admin(&state, "admin2", "admin2@test.com").await;
    let (user_id, _user_token) = create_test_user(&state, "user3", "user3@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/users/{}/role", user_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"role": "admin"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let updated_user = state
        .user_service()
        .get_user_by_id(user_id)
        .await
        .unwrap()
        .unwrap();
    assert!(updated_user.role.is_admin());
}

#[tokio::test]
async fn test_admin_cannot_demote_super_admin() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) = create_test_admin(&state, "admin3", "admin3@test.com").await;
    let (super_admin_id, _super_admin_token) =
        create_test_super_admin(&state, "superadmin1", "superadmin1@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/users/{}/role", super_admin_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"role": "user"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_cannot_manage_other_admin() {
    let (app, state) = create_test_app().await;

    let (_admin1_id, admin1_token) = create_test_admin(&state, "admin7", "admin7@test.com").await;
    let (admin2_id, _admin2_token) = create_test_admin(&state, "admin8", "admin8@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/users/{}/role", admin2_id))
                .header("Authorization", format!("Bearer {}", admin1_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"role": "user"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_cannot_delete_other_admin() {
    let (app, state) = create_test_app().await;

    let (_admin1_id, admin1_token) = create_test_admin(&state, "admin9", "admin9@test.com").await;
    let (admin2_id, _admin2_token) = create_test_admin(&state, "admin10", "admin10@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/admin/users/{}", admin2_id))
                .header("Authorization", format!("Bearer {}", admin1_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_super_admin_can_manage_admin() {
    let (app, state) = create_test_app().await;

    let (_super_admin_id, super_admin_token) =
        create_test_super_admin(&state, "superadmin3", "superadmin3@test.com").await;
    let (admin_id, _admin_token) = create_test_admin(&state, "admin11", "admin11@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/users/{}/role", admin_id))
                .header("Authorization", format!("Bearer {}", super_admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"role": "user"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let updated_user = state
        .user_service()
        .get_user_by_id(admin_id)
        .await
        .unwrap()
        .unwrap();
    assert!(!updated_user.role.is_admin());
}

#[tokio::test]
async fn test_admin_can_delete_user() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) = create_test_admin(&state, "admin4", "admin4@test.com").await;
    let (user_id, _user_token) = create_test_user(&state, "user4", "user4@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/admin/users/{}", user_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let deleted_user = state.user_service().get_user_by_id(user_id).await.unwrap();
    assert!(deleted_user.is_none());
}

#[tokio::test]
async fn test_admin_can_list_configs() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) = create_test_admin(&state, "admin5", "admin5@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/configs")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_cannot_update_config() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) = create_test_admin(&state, "admin6", "admin6@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/configs/jwt.expiration_hours")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"value": "48"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_super_admin_can_update_config() {
    let (app, state) = create_test_app().await;

    let (_super_admin_id, super_admin_token) =
        create_test_super_admin(&state, "superadmin2", "superadmin2@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/configs/jwt.expiration_hours")
                .header("Authorization", format!("Bearer {}", super_admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"value": "48"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let config = state
        .config_manager()
        .get_config_by_key("jwt.expiration_hours")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(config.value, "48");
}

#[tokio::test]
async fn test_user_role_permissions() {
    use seredeli_room::models::user::UserRole;

    let user_role = UserRole::User;
    let admin_role = UserRole::Admin;
    let super_admin_role = UserRole::SuperAdmin;

    assert!(!user_role.is_admin());
    assert!(!user_role.is_super_admin());

    assert!(admin_role.is_admin());
    assert!(!admin_role.is_super_admin());

    assert!(super_admin_role.is_admin());
    assert!(super_admin_role.is_super_admin());

    assert!(super_admin_role.can_manage_user(&admin_role));
    assert!(super_admin_role.can_manage_user(&user_role));
    assert!(super_admin_role.can_manage_user(&super_admin_role));

    assert!(admin_role.can_manage_user(&user_role));
    assert!(!admin_role.can_manage_user(&admin_role));
    assert!(!admin_role.can_manage_user(&super_admin_role));

    assert!(!user_role.can_manage_user(&user_role));
    assert!(!user_role.can_manage_user(&admin_role));
    assert!(!user_role.can_manage_user(&super_admin_role));
}

#[tokio::test]
async fn test_super_admin_initialization() {
    let (_app, state) = create_test_app().await;

    let has_super_admin_before = state.user_service().has_super_admin().await.unwrap();
    assert!(
        !has_super_admin_before,
        "No super admin should exist before creation"
    );

    let password_hash = state
        .auth_service()
        .hash_password("AdminPassword123")
        .unwrap();
    let _admin = state
        .user_service()
        .create_super_admin("initial_admin", "initial@admin.com", &password_hash)
        .await
        .unwrap();

    let has_super_admin_after = state.user_service().has_super_admin().await.unwrap();
    assert!(
        has_super_admin_after,
        "Super admin should exist after creation"
    );
}
