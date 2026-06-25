use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use chrono::Utc;
use std::sync::Arc;

use crate::{
    handlers::{
        account_security, admin, audit, auth, auth_v2, config, custom_event, dlq_admin, file, message,
        message_reaction, notification, oauth, pin_message, room, security, ui_config, user,
        user_settings, webhook,
    },
    middleware::admin::admin_auth_middleware,
    middleware::audit::audit_middleware,
    middleware::auth_middleware,
    middleware::oauth_auth::oauth_auth_middleware,
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
        // 客户端配置（公开访问）
        .route("/api/config/client", get(config::get_client_config))
        // WebSocket 端点
        .route("/ws", get(ws_handler));

    // 认证路由（公开访问）
    let auth_routes_router = Router::new()
        .nest(&format!("/api/{}/auth/", API_VERSION), auth_routes())
        .nest("/api/auth/", auth_routes())
        .nest("/api/v2/auth/", v2_auth_routes());

    // v1 注册端点（需要管理员权限）
    let register_admin_router = Router::new()
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/auth/register", post(auth::register))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            admin_auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            auth_middleware,
        ));

    // 创建受保护路由（需要认证）
    let protected_routes = Router::new()
        // 用户路由
        .nest(&format!("/api/{}/users/", API_VERSION), user_routes())
        .nest("/api/users/", user_routes())
        .nest("/api/v2/users/", v2_user_routes())
        // 聊天室路由
        .nest(&format!("/api/{}/rooms/", API_VERSION), room_routes())
        .nest("/api/rooms/", room_routes())
        // 消息路由
        .nest(&format!("/api/{}/messages/", API_VERSION), message_routes())
        .nest("/api/messages/", message_routes())
        // 通知路由
        .nest(
            &format!("/api/{}/notifications", API_VERSION),
            notification_routes(),
        )
        .nest("/api/notifications", notification_routes())
        // 文件路由
        .nest(&format!("/api/{}/files/", API_VERSION), file_routes())
        .nest("/api/files", file_routes())
        .nest(&format!("/api/{}/upload/", API_VERSION), upload_routes())
        .nest("/api/upload", upload_routes())
        // UI 配置路由
        .nest(&format!("/api/{}/ui/", API_VERSION), ui_config_routes())
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
        .nest(&format!("/api/{}/admin/", API_VERSION), admin_router())
        .nest("/api/admin/", admin_router())
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

    // 创建 OAuth API 路由（接受 OAuth access_token 或 CapellaRoom JWT）
    let oauth_api_routes = Router::new()
        .merge(oauth_protected_routes())
        .merge(room_resource_routes())
        .merge(webhook_routes())
        .merge(custom_event_routes())
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            oauth_auth_middleware,
        ));

    // 合并所有路由
    public_routes
        .merge(auth_routes_router)
        .merge(oauth_browser_routes())   // /oauth/authorize, /oauth/token
        .route("/oauth/userinfo", get(oauth::userinfo))  // OAuth token auth (no app auth middleware)
        .merge(register_admin_router)
        .merge(protected_routes)
        .merge(admin_routes)
        .merge(oauth_api_routes)
        .with_state(state)
}

/// 认证路由（公开访问）
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh_token))
}

/// UI 配置路由（需要认证）
fn ui_config_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/config", get(ui_config::get_user_config))
        .route("/config", post(ui_config::save_user_config))
        .route("/config", delete(ui_config::reset_user_config))
}

/// 用户路由
fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        // 当前用户相关
        .route("/me", get(user::get_current_user))
        .route("/me", put(user::update_user))
        .route("/me", delete(user::delete_current_user))
        .route("/me/password", put(user::change_password))
        .route("/me/stats", get(user::get_user_stats))
        .route("/me/rooms", get(room::get_my_rooms))
        // 用户设置
        .route("/me/settings", get(user_settings::get_user_settings))
        .route("/me/settings", patch(user_settings::update_user_settings))
        .route("/me/settings", delete(user_settings::reset_user_settings))
        // 账号安全
        .route(
            "/me/security/overview",
            get(account_security::get_security_overview),
        )
        .route("/me/devices", get(account_security::list_devices))
        .route(
            "/me/devices",
            delete(account_security::terminate_all_other_devices),
        )
        .route(
            "/me/devices/:device_id",
            delete(account_security::terminate_device),
        )
        .route(
            "/me/devices/:device_id/block",
            post(account_security::block_device),
        )
        .route(
            "/me/devices/:device_id/unblock",
            post(account_security::unblock_device),
        )
        .route(
            "/me/login-history",
            get(account_security::get_login_history),
        )
        .route(
            "/me/login-history/suspicious",
            get(account_security::get_suspicious_logins),
        )
        // 房间级设置
        .route("/me/rooms/settings", get(user_settings::list_room_settings))
        .route(
            "/me/rooms/:room_id/settings",
            get(user_settings::get_room_settings)
                .patch(user_settings::update_room_settings)
                .delete(user_settings::delete_room_settings),
        )
        .route("/logout", post(user::logout))
        // 用户列表和详情
        .route("/", get(user::list_users))
        .route("/:user_id", get(user::get_user_by_id))
        // 搜索用户
        .route("/search", get(user::search_users))
        // 推荐用户（在线优先 + 随机补充）
        .route("/recommended", get(user::get_recommended_users))
        // 好友功能
        .route("/friends", get(user::get_friends))
        .route("/friends/requests", post(user::send_friend_request))
        .route(
            "/friends/requests/received",
            get(user::get_received_friend_requests),
        )
        .route(
            "/friends/requests/sent",
            get(user::get_sent_friend_requests),
        )
        .route(
            "/friends/requests/handle",
            post(user::handle_friend_request),
        )
        .route(
            "/friends/requests/:request_id",
            delete(user::cancel_friend_request),
        )
        .route("/friends/:friend_id", delete(user::remove_friend))
}

