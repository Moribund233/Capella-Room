use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

use capella_room::{
    config::{AppConfig, ConfigManager, DatabaseConfig, JwtConfig, UploadConfig},
    db::Database,
    models::audit::{AuditEventType, AuditSeverity, CreateAuditLogRequest},
    routes::create_router,
    state::AppState,
    utils::logging::MetricsCollector,
    websocket::manager::WebSocketManager,
};

/// 清理测试数据库
async fn cleanup_database(db: &Database) {
    sqlx::query("DELETE FROM audit_logs")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM audit_alerts")
        .execute(db.pool())
        .await
        .ok();
    sqlx::query("DELETE FROM audit_alert_rules")
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

/// 创建测试应用
async fn create_test_app() -> (Router, Arc<AppState>) {
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
        websocket: Default::default(),
        reconnect: Default::default(),
        logging: Default::default(),
        system: Default::default(),
        admin: Default::default(),
        audit: capella_room::config::AuditConfig {
            enabled: true,
            log_retention_days: 90,
            buffer_size: 10,
            flush_interval_seconds: 1,
            excluded_paths: vec![],
            alert_enabled: true,
            alert_cooldown_minutes: 5,
            auto_archive_enabled: false,
            archive_hour: 3,
        },
        redis: Default::default(),
        batch_message: Default::default(),
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

/// 创建测试用户
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

/// 创建测试管理员
async fn create_test_admin(state: &AppState, username: &str, email: &str) -> (Uuid, String) {
    let password = "AdminPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();

    let user = state
        .user_service()
        .create_admin(username, email, &password_hash)
        .await
        .unwrap();

    let token = state
        .auth_service()
        .generate_token_pair(user.id, &user.username, user.role.clone())
        .unwrap();

    (user.id, token.access_token)
}

/// 创建测试超级管理员
async fn create_test_super_admin(state: &AppState, username: &str, email: &str) -> (Uuid, String) {
    let password = "SuperAdminPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();

    let user = state
        .user_service()
        .create_super_admin(username, email, &password_hash)
        .await
        .unwrap();

    let token = state
        .auth_service()
        .generate_token_pair(user.id, &user.username, user.role.clone())
        .unwrap();

    (user.id, token.access_token)
}

// ==================== 阶段1测试：基础数据模型和服务 ====================

#[tokio::test]
async fn test_audit_service_can_create_log() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) = create_test_user(&state, "audituser1", "audituser1@test.com").await;

    let log_request =
        CreateAuditLogRequest::new(AuditEventType::UserLogin, "user_login", "用户登录成功")
            .with_actor(user_id, capella_room::models::user::UserRole::User)
            .with_metadata(
                capella_room::models::audit::AuditMetadata::new()
                    .with_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
            );

    let result = state.audit_service().log_event(log_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_audit_service_can_query_logs() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) = create_test_user(&state, "audituser2", "audituser2@test.com").await;

    // 创建多条审计日志
    for i in 0..5 {
        let log_request = CreateAuditLogRequest::new(
            AuditEventType::UserLogin,
            "user_login",
            format!("用户登录成功 {}", i),
        )
        .with_actor(user_id, capella_room::models::user::UserRole::User);

        state.audit_service().log_event(log_request).await.unwrap();
    }

    // 手动刷新缓冲区到数据库
    state.audit_service().flush_buffer().await.unwrap();

    // 查询日志
    let query = capella_room::models::audit::AuditLogQuery {
        actor_id: Some(user_id),
        ..Default::default()
    };

    let (logs, total) = state.audit_service().query_logs(query).await.unwrap();
    assert_eq!(total, 5);
    assert_eq!(logs.len(), 5);
}

