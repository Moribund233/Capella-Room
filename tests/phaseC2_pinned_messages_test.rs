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
    sqlx::query("DELETE FROM pinned_messages").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM message_reactions").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM messages").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM room_members").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM rooms").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM users").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM system_configs").execute(db.pool()).await.ok();
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

async fn create_test_room(state: &AppState, name: &str, owner_id: Uuid) -> Uuid {
    state
        .room_service()
        .create_room(name, Some("Test room"), owner_id, false, 100)
        .await
        .unwrap()
        .id
}

async fn create_test_message(state: &AppState, room_id: Uuid, sender_id: Uuid, content: &str) -> Uuid {
    state
        .message_service()
        .create_text_message(room_id, sender_id, content, None)
        .await
        .unwrap()
        .id
}

#[tokio::test]
async fn test_pin_message_success() {
    let (app, state, _guard) = create_test_app().await;

    let (user_id, token) = create_test_user(&state, "pinuser1", "pinuser1@test.com").await;
    let room_id = create_test_room(&state, "Pin Test Room", user_id).await;
    let message_id = create_test_message(&state, room_id, user_id, "Message to pin").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/messages/{}/pin", message_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_pin_message_requires_auth() {
    let (app, state, _guard) = create_test_app().await;

    let (user_id, _) = create_test_user(&state, "pinuser2", "pinuser2@test.com").await;
    let room_id = create_test_room(&state, "Pin Auth Test", user_id).await;
    let message_id = create_test_message(&state, room_id, user_id, "Auth test message").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/messages/{}/pin", message_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unpin_message_success() {
    let (app, state, _guard) = create_test_app().await;

    let (user_id, token) = create_test_user(&state, "pinuser3", "pinuser3@test.com").await;
    let room_id = create_test_room(&state, "Unpin Test Room", user_id).await;
    let message_id = create_test_message(&state, room_id, user_id, "Message to unpin").await;

    // Pin first
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/messages/{}/pin", message_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();
    let pin_resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(pin_resp.status(), StatusCode::OK);

    // Then unpin
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/messages/{}/pin", message_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let unpin_resp = app.oneshot(req).await.unwrap();

    assert_eq!(unpin_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_pinned_messages() {
    let (app, state, _guard) = create_test_app().await;

    let (user_id, token) = create_test_user(&state, "pinuser4", "pinuser4@test.com").await;
    let room_id = create_test_room(&state, "List Pinned Room", user_id).await;
    let msg1 = create_test_message(&state, room_id, user_id, "Pinned msg 1").await;
    let msg2 = create_test_message(&state, room_id, user_id, "Pinned msg 2").await;

    // Pin both messages
    for msg_id in [msg1, msg2] {
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/messages/{}/pin", msg_id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // List pinned messages
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/rooms/{}/pinned-messages", room_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
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

    let pinned = body.get("data");
    assert!(pinned.is_some(), "Response should contain data field with pinned messages");
}

#[tokio::test]
async fn test_pin_nonexistent_message_returns_not_found() {
    let (app, state, _guard) = create_test_app().await;

    let (_, token) = create_test_user(&state, "pinuser5", "pinuser5@test.com").await;
    let fake_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/messages/{}/pin", fake_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