/// 聊天室路由
fn room_routes() -> Router<Arc<AppState>> {
    Router::new()
        // 聊天室列表和创建
        .route("/", get(room::list_rooms).post(room::create_room))
        // 最近更新的聊天室列表
        .route("/recent", get(room::list_recent_rooms))
        // 私聊房间
        .route("/direct", post(room::create_direct_room))
        .route("/direct/list", get(room::get_direct_rooms))
        // 通过邀请码加入房间（公开端点，需要认证）
        .route("/join-by-invite", post(room::join_by_invite))
        // 验证邀请码（公开端点，需要认证）
        .route("/validate-invite", get(room::validate_invite_code))
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
        // 邀请管理
        .route(
            "/:room_id/invitations",
            get(room::get_room_invitations).post(room::create_invitation),
        )
        .route(
            "/:room_id/invitations/:invitation_id",
            delete(room::revoke_invitation),
        )
        // 消息
        .route("/:room_id/messages", get(message::get_room_messages))
        // 置顶消息
        .route(
            "/:room_id/pinned-messages",
            get(pin_message::get_room_pinned_messages),
        )
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
        // 消息反应
        .route(
            "/:message_id/reactions",
            get(message_reaction::get_message_reactions)
                .post(message_reaction::add_reaction)
                .delete(message_reaction::remove_reaction),
        )
        // 消息置顶
        .route("/:message_id/pin", post(pin_message::pin_message))
        .route("/:message_id/pin", delete(pin_message::unpin_message))
}

/// 通知路由
fn notification_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(notification::get_notifications))
        .route("/unread-count", get(notification::get_unread_count))
        .route("/:id/read", post(notification::mark_as_read))
        .route("/read-all", post(notification::mark_all_as_read))
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
        // 分片上传
        .route("/chunked/init", post(file::init_chunked_upload))
        .route("/chunked/:session_id/:chunk_index", post(file::upload_chunk))
        .route("/chunked/:session_id/status", get(file::get_upload_status))
        .route("/chunked/:session_id/complete", post(file::complete_chunked_upload))
        .route("/chunked/:session_id", delete(file::cancel_chunked_upload))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
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
        .route(
            "/users/:user_id/password",
            put(admin::admin_reset_user_password),
        )
        // 房间管理
        .route("/rooms", get(admin::list_rooms))
        .route(
            "/rooms/:room_id",
            get(admin::get_room).delete(admin::delete_room),
        )
        .route("/rooms/:room_id/messages", get(admin::get_room_messages))
        // 房间成员管理（管理员专用）
        .route(
            "/rooms/:room_id/members/:user_id",
            delete(admin::kick_room_member),
        )
        .route(
            "/rooms/:room_id/members/:user_id/role",
            put(admin::set_room_member_role),
        )
        // 消息审核
        .route("/messages", get(admin::list_messages))
        .route("/messages/:message_id", delete(admin::delete_message))
        // 系统统计
        .route("/stats", get(admin::get_stats))
        .route("/stats/activity", get(admin::get_activity_stats))
        .route("/stats/performance", get(admin::get_performance_metrics))
        // 增强统计接口
        .route("/stats/users/growth", get(admin::get_user_growth_stats))
        .route("/stats/users/behavior", get(admin::get_user_behavior_stats))
        .route("/stats/users/friends", get(admin::get_friend_stats))
        .route("/stats/rooms/activity", get(admin::get_room_activity_ranking))
        .route("/stats/rooms/overview", get(admin::get_room_stats))
        .route("/stats/messages/types", get(admin::get_message_type_stats))
        .route("/stats/messages/hourly", get(admin::get_message_hourly_distribution))
        .route("/stats/security", get(admin::get_security_stats))
        // 系统监控
        .route("/monitor", get(admin::get_monitor_data))
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
        // IP 安全路由
        .nest("/security", security_routes())
        // 待办通知管理
        .route("/pending-actions/:id/respond", post(admin::respond_pending_action))
        // 死信队列管理
        .route("/dlq/messages", get(dlq_admin::list_dlq_messages))
        .route("/dlq/stats", get(dlq_admin::dlq_stats))
        .route("/dlq/batch-requeue", post(dlq_admin::batch_requeue_dlq))
        .route("/dlq/:id/requeue", post(dlq_admin::requeue_dlq_message))
        .route("/dlq/:id", delete(dlq_admin::delete_dlq_message))
        // Redis 管理路由
        .route("/redis/status", get(admin::get_redis_status))
        .route("/redis/stats", get(admin::get_redis_stats))
        .route("/redis/refresh", post(admin::refresh_redis))
        // 配置同步路由
        .route("/config/sync", post(admin::trigger_config_sync))
        .route("/config/sync/status", get(admin::get_config_sync_status))
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

