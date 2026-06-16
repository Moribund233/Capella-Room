use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::util::ServiceExt;
use uuid::Uuid;

use capella_room::{
    config::{AppConfig, AuditConfig, BatchMessageConfig, ConfigManager, DatabaseConfig, JwtConfig, UploadConfig},
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

async fn create_test_user(state: &AppState, username: &str, email: &str) -> (Uuid, String) {
    let password = "TestPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();

    let user = state
        .user_service()
        .create_user(username, email, &password_hash)
        .await
        .unwrap();

    let token = state
        .auth_service()
        .generate_token_pair(user.id, &user.username, user.role.clone())
        .unwrap();

    (user.id, token.access_token)
}

#[tokio::test]
async fn test_self_delete_account_success() {
    let (app, state, _guard) = create_test_app().await;

    let (user_id, token) = create_test_user(&state, "selfdel1", "selfdel1@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/users/me")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let user = state
        .user_service()
        .get_user_by_id(user_id)
        .await
        .unwrap()
        .expect("User should still exist in DB (soft delete)");

    assert!(!user.is_active, "User should be deactivated after self-delete");
}

#[tokio::test]
async fn test_self_delete_account_requires_auth() {
    let (app, _state, _guard) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/users/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_self_delete_account_anonymizes_data() {
    let (app, state, _guard) = create_test_app().await;

    let (user_id, token) = create_test_user(&state, "anontest", "anontest@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/users/me")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let user = state
        .user_service()
        .get_user_by_id(user_id)
        .await
        .unwrap()
        .expect("User should still exist");

    assert_ne!(user.username, "anontest", "Username should be anonymized");
    assert_ne!(user.email, "anontest@test.com", "Email should be anonymized");
    assert!(user.avatar_url.is_none() || user.avatar_url.as_deref() == Some(""), "Avatar should be cleared");
}

#[tokio::test]
async fn test_self_delete_account_login_fails() {
    let (app, state, _guard) = create_test_app().await;

    let (user_id, token) = create_test_user(&state, "loginfail", "loginfail@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/users/me")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let user = state
        .user_service()
        .get_user_by_id(user_id)
        .await
        .unwrap()
        .expect("User should still exist");

    let password_valid = state
        .auth_service()
        .verify_password("TestPassword123", &user.password_hash)
        .unwrap_or(false);

    assert!(!password_valid, "Old password should no longer be valid after account deletion");
}