#[tokio::test]
async fn test_audit_service_can_get_log_by_id() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) = create_test_user(&state, "audituser3", "audituser3@test.com").await;

    // 先创建日志
    let log_request = CreateAuditLogRequest::new(
        AuditEventType::UserRegister,
        "user_register",
        "用户注册成功",
    )
    .with_actor(user_id, capella_room::models::user::UserRole::User);

    state.audit_service().log_event(log_request).await.unwrap();

    // 手动刷新缓冲区到数据库
    state.audit_service().flush_buffer().await.unwrap();

    // 查询所有日志获取ID
    let query = capella_room::models::audit::AuditLogQuery {
        actor_id: Some(user_id),
        event_type: Some(AuditEventType::UserRegister),
        ..Default::default()
    };

    let (logs, total) = state.audit_service().query_logs(query).await.unwrap();
    assert!(total >= 1, "应该有至少1条日志, 实际有 {} 条", total);

    let log_id = logs[0].id;

    // 通过ID查询
    let log = state.audit_service().get_log_by_id(log_id).await.unwrap();
    assert!(log.is_some());

    let log = log.unwrap();
    assert_eq!(log.id, log_id);
    assert_eq!(log.event_type, AuditEventType::UserRegister);
}

#[tokio::test]
async fn test_audit_service_can_log_admin_action() {
    let (_app, state) = create_test_app().await;

    let (admin_id, _token) = create_test_admin(&state, "auditadmin1", "auditadmin1@test.com").await;
    let (target_user_id, _token) =
        create_test_user(&state, "targetuser1", "targetuser1@test.com").await;

    let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1));
    let result = state
        .audit_service()
        .log_admin_action(admin_id, "user_role_change", "user", target_user_id, ip)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_audit_service_can_log_user_login() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) = create_test_user(&state, "audituser4", "audituser4@test.com").await;

    let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1));
    let result = state
        .audit_service()
        .log_user_login(
            user_id,
            capella_room::models::user::UserRole::User,
            ip,
            None,
            true,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_audit_service_can_get_stats() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) = create_test_user(&state, "audituser5", "audituser5@test.com").await;

    // 创建不同严重级别的日志
    for severity in [
        AuditSeverity::Info,
        AuditSeverity::Warning,
        AuditSeverity::Error,
    ] {
        let log_request = CreateAuditLogRequest::new(
            AuditEventType::UserLogin,
            "user_login",
            format!("登录事件 {:?}", severity),
        )
        .with_actor(user_id, capella_room::models::user::UserRole::User)
        .with_severity(severity);

        state.audit_service().log_event(log_request).await.unwrap();
    }

    // 手动刷新缓冲区到数据库
    state.audit_service().flush_buffer().await.unwrap();

    let stats = state
        .audit_service()
        .get_audit_stats(None, None)
        .await
        .unwrap();

    assert!(
        stats.total_logs >= 3,
        "应该有至少3条日志, 实际有 {} 条",
        stats.total_logs
    );
    assert!(!stats.logs_by_severity.is_empty());
}

#[tokio::test]
async fn test_audit_service_can_export_logs() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) = create_test_user(&state, "audituser6", "audituser6@test.com").await;

    // 创建测试日志
    for i in 0..3 {
        let log_request = CreateAuditLogRequest::new(
            AuditEventType::UserLogin,
            "user_login",
            format!("导出测试 {}", i),
        )
        .with_actor(user_id, capella_room::models::user::UserRole::User);

        state.audit_service().log_event(log_request).await.unwrap();
    }

    let query = capella_room::models::audit::AuditLogQuery::default();

    // 测试 JSON 导出
    let json_data = state.audit_service().export_logs_json(query.clone()).await;
    assert!(json_data.is_ok());

    // 测试 CSV 导出
    let csv_data = state.audit_service().export_logs_csv(query).await;
    assert!(csv_data.is_ok());
}

// ==================== 阶段2测试：审计中间件和API ====================

