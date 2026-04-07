use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use chrono::Utc;
use std::sync::Arc;

use crate::{
    handlers::{admin, audit, auth, file, message, room, user},
    middleware::admin::admin_auth_middleware,
    middleware::audit::audit_middleware,
    middleware::auth_middleware,
    state::AppState,
    websocket::handler::ws_handler,
};

/// API 版本
pub const API_VERSION: &str = "v1";

/// 构建应用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    // 创建公开路由（不需要认证）
    let public_routes = Router::new()
        // 健康检查
        .route("/health", get(health_check))
        .route("/health/detail", get(health_check_detailed))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        // API 版本信息
        .route("/api/version", get(api_version))
        // WebSocket 端点
        .route("/ws", get(ws_handler));

    // 认证路由（公开访问）
    let auth_routes_router = Router::new()
        .nest(&format!("/api/{}/auth", API_VERSION), auth_routes())
        .nest("/api/auth", auth_routes());

    // 创建受保护路由（需要认证）
    let protected_routes = Router::new()
        // 用户路由
        .nest(&format!("/api/{}/users", API_VERSION), user_routes())
        .nest("/api/users", user_routes())
        // 聊天室路由
        .nest(&format!("/api/{}/rooms", API_VERSION), room_routes())
        .nest("/api/rooms", room_routes())
        // 消息路由
        .nest(&format!("/api/{}/messages", API_VERSION), message_routes())
        .nest("/api/messages", message_routes())
        // 文件路由
        .nest(&format!("/api/{}/files", API_VERSION), file_routes())
        .nest(&format!("/api/{}/upload", API_VERSION), upload_routes())
        // 添加审计中间件
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            audit_middleware,
        ))
        // 添加认证中间件
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            auth_middleware,
        ));

    // 创建管理员路由（需要管理员权限）
    let admin_routes = Router::new()
        .nest(&format!("/api/{}/admin", API_VERSION), admin_router())
        .nest("/api/admin", admin_router())
        // 添加管理员认证中间件
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            admin_auth_middleware,
        ))
        // 添加审计中间件
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            audit_middleware,
        ))
        // 添加基础认证中间件
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            auth_middleware,
        ));

    // 合并所有路由
    public_routes
        .merge(auth_routes_router)
        .merge(protected_routes)
        .merge(admin_routes)
        .with_state(state)
}

/// 认证路由（公开访问）
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(user::logout))
        .route("/refresh", post(auth::refresh_token))
}

/// 用户路由
fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        // 当前用户相关
        .route("/me", get(user::get_current_user))
        .route("/me", put(user::update_user))
        .route("/me/password", put(user::change_password))
        .route("/me/rooms", get(room::get_my_rooms))
        // 用户列表和详情
        .route("/", get(user::list_users))
        .route("/:user_id", get(user::get_user_by_id))
}

/// 聊天室路由
fn room_routes() -> Router<Arc<AppState>> {
    Router::new()
        // 聊天室列表和创建
        .route("/", get(room::list_rooms).post(room::create_room))
        // 最近更新的聊天室列表
        .route("/recent", get(room::list_recent_rooms))
        // 聊天室详情、更新、删除
        .route(
            "/:room_id",
            get(room::get_room)
                .put(room::update_room)
                .delete(room::delete_room),
        )
        // 加入/离开聊天室
        .route("/:room_id/join", post(room::join_room))
        .route("/:room_id/leave", delete(room::leave_room))
        // 成员管理
        .route("/:room_id/members", get(room::get_room_members))
        .route("/:room_id/members/:user_id", delete(room::kick_member))
        .route(
            "/:room_id/members/:user_id/role",
            put(room::set_member_role),
        )
        // 消息
        .route("/:room_id/messages", get(message::get_room_messages))
}

/// 消息路由
fn message_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search", get(message::search_messages))
        .route(
            "/:message_id",
            put(message::edit_message).delete(message::delete_message),
        )
        .route(
            "/:message_id/history",
            get(message::get_message_edit_history),
        )
}

/// 文件路由
fn file_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(file::list_files))
        .route("/:file_id", get(file::get_file).delete(file::delete_file))
}

/// 上传路由
fn upload_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(file::upload_file))
        .route("/image", post(file::upload_image))
        .route("/avatar", post(file::upload_avatar))
}

