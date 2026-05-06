use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::util::ServiceExt;
use uuid::Uuid;

use seredeli_room::{
    config::{AppConfig, AuditConfig, ConfigManager, DatabaseConfig, JwtConfig, UploadConfig},
    db::Database,
    routes::create_router,
    state::AppState,
    utils::logging::MetricsCollector,
    websocket::manager::WebSocketManager,
};

async fn cleanup_database(db: &Database) {
    sqlx::query("DELETE FROM user_room_settings")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM user_settings")
        .execute(db.pool())
        .await
        .ok();
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
        websocket: seredeli_room::config::WebSocketConfig {
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
            enabled: false,
            log_retention_days: 90,
            buffer_size: 1000,
            flush_interval_seconds: 60,
            excluded_paths: vec![],
            alert_enabled: false,
            alert_cooldown_minutes: 10,
            auto_archive_enabled: false,
            archive_hour: 3,
        },
        redis: Default::default(),
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

    let token = state
        .auth_service()
        .generate_token_pair(user.id, &user.username, user.role.clone())
        .unwrap();

    (user.id, token.access_token)
}

async fn create_test_room(
    state: &AppState,
    token: &str,
    name: &str,
) -> (Uuid, Uuid) {
    let claims = state.auth_service().verify_token(token).unwrap();
    let user_id: Uuid = claims.sub.parse().unwrap();

    let room = state
        .room_service()
        .create_room(name, Some("test room"), user_id, false, 100)
        .await
        .unwrap();

    (user_id, room.id)
}

async fn parse_body(response: axum::response::Response) -> Value {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body_bytes).unwrap()
}

// ============================================================
// 用户整体设置测试
// ============================================================

#[tokio::test]
async fn test_get_settings_returns_defaults() {
    let (app, state) = create_test_app().await;
    let (_user_id, token) = create_test_user(&state, "set_user1", "set_user1@test.com").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me/settings")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    assert!(body["success"].as_bool().unwrap());

    let data = &body["data"];
    assert_eq!(data["notification"]["private_message"], true);
    assert_eq!(data["notification"]["mentioned"], true);
    assert_eq!(data["privacy"]["online_status_visibility"], "everyone");
    assert_eq!(data["language"]["language"], "zh-CN");
    assert_eq!(data["accessibility"]["font_size"], "medium");
    assert_eq!(data["media"]["image_quality"], "high");
}

#[tokio::test]
async fn test_patch_settings_updates_specific_groups() {
    let (app, state) = create_test_app().await;
    let (_user_id, token) = create_test_user(&state, "set_user2", "set_user2@test.com").await;

    // Patch notification + privacy groups
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/users/me/settings")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "notification": {
                            "private_message": false,
                            "sound_enabled": false,
                            "desktop_notification": false
                        },
                        "privacy": {
                            "online_status_visibility": "nobody",
                            "allow_stranger_message": false
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;

    // Updated groups should reflect new values
    assert_eq!(body["data"]["notification"]["private_message"], false);
    assert_eq!(body["data"]["notification"]["sound_enabled"], false);
    assert_eq!(body["data"]["privacy"]["online_status_visibility"], "nobody");
    assert_eq!(body["data"]["privacy"]["allow_stranger_message"], false);

    // Unchanged fields in updated groups should still be defaults
    assert_eq!(body["data"]["notification"]["mentioned"], true);
    assert_eq!(body["data"]["privacy"]["profile_visibility"], "everyone");

    // Unchanged groups should remain defaults
    assert_eq!(body["data"]["message"]["message_preview"], true);
    assert_eq!(body["data"]["language"]["language"], "zh-CN");
    assert_eq!(body["data"]["accessibility"]["font_size"], "medium");
    assert_eq!(body["data"]["media"]["auto_download_media"], true);
}

#[tokio::test]
async fn test_patch_settings_no_side_effects_on_other_groups() {
    let (app, state) = create_test_app().await;
    let (_user_id, token) = create_test_user(&state, "set_user3", "set_user3@test.com").await;

    // Only update language
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/users/me/settings")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "language": {
                            "language": "en-US",
                            "timezone": "America/New_York",
                            "time_format": "12h"
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;

    // Language updated
    assert_eq!(body["data"]["language"]["language"], "en-US");
    assert_eq!(body["data"]["language"]["timezone"], "America/New_York");
    assert_eq!(body["data"]["language"]["time_format"], "12h");

    // Other groups untouched
    assert_eq!(body["data"]["notification"]["private_message"], true);
    assert_eq!(body["data"]["privacy"]["online_status_visibility"], "everyone");
    assert_eq!(body["data"]["message"]["message_preview"], true);
    assert_eq!(body["data"]["accessibility"]["font_size"], "medium");
    assert_eq!(body["data"]["media"]["auto_download_media"], true);
}