#[tokio::test]
async fn test_admin_can_list_audit_logs() {
    let (app, state) = create_test_app().await;

    // 使用 SuperAdmin 确保有权限访问审计 API
    let (_admin_id, admin_token) =
        create_test_super_admin(&state, "auditadmin2", "auditadmin2@test.com").await;

    // 先创建一些审计日志
    let (user_id, _token) = create_test_user(&state, "audituser7", "audituser7@test.com").await;
    let log_request =
        CreateAuditLogRequest::new(AuditEventType::UserLogin, "user_login", "测试日志")
            .with_actor(user_id, capella_room::models::user::UserRole::User);
    state.audit_service().log_event(log_request).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit/logs")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_can_get_audit_stats() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) =
        create_test_admin(&state, "auditadmin3", "auditadmin3@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit/stats")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_can_query_alerts() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) =
        create_test_admin(&state, "auditadmin4", "auditadmin4@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit/alerts")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_can_list_alert_rules() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) =
        create_test_admin(&state, "auditadmin5", "auditadmin5@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit/rules")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_creates_audit_log() {
    let (app, state) = create_test_app().await;

    // 创建用户
    let password = "TestPassword123";
    let password_hash = state.auth_service().hash_password(password).unwrap();
    let user = state
        .user_service()
        .create_user("logintest", "logintest@test.com", &password_hash)
        .await
        .unwrap();

    // 执行登录
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "logintest@test.com",
                        "password": password
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::OK {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        println!("Login failed with status: {:?}", status);
        println!("Response body: {}", body_str);
    }
    assert_eq!(status, StatusCode::OK);

    // 等待异步审计日志任务完成，然后刷新缓冲区
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    state.audit_service().flush_buffer().await.unwrap();
    // 再等待一下确保 spawned 任务完成
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let query = capella_room::models::audit::AuditLogQuery {
        actor_id: Some(user.id),
        event_type: Some(AuditEventType::UserLogin),
        ..Default::default()
    };

    let (logs, total) = state.audit_service().query_logs(query).await.unwrap();
    assert!(total >= 1, "登录应该创建审计日志，实际找到 {} 条", total);
    assert_eq!(logs[0].event_type, AuditEventType::UserLogin);
}

#[tokio::test]
async fn test_admin_action_creates_audit_log() {
    let (app, state) = create_test_app().await;

    let (_admin_id, admin_token) =
        create_test_super_admin(&state, "superadmin1", "superadmin1@test.com").await;
    let (target_user_id, _token) =
        create_test_user(&state, "targetuser2", "targetuser2@test.com").await;

    // 执行管理员操作（更新用户角色）
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/users/{}/role", target_user_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "role": "admin"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 等待异步审计日志任务完成，然后刷新缓冲区
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    state.audit_service().flush_buffer().await.unwrap();
    // 再等待一下确保 spawned 任务完成
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let query = capella_room::models::audit::AuditLogQuery {
        event_type: Some(AuditEventType::AdminUserRoleChange),
        ..Default::default()
    };

    let (_logs, total) = state.audit_service().query_logs(query).await.unwrap();
    assert!(
        total >= 1,
        "管理员操作应该创建审计日志，实际找到 {} 条",
        total
    );
}

