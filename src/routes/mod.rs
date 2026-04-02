use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::{
    handlers::{auth, message, room, user},
    state::AppState,
    websocket::handler::ws_handler,
};

/// API 版本
pub const API_VERSION: &str = "v1";

/// 构建应用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(health_check))
        // API 版本信息
        .route("/api/version", get(api_version))
        // WebSocket端点
        .route("/ws", get(ws_handler))
        // API v1 路由
        .nest(&format!("/api/{}", API_VERSION), api_v1_routes())
        // 保持向后兼容的未版本化路由（建议客户端迁移到版本化路由）
        .nest("/api", api_v1_routes())
        // 静态文件服务（可选）
        // .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// API v1 路由组
fn api_v1_routes() -> Router<Arc<AppState>> {
    Router::new()
        // 认证路由
        .nest("/auth", auth_routes())
        // 用户路由
        .nest("/users", user_routes())
        // 聊天室路由
        .nest("/rooms", room_routes())
        // 消息路由
        .nest("/messages", message_routes())
}

/// 认证路由
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh_token))
}

/// 用户路由
fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(user::get_current_user))
        .route("/me", put(user::update_user))
        .route("/", get(user::list_users))
}

/// 聊天室路由
fn room_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(room::list_rooms).post(room::create_room))
        .route("/:room_id", get(room::get_room))
        .route("/:room_id/join", post(room::join_room))
        .route("/:room_id/leave", post(room::leave_room))
        .route("/:room_id/members", get(room::get_room_members))
        .route("/:room_id/messages", get(message::get_room_messages))
}

/// 消息路由
fn message_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search", get(message::search_messages))
        .route("/:message_id", delete(message::delete_message))
}

/// 健康检查
async fn health_check() -> &'static str {
    "OK"
}

/// API 版本信息
async fn api_version() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "version": API_VERSION,
        "name": "Seredeli Room API",
        "description": "Real-time chat room API",
        "deprecated_routes": ["/api/*"],
        "recommended_routes": ["/api/v1/*"]
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