/// 管理员路由
fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        // 用户管理
        .route("/users", get(admin::list_users))
        .route(
            "/users/:user_id",
            get(admin::get_user).delete(admin::delete_user),
        )
        .route("/users/:user_id/role", put(admin::update_user_role))
        .route("/users/:user_id/status", put(admin::set_user_status))
        // 房间管理
        .route("/rooms", get(admin::list_rooms))
        .route(
            "/rooms/:room_id",
            get(admin::get_room).delete(admin::delete_room),
        )
        .route("/rooms/:room_id/messages", get(admin::get_room_messages))
        // 消息审核
        .route("/messages", get(admin::list_messages))
        .route("/messages/:message_id", delete(admin::delete_message))
        // 系统统计
        .route("/stats", get(admin::get_stats))
        .route("/stats/activity", get(admin::get_activity_stats))
        // 日志查看
        .route("/logs", get(admin::list_logs))
        .route("/logs/download", get(admin::download_logs))
        // 系统配置管理
        .route(
            "/configs",
            get(admin::list_configs).post(admin::reset_configs),
        )
        .route(
            "/configs/:key",
            get(admin::get_config).put(admin::update_config),
        )
        // 审计系统路由
        .nest("/audit", audit_routes())
}

/// 审计系统路由
fn audit_routes() -> Router<Arc<AppState>> {
    Router::new()
        // 审计日志
        .route("/logs", get(audit::list_audit_logs))
        .route("/logs/:id", get(audit::get_audit_log_detail))
        .route("/stats", get(audit::get_audit_stats))
        .route("/export", get(audit::export_audit_logs))
        // 告警管理
        .route("/alerts", get(audit::list_alerts))
        .route("/alerts/:id/status", put(audit::update_alert_status))
        // 告警规则
        .route("/rules", get(audit::list_alert_rules))
        .route("/rules/:id", put(audit::update_alert_rule))
        // 日志清理
        .route("/cleanup", post(audit::cleanup_audit_logs))
}

/// 健康检查
async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "success": true,
        "data": {
            "status": "healthy",
            "timestamp": Utc::now().to_rfc3339()
        }
    }))
}

/// 详细健康检查（包含数据库和WebSocket状态）
async fn health_check_detailed(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    let db_healthy = check_database_health(&state).await;
    let ws_connections = state.ws_manager().get_connection_count();
    let online_users = state.ws_manager().get_online_user_count();

    let status = if db_healthy { "healthy" } else { "degraded" };

    axum::Json(serde_json::json!({
        "success": true,
        "data": {
            "status": status,
            "timestamp": Utc::now().to_rfc3339(),
            "components": {
                "database": {
                    "status": if db_healthy { "healthy" } else { "unhealthy" }
                },
                "websocket": {
                    "status": "healthy",
                    "connections": ws_connections,
                    "online_users": online_users
                }
            }
        }
    }))
}

/// 就绪检查（Readiness Probe）
async fn readiness_check(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    let db_ready = check_database_health(&state).await;

    if db_ready {
        axum::Json(serde_json::json!({
            "success": true,
            "data": {
                "status": "ready",
                "timestamp": Utc::now().to_rfc3339()
            }
        }))
    } else {
        axum::Json(serde_json::json!({
            "success": false,
            "data": {
                "status": "not_ready",
                "timestamp": Utc::now().to_rfc3339(),
                "reason": "database unavailable"
            }
        }))
    }
}

/// 存活检查（Liveness Probe）
async fn liveness_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "success": true,
        "data": {
            "status": "alive",
            "timestamp": Utc::now().to_rfc3339()
        }
    }))
}

/// 检查数据库健康状态
async fn check_database_health(state: &AppState) -> bool {
    let result: Result<(i64,), _> = sqlx::query_as("SELECT 1")
        .fetch_one(state.db().pool())
        .await;
    result.is_ok()
}

/// API 版本信息
async fn api_version() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "success": true,
        "data": {
            "version": API_VERSION,
            "name": "Seredeli Room API",
            "description": "Real-time chat room API",
            "deprecated_routes": ["/api/*"],
            "recommended_routes": ["/api/v1/*"]
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_constant() {
        assert_eq!(API_VERSION, "v1");
    }
}