#[tokio::test]
async fn test_non_admin_cannot_access_audit_api() {
    let (app, state) = create_test_app().await;

    let (_user_id, user_token) =
        create_test_user(&state, "normaluser1", "normaluser1@test.com").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/audit/logs")
                .header("Authorization", format!("Bearer {}", user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_audit_log_query_by_event_type() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) = create_test_user(&state, "audituser8", "audituser8@test.com").await;

    // 创建不同类型的事件
    let login_request =
        CreateAuditLogRequest::new(AuditEventType::UserLogin, "user_login", "用户登录")
            .with_actor(user_id, capella_room::models::user::UserRole::User);
    state
        .audit_service()
        .log_event(login_request)
        .await
        .unwrap();

    let register_request =
        CreateAuditLogRequest::new(AuditEventType::UserRegister, "user_register", "用户注册")
            .with_actor(user_id, capella_room::models::user::UserRole::User);
    state
        .audit_service()
        .log_event(register_request)
        .await
        .unwrap();

    // 手动刷新缓冲区到数据库
    state.audit_service().flush_buffer().await.unwrap();
    // 等待确保数据写入
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 按事件类型查询
    let query = capella_room::models::audit::AuditLogQuery {
        event_type: Some(AuditEventType::UserLogin),
        ..Default::default()
    };

    let (logs, total) = state.audit_service().query_logs(query).await.unwrap();
    assert!(total >= 1, "应该有至少1条登录日志, 实际有 {} 条", total);
    assert!(logs
        .iter()
        .all(|log| log.event_type == AuditEventType::UserLogin));
}

#[tokio::test]
async fn test_audit_log_query_by_severity() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) = create_test_user(&state, "audituser9", "audituser9@test.com").await;

    // 创建不同严重级别的日志
    let info_request =
        CreateAuditLogRequest::new(AuditEventType::UserLogin, "user_login", "普通登录")
            .with_actor(user_id, capella_room::models::user::UserRole::User)
            .with_severity(AuditSeverity::Info);
    state.audit_service().log_event(info_request).await.unwrap();

    let error_request = CreateAuditLogRequest::new(
        AuditEventType::SystemLoginFailure,
        "login_failure",
        "登录失败",
    )
    .with_actor(user_id, capella_room::models::user::UserRole::User)
    .with_severity(AuditSeverity::Error);
    state
        .audit_service()
        .log_event(error_request)
        .await
        .unwrap();

    // 手动刷新缓冲区到数据库
    state.audit_service().flush_buffer().await.unwrap();

    // 按严重级别查询
    let query = capella_room::models::audit::AuditLogQuery {
        severity: Some(AuditSeverity::Error),
        ..Default::default()
    };

    let (logs, total) = state.audit_service().query_logs(query).await.unwrap();
    assert!(total >= 1, "应该有至少1条错误日志, 实际有 {} 条", total);
    assert!(logs.iter().all(|log| log.severity == AuditSeverity::Error));
}

/// 测试删除用户后审计日志 FK 约束不会阻断其他日志写入
///
/// 复现场景：
/// 1. 创建用户后硬删除（模拟 admin 删除用户）
/// 2. 用已删除用户的 UUID 创建审计日志（FK 约束会失败）
/// 3. 混合正常日志一起刷入
/// 4. 验证：正常日志不会被事务中毒影响（旧行为会被整个事务 abort 掉）
#[tokio::test]
async fn test_audit_log_fk_violation_does_not_poison_batch() {
    let (_app, state) = create_test_app().await;

    let (user_id, _token) =
        create_test_user(&state, "fk_test_user", "fk_test_user@test.com").await;

    // 硬删除用户（模拟 admin 删除）
    let deleted = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(state.db().pool())
        .await
        .unwrap();
    assert_eq!(deleted.rows_affected(), 1, "用户应该被删除");

    // 第 1 条：actor_id 引用已删除用户 → 会触发 FK 约束失败
    let bad_log = CreateAuditLogRequest::new(
        AuditEventType::UserLogin,
        "login",
        "这条日志的 actor 已被删除",
    )
    .with_actor(user_id, capella_room::models::user::UserRole::User);

    state.audit_service().log_event(bad_log).await.unwrap();

    // 第 2 条：没有 actor_id（合法的日志）
    let good_log = CreateAuditLogRequest::new(
        AuditEventType::SystemLoginFailure,
        "login_failure",
        "这条日志没有 actor_id，应该正常写入",
    );

    state.audit_service().log_event(good_log).await.unwrap();

    // 刷入数据库
    let flush_result = state.audit_service().flush_buffer().await;
    assert!(
        flush_result.is_ok(),
        "flush_buffer 不应因单条 FK 失败而整体失败"
    );

    // 查询——good_log 应该被成功写入
    let (_logs, total) = state
        .audit_service()
        .query_logs(capella_room::models::audit::AuditLogQuery {
            event_type: Some(AuditEventType::SystemLoginFailure),
            ..Default::default()
        })
        .await
        .unwrap_or_else(|_| (vec![], 0));

    assert!(
        total >= 1,
        "正常日志（没有 actor_id）应该被成功写入，但实际查询到 {} 条",
        total
    );
}
