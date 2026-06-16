use std::sync::Arc;

use axum::Router;
use capella_room::{
    config::{
        AppConfig, BatchMessageConfig, ConfigManager, DatabaseConfig,
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
        jwt: capella_room::config::JwtConfig {
            secret: Some("test_secret_key_for_testing_purposes_only".to_string()),
            expiration_hours: 24,
        },
        upload: capella_room::config::UploadConfig {
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
        audit: capella_room::config::AuditConfig {
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
    sqlx::query("DELETE FROM verification_codes")
        .execute(db.pool())
        .await
        .ok();
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

#[tokio::test]
async fn test_new_user_has_email_verified_false() {
    let (_app, state, _guard) = create_test_app().await;

    let password_hash = state.auth_service().hash_password("TestPassword123").unwrap();
    let user = state
        .user_service()
        .create_user("verifyuser", "verify@test.com", &password_hash)
        .await
        .unwrap();

    // 直接查询数据库验证 email_verified 字段
    let row: (bool,) = sqlx::query_as(
        "SELECT email_verified FROM users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_one(state.db.pool())
    .await
    .unwrap();

    assert!(!row.0, "新用户的 email_verified 应该为 false");
}

#[tokio::test]
async fn test_user_struct_has_email_verified_field() {
    let (_app, state, _guard) = create_test_app().await;

    let password_hash = state.auth_service().hash_password("TestPassword123").unwrap();
    let user = state
        .user_service()
        .create_user("structuser", "struct@test.com", &password_hash)
        .await
        .unwrap();

    // 通过 User 结构体验证（需要 FromRow 正确映射）
    assert!(!user.email_verified, "新用户的 email_verified 应该为 false");
    assert!(user.email_verified_at.is_none(), "新用户的 email_verified_at 应该为 None");

    // 通过 get_user_by_id 验证
    let fetched = state
        .user_service()
        .get_user_by_id(user.id)
        .await
        .unwrap()
        .expect("用户应该存在");
    assert!(!fetched.email_verified, "取回的用户 email_verified 应该为 false");
    assert!(fetched.email_verified_at.is_none(), "取回的用户 email_verified_at 应该为 None");
}