#[tokio::test]
async fn test_settings_persist_across_calls() {
    let (app, state) = create_test_app().await;
    let (_user_id, token) = create_test_user(&state, "set_user4", "set_user4@test.com").await;

    // Update privacy
    app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/users/me/settings")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "privacy": {
                            "online_status_visibility": "friends",
                            "allow_room_invitation": false
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // GET again — should reflect persisted changes
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me/settings")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    assert_eq!(body["data"]["privacy"]["online_status_visibility"], "friends");
    assert_eq!(body["data"]["privacy"]["allow_room_invitation"], false);
    assert_eq!(body["data"]["privacy"]["profile_visibility"], "everyone");
}

#[tokio::test]
async fn test_settings_require_auth() {
    let (app, _state) = create_test_app().await;

    // No auth header
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================
// 用户房间级设置测试
// ============================================================

#[tokio::test]
async fn test_room_settings_defaults() {
    let (app, state) = create_test_app().await;
    let (_user_id, token) = create_test_user(&state, "rset_user1", "rset_user1@test.com").await;
    let (_user_id, room_id) = create_test_room(&state, &token, "room-defaults-test").await;

    // Get room settings — should return defaults
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    assert_eq!(body["data"]["room_id"], room_id.to_string());
    assert_eq!(body["data"]["is_muted"], false);
    assert_eq!(body["data"]["is_pinned"], false);
    assert_eq!(body["data"]["notification_preference"], "all");
    assert!(body["data"]["custom_name"].is_null());
    assert!(body["data"]["custom_color"].is_null());
}

#[tokio::test]
async fn test_room_settings_update_and_get() {
    let (app, state) = create_test_app().await;
    let (_user_id, token) = create_test_user(&state, "rset_user2", "rset_user2@test.com").await;
    let (_user_id, room_id) = create_test_room(&state, &token, "room-update-test").await;

    // Update room settings
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "is_muted": true,
                        "is_pinned": true,
                        "notification_preference": "mention_only",
                        "custom_name": "My Room",
                        "custom_color": "#00ff00"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    assert_eq!(body["data"]["is_muted"], true);
    assert_eq!(body["data"]["is_pinned"], true);
    assert_eq!(body["data"]["notification_preference"], "mention_only");
    assert_eq!(body["data"]["custom_name"], "My Room");
    assert_eq!(body["data"]["custom_color"], "#00ff00");

    // GET should return updated values
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    assert_eq!(body["data"]["is_muted"], true);
    assert_eq!(body["data"]["custom_name"], "My Room");
}

#[tokio::test]
async fn test_room_settings_delete_resets_to_default() {
    let (app, state) = create_test_app().await;
    let (_user_id, token) = create_test_user(&state, "rset_user3", "rset_user3@test.com").await;
    let (_user_id, room_id) = create_test_room(&state, &token, "room-delete-test").await;

    // Set some values first
    app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "is_muted": true, "is_pinned": true }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Delete (reset)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // After delete GET should return defaults
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    assert_eq!(body["data"]["is_muted"], false);
    assert_eq!(body["data"]["is_pinned"], false);
}

#[tokio::test]
async fn test_list_room_settings() {
    let (app, state) = create_test_app().await;
    let (_user_id, token) = create_test_user(&state, "rset_user4", "rset_user4@test.com").await;

    let (_u1, room1) = create_test_room(&state, &token, "list-room1").await;
    let (_u2, room2) = create_test_room(&state, &token, "list-room2").await;
    let (_u3, room3) = create_test_room(&state, &token, "list-room3").await;

    // Pin room1, mute room2, add default settings for room3
    app.clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room1))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "is_pinned": true }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    app.clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room2))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "is_muted": true }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    app.clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room3))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // List should return 3 settings
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me/rooms/settings")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    let settings = body["data"].as_array().unwrap();
    assert_eq!(settings.len(), 3);

    // room1 should be pinned
    let room1_settings = settings
        .iter()
        .find(|s| s["room_id"] == room1.to_string())
        .unwrap();
    assert_eq!(room1_settings["is_pinned"], true);
    assert_eq!(room1_settings["is_muted"], false);

    // room2 should be muted
    let room2_settings = settings
        .iter()
        .find(|s| s["room_id"] == room2.to_string())
        .unwrap();
    assert_eq!(room2_settings["is_muted"], true);
    assert_eq!(room2_settings["is_pinned"], false);

    // room3 should be plain defaults
    let room3_settings = settings
        .iter()
        .find(|s| s["room_id"] == room3.to_string())
        .unwrap();
    assert_eq!(room3_settings["is_muted"], false);
    assert_eq!(room3_settings["is_pinned"], false);
}

#[tokio::test]
async fn test_room_settings_isolated_per_user() {
    let (app, state) = create_test_app().await;
    let (_user1_id, token1) = create_test_user(&state, "rset_user5a", "rset_user5a@test.com").await;
    let (_user2_id, token2) = create_test_user(&state, "rset_user5b", "rset_user5b@test.com").await;

    let (_owner_id, room_id) = create_test_room(&state, &token1, "isolation-room").await;

    // User1 mutes the room
    app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room_id))
                .header("Authorization", format!("Bearer {}", token1))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "is_muted": true }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // User2's settings for the same room should be default (not muted)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/users/me/rooms/{}/settings", room_id))
                .header("Authorization", format!("Bearer {}", token2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    assert_eq!(body["data"]["is_muted"], false);
}
