use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::Value;
use tower::util::ServiceExt;
use uuid::Uuid;

use capella_room::{
    config::{
        AppConfig, AuditConfig, BatchMessageConfig, ConfigManager, DatabaseConfig, JwtConfig,
        UploadConfig,
    },
    db::Database,
    routes::create_router,
    state::AppState,
    test_helpers,
    utils::logging::MetricsCollector,
    websocket::manager::WebSocketManager,
};

async fn create_test_app() -> (Router, Arc<AppState>, tokio::sync::MutexGuard<'static, ()>) {
    let guard = test_helpers::db_guard().lock().await;
    dotenvy::from_filename(".env.test").ok();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://test:test123@localhost:5432/capella_room_test".to_string()
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
        websocket: capella_room::config::WebSocketConfig {
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 60,
            auth_timeout_secs: 10,
            message_buffer_size: 100,
        },
        reconnect: Default::default(),
        logging: Default::default(),
        system: Default::default(),
        admin: Default::default(),
        audit: AuditConfig {
            flush_interval_seconds: 60,
            ..Default::default()
        },
        redis: Default::default(),
        batch_message: BatchMessageConfig {
            batch_size: 50,
            flush_interval_ms: 5000,
            max_queue_size: 1000,
        },
        mail: Default::default(),
        oauth: Default::default(),
        webhook: Default::default(),
    };
    let config_manager = ConfigManager::new(db.clone(), config.clone(), None);

    let state = AppState::new(
        db,
        ws_manager,
        config,
        metrics_collector,
        Arc::new(config_manager),
        None,
    )
    .await
    .expect("Failed to create app state");

    let app = create_router(Arc::clone(&state));

    (app, state, guard)
}

async fn cleanup_database(db: &Database) {
    sqlx::query("DELETE FROM pinned_messages")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM message_reactions")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM messages").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM room_members")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM rooms").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM users").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM system_configs")
        .execute(db.pool())
        .await
        .ok();
}

async fn create_user_token(state: &AppState, username: &str, email: &str, is_admin: bool) -> (Uuid, String) {
    let password = "TestPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();

    let user = state
        .user_service()
        .create_user(username, email, &password_hash)
        .await
        .unwrap();

    let (actual_role, actual_username) = if is_admin {
        sqlx::query("UPDATE users SET role = 'admin' WHERE id = $1")
            .bind(user.id)
            .execute(state.db.pool())
            .await
            .unwrap();
        (capella_room::models::user::UserRole::Admin, user.username.clone())
    } else {
        (user.role.clone(), user.username.clone())
    };

    let token = state
        .auth_service()
        .generate_token_pair(user.id, &actual_username, actual_role)
        .unwrap();

    (user.id, token.access_token)
}

#[tokio::test]
async fn test_v1_register_without_auth_returns_401() {
    let (app, _state, _guard) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "username": "newuser",
                        "email": "newuser@test.com",
                        "password": "TestPass123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(!body["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_v1_register_with_user_role_returns_403() {
    let (app, state, _guard) = create_test_app().await;
    let (_user_id, token) = create_user_token(&state, "regularuser", "regular@test.com", false).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "username": "newuser",
                        "email": "newuser@test.com",
                        "password": "TestPass123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["code"], "FORBIDDEN");
    assert!(!body["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_v1_register_with_admin_role_returns_200() {
    let (app, state, _guard) = create_test_app().await;
    let (_admin_id, admin_token) =
        create_user_token(&state, "adminuser", "admin@test.com", true).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "username": "newuser2",
                        "email": "newuser2@test.com",
                        "password": "TestPass123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(body["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_v1_login_still_public() {
    let (app, state, _guard) = create_test_app().await;
    let password = "TestPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();
    state
        .user_service()
        .create_user("loginuser", "loginuser@test.com", &password_hash)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "email": "loginuser@test.com",
                        "password": "TestPassword123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["access_token"].as_str().is_some());
}
