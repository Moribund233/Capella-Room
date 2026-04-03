use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use chrono::Utc;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::{
    handlers::{auth, file, message, room, user},
    middleware::auth_middleware,
    middleware::rate_limit::{rate_limit_middleware, strict_rate_limit_middleware},
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
        // API 版本信息
        .route("/api/version", get(api_version))
        // WebSocket 端点
        .route("/ws", get(ws_handler));

    // 认证路由（使用严格的速率限制）
    let auth_routes_router = Router::new()
        .nest(&format!("/api/{}/auth", API_VERSION), auth_routes())
        .nest("/api/auth", auth_routes())
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            strict_rate_limit_middleware,
        ));

    // 创建受保护路由（需要认证 + 速率限制）
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
        // 添加速率限制中间件（在认证之前）
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            rate_limit_middleware,
        ))
        // 添加认证中间件
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            auth_middleware,
        ));

    // 合并所有路由
    public_routes
        .merge(auth_routes_router)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// 认证路由（公开访问）
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh_token))
}

/// 用户路由
fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        // 当前用户相关
        .route("/me", get(user::get_current_user))
        .route("/me", put(user::update_user))
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