/// IP 安全路由
fn security_routes() -> Router<Arc<AppState>> {
    Router::new()
        // IP 列表管理
        .route(
            "/ip-list",
            get(security::list_ip_entries).post(security::add_ip_entry),
        )
        .route("/ip-list/batch", post(security::batch_add_ip_entries))
        .route(
            "/ip-list/:id",
            put(security::update_ip_entry).delete(security::remove_ip_entry),
        )
        // IP 检查
        .route("/ip-check", post(security::check_ip))
        // 统计信息
        .route("/stats", get(security::get_security_stats))
        // 缓存管理
        .route("/refresh-cache", post(security::refresh_cache))
        // 清理过期条目
        .route("/cleanup-expired", post(security::cleanup_expired))
        // 白名单模式
        .route("/whitelist-mode", get(security::get_whitelist_mode))
        .route("/whitelist-mode", post(security::set_whitelist_mode))
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
    let result: Result<(i32,), _> = sqlx::query_as("SELECT 1")
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
            "name": "Capella Room API",
            "description": "Real-time chat room API",
            "deprecated_routes": ["/api/*"],
            "recommended_routes": ["/api/v1/*"]
        }
    }))
}

/// v2 认证路由（公开访问）
fn v2_auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register/send-code", post(auth_v2::register_send_code))
        .route("/register", post(auth_v2::register))
        .route("/login/send-code", post(auth_v2::login_send_code))
        .route("/login", post(auth_v2::login))
        .route("/reset-password/send-code", post(auth_v2::reset_password_send_code))
        .route("/reset-password", post(auth_v2::reset_password))
        .route("/login-with-password", post(auth_v2::login_with_password))
        .route("/refresh", post(auth_v2::refresh_token))
}

/// 用户登出路由（需要认证）
fn v2_user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/logout", post(auth_v2::logout))
}

/// OAuth 浏览器路由（公开访问）
fn oauth_browser_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/oauth/authorize", get(oauth::authorize_get).post(oauth::authorize_post))
        .route("/oauth/authorize/consent", post(oauth::authorize_consent))
        .route("/oauth/token", post(oauth::token))
}

/// OAuth 受保护路由（需要认证）
fn oauth_protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/oauth/apps", post(oauth::create_app).get(oauth::list_apps))
        .route("/oauth/apps/:app_id", get(oauth::get_app).put(oauth::update_app).delete(oauth::delete_app))
        .route("/oauth/apps/:app_id/rotate-secret", post(oauth::rotate_secret))
        .route("/oauth/mappings", post(oauth::create_mapping).get(oauth::lookup_mapping))
        .route("/oauth/mappings/:mapping_id", delete(oauth::delete_mapping_handler))
        .route("/oauth/resources", get(oauth::lookup_resource).post(oauth::bind_resource_auto_create))
}

/// 房间资源绑定路由（需要认证）
fn room_resource_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/rooms/:room_id/resources", post(oauth::bind_resource).get(oauth::list_bindings))
        .route("/rooms/:room_id/resources/:binding_id", put(oauth::update_binding).delete(oauth::unbind_resource))
}

/// Webhook 订阅路由（需要认证）
fn webhook_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhook/subscriptions", post(webhook::create_subscription).get(webhook::list_subscriptions))
        .route("/webhook/subscriptions/:subscription_id", get(webhook::list_subscriptions).put(webhook::update_subscription).delete(webhook::delete_subscription))
        .route("/webhook/subscriptions/:subscription_id/deliveries", get(webhook::get_deliveries))
        .route("/webhook/subscriptions/:subscription_id/deliveries/:delivery_id/redeliver", post(webhook::redeliver))
}

/// 自定义事件 HTTP API 路由（需要认证）
fn custom_event_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/rooms/:room_id/custom-events", post(custom_event::send_custom_event).get(custom_event::get_missed_events))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_constant() {
        assert_eq!(API_VERSION, "v1");
    }
}
