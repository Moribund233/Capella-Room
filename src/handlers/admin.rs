use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    Extension, Json,
};

use serde::{Deserialize, Serialize};
use tracing::warn;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::{
    config::SystemConfigItem,
    error::{AppError, Result},
    middleware::admin::{CurrentUserId, CurrentUserRole},
    models::{
        message::MessageResponse,
        response::ApiResponse,
        room::{MemberRole, RoomResponse},
        user::{UserResponse, UserRole},
    },
    redis::ConfigSyncMessage,
    state::AppState,
    websocket::protocol::{PendingActionInfo, PendingActionStatus, PendingActionType, WebSocketMessage},
};

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct SetUserStatusRequest {
    pub disabled: bool,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<UserListResponse>>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let (users, total) = if let Some(search) = query.search {
        state
            .user_service()
            .search_users(&search, page_size, offset)
            .await?
    } else {
        let users = state.user_service().list_users(page_size, offset).await?;
        let total = state.user_service().count_users().await?;
        (users, total)
    };

    let user_responses: Vec<UserResponse> = users.into_iter().map(|u| u.to_response()).collect();

    Ok(Json(ApiResponse::success(UserListResponse {
        users: user_responses,
        total,
        page,
        page_size,
    })))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    let user = state
        .user_service()
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(user.to_response())))
}

pub async fn update_user_role(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Path(user_id): Path<Uuid>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    let new_role = match request.role.as_str() {
        "user" => UserRole::User,
        "admin" => UserRole::Admin,
        "super_admin" => UserRole::SuperAdmin,
        _ => return Err(AppError::Validation("Invalid role".to_string())),
    };

    let target_user = state
        .user_service()
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if target_user.role.is_super_admin() && new_role != UserRole::SuperAdmin {
        return Err(AppError::Forbidden);
    }

    if target_user.role.is_admin() && !current_role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    if new_role == UserRole::SuperAdmin && !current_role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    let user = state
        .user_service()
        .update_user_role(user_id, new_role)
        .await?;

    // 记录管理员操作审计日志
    let ip = connect_info
        .map(|ci| ci.0.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(admin_id, "user_role_change", "user", user_id, ip)
            .await;
    });

    Ok(Json(ApiResponse::success(user.to_response())))
}

pub async fn set_user_status(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(user_id): Path<Uuid>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
    Json(request): Json<SetUserStatusRequest>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    let target_user = state
        .user_service()
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if target_user.role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    if target_user.role.is_admin() && !current_role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    let user = state
        .user_service()
        .set_user_disabled(user_id, request.disabled)
        .await?;

    // 记录管理员操作审计日志
    let ip = addr.ip();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(admin_id, "user_disable", "user", user_id, ip)
            .await;
    });

    Ok(Json(ApiResponse::success(user.to_response())))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Path(user_id): Path<Uuid>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
) -> Result<StatusCode> {
    let target_user = state
        .user_service()
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if target_user.role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    if target_user.role.is_admin() && !current_role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    state.user_service().delete_user(user_id).await?;

    // 记录管理员操作审计日志
    let ip = connect_info
        .map(|ci| ci.0.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(admin_id, "user_delete", "user", user_id, ip)
            .await;
    });

    Ok(StatusCode::NO_CONTENT)
}

pub async fn admin_reset_user_password(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Path(user_id): Path<Uuid>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
    Json(request): Json<AdminResetPasswordRequest>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    if !current_role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    let target_user = state
        .user_service()
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if target_user.role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    if request.new_password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    let password_hash = state.auth_service().hash_password(&request.new_password)?;

    state
        .user_service()
        .update_password(user_id, &password_hash)
        .await?;

    let ip = connect_info
        .map(|ci| ci.0.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));

    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(admin_id, "password_reset", "user", user_id, ip)
            .await;
    });

    Ok(Json(ApiResponse::success(target_user.to_response())))
}

#[derive(Debug, Deserialize)]
pub struct ListConfigsQuery {
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminResetPasswordRequest {
    pub new_password: String,
}

pub async fn list_configs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListConfigsQuery>,
) -> Result<Json<ApiResponse<Vec<SystemConfigItem>>>> {
    let configs = if let Some(category) = query.category {
        state
            .config_manager()
            .get_configs_by_category(&category)
            .await?
    } else {
        state.config_manager().get_all_configs().await?
    };

    Ok(Json(ApiResponse::success(configs)))
}

pub async fn get_config(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Json<ApiResponse<SystemConfigItem>>> {
    let config = state
        .config_manager()
        .get_config_by_key(&key)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(config)))
}

pub async fn update_config(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Path(key): Path<String>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<ApiResponse<SystemConfigItem>>> {
    if !current_role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    let config = state
        .config_manager()
        .set_config(&key, &request.value)
        .await?;

    // 记录管理员操作审计日志
    let ip = connect_info
        .map(|ci| ci.0.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(admin_id, "config_update", "config", Uuid::nil(), ip)
            .await;
    });

    // 如果配置项不支持热更新，创建待办通知提醒管理员重启
    if !config.is_hot_reloadable {
        let notification_service = Arc::clone(&state.notification_service);
        let action_info = PendingActionInfo {
            notification_id: Uuid::new_v4(),
            action_type: "config_reload".to_string(),
            title: "配置变更需要重启生效".to_string(),
            description: format!(
                "配置项 {} 已修改为 {}，该配置需要重启服务才能生效",
                key, request.value
            ),
            deadline: Some(Utc::now() + chrono::Duration::days(7)),
            action_status: PendingActionStatus::Pending,
            related_config_key: Some(key.clone()),
            related_config_value: Some(request.value.clone()),
            created_at: Utc::now(),
        };
        tokio::spawn(async move {
            if let Err(e) = notification_service.send_pending_action(admin_id, action_info).await {
                warn!("发送配置重载待办通知失败: {}", e);
            }
        });
    }

    Ok(Json(ApiResponse::success(config)))
}

pub async fn reset_configs(
    State(state): State<Arc<AppState>>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
) -> Result<Json<ApiResponse<Vec<SystemConfigItem>>>> {
    if !current_role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    let configs = state.config_manager().reset_to_defaults().await?;

    Ok(Json(ApiResponse::success(configs)))
}

// ==================== 房间管理接口 ====================

#[derive(Debug, Deserialize)]
pub struct ListRoomsQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoomListResponse {
    pub rooms: Vec<RoomResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

pub async fn list_rooms(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListRoomsQuery>,
) -> Result<Json<ApiResponse<RoomListResponse>>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let rooms = state
        .room_service()
        .list_all_rooms(query.search.as_deref(), page_size, offset)
        .await?;

    let total = state.room_service().count_all_rooms().await?;

    Ok(Json(ApiResponse::success(RoomListResponse {
        rooms,
        total,
        page,
        page_size,
    })))
}

pub async fn get_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<ApiResponse<RoomResponse>>> {
    let room = state
        .room_service()
        .get_room_detail(room_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(room)))
}

pub async fn delete_room(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(room_id): Path<Uuid>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
) -> Result<StatusCode> {
    let room = state
        .room_service()
        .get_room_by_id(room_id)
        .await?
        .ok_or(AppError::NotFound)?;

    state.room_service().force_delete_room(room.id).await?;

    // 记录管理员操作审计日志
    let ip = addr.ip();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(admin_id, "room_delete", "room", room_id, ip)
            .await;
    });

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ListRoomMessagesQuery {
    pub limit: Option<i64>,
    pub before: Option<Uuid>,
}

pub async fn get_room_messages(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    Query(query): Query<ListRoomMessagesQuery>,
) -> Result<Json<ApiResponse<Vec<MessageResponse>>>> {
    let limit = query.limit.unwrap_or(50);

    let messages = state
        .message_service()
        .get_room_messages(room_id, limit, query.before)
        .await?;

    Ok(Json(ApiResponse::success(messages)))
}

// ==================== 消息审核接口 ====================

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
    pub room_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<ApiResponse<MessageListResponse>>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(50);
    let limit = page_size;
    let offset = (page - 1) * page_size;

    let (messages, total) = state
        .message_service()
        .list_all_messages(query.search.as_deref(), query.room_id, limit, offset)
        .await?;

    Ok(Json(ApiResponse::success(MessageListResponse {
        messages,
        total,
        page,
        page_size,
    })))
}

pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(message_id): Path<Uuid>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
) -> Result<StatusCode> {
    let message = state
        .message_service()
        .get_message_by_id(message_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let room_id = message.room_id;
    let sender_id = message.sender_id;

    // 获取发送者信息（用于通知）
    let sender = state.user_service().get_user_by_id(sender_id).await?;
    let sender_username = sender
        .map(|u| u.username)
        .unwrap_or_else(|| "未知用户".to_string());

    state
        .message_service()
        .admin_delete_message(message.id)
        .await?;

    // 向房间广播系统通知
    let ws_manager = Arc::clone(&state.ws_manager);
    let system_message = format!("系统管理员撤回了用户 {} 的消息", sender_username);
    let broadcast_message = serde_json::to_string(&WebSocketMessage::SystemMessage {
        content: system_message,
    })
    .unwrap_or_default();
    ws_manager
        .broadcast_to_room_all(room_id, broadcast_message)
        .await;

    // 记录管理员操作审计日志
    let ip = addr.ip();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(admin_id, "message_delete", "message", message_id, ip)
            .await;
    });

    Ok(StatusCode::NO_CONTENT)
}

// ==================== 房间成员管理接口 ====================

#[derive(Debug, Deserialize)]
pub struct SetMemberRoleRequest {
    pub role: String,
}

/// 管理员踢出房间成员
pub async fn kick_room_member(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path((room_id, user_id)): Path<(Uuid, Uuid)>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
    Extension(CurrentUserRole(admin_role)): Extension<CurrentUserRole>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    // 获取被踢出用户信息（用于通知）
    let target_user = state.user_service().get_user_by_id(user_id).await?;
    let target_username = target_user
        .map(|u| u.username)
        .unwrap_or_else(|| "未知用户".to_string());

    state
        .room_service()
        .admin_kick_member(room_id, user_id, &admin_role)
        .await?;

    // 向房间广播系统通知
    let ws_manager = Arc::clone(&state.ws_manager);
    let system_message = format!("系统管理员已将用户 {} 移出房间", target_username);
    let broadcast_message = serde_json::to_string(&WebSocketMessage::SystemMessage {
        content: system_message,
    })
    .unwrap_or_default();
    ws_manager
        .broadcast_to_room_all(room_id, broadcast_message)
        .await;

    // 记录审计日志
    let ip = addr.ip();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(admin_id, "room_member_kick", "room_member", room_id, ip)
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("成员已被踢出")))
}

/// 管理员设置房间成员角色
pub async fn set_room_member_role(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path((room_id, user_id)): Path<(Uuid, Uuid)>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
    Extension(CurrentUserRole(admin_role)): Extension<CurrentUserRole>,
    Json(request): Json<SetMemberRoleRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let new_role = match request.role.as_str() {
        "owner" => MemberRole::Owner,
        "admin" => MemberRole::Admin,
        "member" => MemberRole::Member,
        _ => return Err(AppError::Validation("无效的角色".to_string())),
    };

    // 获取目标用户信息（用于通知）
    let target_user = state.user_service().get_user_by_id(user_id).await?;
    let target_username = target_user
        .map(|u| u.username)
        .unwrap_or_else(|| "未知用户".to_string());

    state
        .room_service()
        .admin_set_member_role(room_id, user_id, new_role.clone(), &admin_role)
        .await?;

    // 向房间广播系统通知
    let ws_manager = Arc::clone(&state.ws_manager);
    let role_text = match new_role {
        MemberRole::Owner => "房主",
        MemberRole::Admin => "管理员",
        MemberRole::Member => "普通成员",
    };
    let system_message = format!("系统管理员已将用户 {} 设置为{}", target_username, role_text);
    let broadcast_message = serde_json::to_string(&WebSocketMessage::SystemMessage {
        content: system_message,
    })
    .unwrap_or_default();
    ws_manager
        .broadcast_to_room_all(room_id, broadcast_message)
        .await;

    // 记录审计日志
    let ip = addr.ip();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_admin_action(
                admin_id,
                "room_member_role_change",
                "room_member",
                room_id,
                ip,
            )
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("成员角色已更新")))
}

// ==================== 系统统计接口 ====================

#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_users: i64,
    pub total_rooms: i64,
    pub total_messages: i64,
    pub online_users: usize,
    pub active_connections: usize,
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<SystemStats>>> {
    let total_users = state.user_service().count_users().await?;
    let total_rooms = state.room_service().count_all_rooms().await?;
    let total_messages = state.message_service().count_all_messages().await?;
    let online_users = state.ws_manager().get_online_user_count();
    let active_connections = state.ws_manager().get_connection_count();

    Ok(Json(ApiResponse::success(SystemStats {
        total_users,
        total_rooms,
        total_messages,
        online_users,
        active_connections,
    })))
}

#[derive(Debug, Serialize)]
pub struct ActivityStats {
    pub daily_active_users: i64,
    pub weekly_active_users: i64,
    pub monthly_active_users: i64,
    pub daily_messages: i64,
    pub weekly_messages: i64,
    pub monthly_messages: i64,
}

pub async fn get_activity_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<ActivityStats>>> {
    let stats = state.message_service().get_activity_stats().await?;

    Ok(Json(ApiResponse::success(ActivityStats {
        daily_active_users: stats.daily_active_users,
        weekly_active_users: stats.weekly_active_users,
        monthly_active_users: stats.monthly_active_users,
        daily_messages: stats.daily_messages,
        weekly_messages: stats.weekly_messages,
        monthly_messages: stats.monthly_messages,
    })))
}

// ==================== 性能指标接口 ====================

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub total_messages: u64,
    pub total_connections: u64,
    pub current_online_users: u64,
    pub active_rooms: u64,
    pub timestamp: String,
}

pub async fn get_performance_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<PerformanceMetrics>>> {
    let snapshot = state.metrics_collector().get_snapshot();

    Ok(Json(ApiResponse::success(PerformanceMetrics {
        total_messages: snapshot.total_messages,
        total_connections: snapshot.total_connections,
        current_online_users: snapshot.current_online_users,
        active_rooms: snapshot.active_rooms,
        timestamp: snapshot.timestamp.to_rfc3339(),
    })))
}

// ==================== 系统监控接口 ====================

use crate::services::monitor_service::MonitorData;

/// 获取系统监控数据
pub async fn get_monitor_data(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<MonitorData>>> {
    let monitor_service = state.monitor_service();
    let data = monitor_service.get_monitor_data().await?;

    Ok(Json(ApiResponse::success(data)))
}

// ==================== Redis 与分布式管理接口 ====================

#[derive(Debug, Serialize)]
pub struct RedisNodeInfo {
    pub id: String,
    pub address: String,
    pub connected: bool,
    pub latency_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RedisStatusResponse {
    pub enabled: bool,
    pub connected: bool,
    pub pool_size: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub cluster_mode: bool,
    pub nodes: Vec<RedisNodeInfo>,
}

pub async fn get_redis_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<RedisStatusResponse>>> {
    let response = if let Some(ref redis_mgr) = state.redis_manager {
        // 测量连接延迟
        let start = std::time::Instant::now();
        let connected = redis_mgr.is_connected().await;
        let latency_ms = if connected {
            Some(start.elapsed().as_secs_f64() * 1000.0)
        } else {
            None
        };

        let node_id = redis_mgr.node_id().to_string();

        // 从配置获取地址
        let config = state.config.read().await;
        let address = config.redis.url.clone();
        drop(config);

        RedisStatusResponse {
            enabled: true,
            connected,
            pool_size: 1, // MultiplexedConnection 是单连接
            active_connections: if connected { 1 } else { 0 },
            idle_connections: 0,
            cluster_mode: false,
            nodes: vec![RedisNodeInfo {
                id: node_id,
                address,
                connected,
                latency_ms,
            }],
        }
    } else {
        RedisStatusResponse {
            enabled: false,
            connected: false,
            pool_size: 0,
            active_connections: 0,
            idle_connections: 0,
            cluster_mode: false,
            nodes: vec![],
        }
    };

    Ok(Json(ApiResponse::success(response)))
}

// ==================== 增强统计接口 ====================

use crate::services::user_service::{UserGrowthStats, UserBehaviorStats, FriendStats};
use crate::services::room_service::{RoomActivity, RoomStats};
use crate::services::message_service::{MessageTypeStats, MessageHourlyDistribution};
use crate::services::audit_service::SecurityStats;

/// 获取用户增长统计
pub async fn get_user_growth_stats(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DaysQuery>,
) -> Result<Json<ApiResponse<UserGrowthStats>>> {
    let days = query.days.unwrap_or(30);
    let stats = state.user_service().get_user_growth_stats(days).await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// 获取用户行为统计
pub async fn get_user_behavior_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<UserBehaviorStats>>> {
    let stats = state.user_service().get_user_behavior_stats().await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// 获取好友关系统计
pub async fn get_friend_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<FriendStats>>> {
    let stats = state.user_service().get_friend_stats().await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// 获取房间活跃度排行
pub async fn get_room_activity_ranking(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<ApiResponse<Vec<RoomActivity>>>> {
    let limit = query.limit.unwrap_or(10);
    let stats = state.room_service().get_room_activity_ranking(limit).await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// 获取房间统计概览
pub async fn get_room_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<RoomStats>>> {
    let stats = state.room_service().get_room_stats().await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// 获取消息类型分布
pub async fn get_message_type_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<MessageTypeStats>>> {
    let stats = state.message_service().get_message_type_stats().await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// 获取消息时间分布
pub async fn get_message_hourly_distribution(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<MessageHourlyDistribution>>>> {
    let stats = state.message_service().get_message_hourly_distribution().await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// 获取安全告警统计
pub async fn get_security_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<SecurityStats>>> {
    let stats = state.audit_service().get_security_stats().await?;
    Ok(Json(ApiResponse::success(stats)))
}

#[derive(Debug, Deserialize)]
pub struct DaysQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct RedisStatsResponse {
    pub pubsub_channels: i64,
    pub pubsub_patterns: i64,
    pub stream_messages: i64,
    pub stream_consumers: i64,
    pub memory_used: i64,
    pub memory_peak: i64,
    pub total_commands_processed: i64,
    pub ops_per_second: i64,
    pub hit_rate: f64,
    pub uptime_seconds: i64,
}

pub async fn get_redis_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<RedisStatsResponse>>> {
    let response = if let Some(ref redis_mgr) = state.redis_manager {
        // 尝试获取 Redis 统计信息
        if let Some(mut conn) = redis_mgr.get_connection().await {
            match redis::cmd("INFO").query_async::<_, String>(&mut conn).await {
                Ok(info) => {
                    // 解析 INFO 命令输出
                    let mut stats = RedisStatsResponse {
                        pubsub_channels: 0,
                        pubsub_patterns: 0,
                        stream_messages: 0,
                        stream_consumers: 0,
                        memory_used: 0,
                        memory_peak: 0,
                        total_commands_processed: 0,
                        ops_per_second: 0,
                        hit_rate: 0.0,
                        uptime_seconds: 0,
                    };

                    for line in info.lines() {
                        if let Some((key, value)) = line.split_once(':') {
                            match key {
                                "used_memory" => stats.memory_used = value.parse().unwrap_or(0),
                                "used_memory_peak" => {
                                    stats.memory_peak = value.parse().unwrap_or(0)
                                }
                                "total_commands_processed" => {
                                    stats.total_commands_processed = value.parse().unwrap_or(0)
                                }
                                "instantaneous_ops_per_sec" => {
                                    stats.ops_per_second = value.parse().unwrap_or(0)
                                }
                                "keyspace_hits" | "keyspace_misses" => {
                                    // 简化处理，实际需要计算命中率
                                }
                                "uptime_in_seconds" => {
                                    stats.uptime_seconds = value.parse().unwrap_or(0)
                                }
                                _ => {}
                            }
                        }
                    }
                    stats
                }
                Err(_) => RedisStatsResponse {
                    pubsub_channels: 0,
                    pubsub_patterns: 0,
                    stream_messages: 0,
                    stream_consumers: 0,
                    memory_used: 0,
                    memory_peak: 0,
                    total_commands_processed: 0,
                    ops_per_second: 0,
                    hit_rate: 0.0,
                    uptime_seconds: 0,
                },
            }
        } else {
            RedisStatsResponse {
                pubsub_channels: 0,
                pubsub_patterns: 0,
                stream_messages: 0,
                stream_consumers: 0,
                memory_used: 0,
                memory_peak: 0,
                total_commands_processed: 0,
                ops_per_second: 0,
                hit_rate: 0.0,
                uptime_seconds: 0,
            }
        }
    } else {
        RedisStatsResponse {
            pubsub_channels: 0,
            pubsub_patterns: 0,
            stream_messages: 0,
            stream_consumers: 0,
            memory_used: 0,
            memory_peak: 0,
            total_commands_processed: 0,
            ops_per_second: 0,
            hit_rate: 0.0,
            uptime_seconds: 0,
        }
    };

    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, Serialize)]
pub struct RedisRefreshResponse {
    pub refreshed: bool,
    pub message: String,
}

pub async fn refresh_redis(
    State(state): State<Arc<AppState>>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
) -> Result<Json<ApiResponse<RedisRefreshResponse>>> {
    // 检查权限
    if current_role != UserRole::SuperAdmin {
        return Err(AppError::Forbidden);
    }

    let result = if let Some(ref redis_mgr) = state.redis_manager {
        match redis_mgr.reconnect().await {
            Ok(_) => RedisRefreshResponse {
                refreshed: true,
                message: "Redis connections refreshed successfully".to_string(),
            },
            Err(e) => RedisRefreshResponse {
                refreshed: false,
                message: format!("Failed to refresh Redis connections: {}", e),
            },
        }
    } else {
        RedisRefreshResponse {
            refreshed: false,
            message: "Redis is not enabled".to_string(),
        }
    };

    Ok(Json(ApiResponse::success(result)))
}

#[derive(Debug, Serialize)]
pub struct ConfigSyncResponse {
    pub synced: bool,
    pub nodes_count: i32,
    pub synced_nodes: i32,
    pub failed_nodes: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TriggerConfigSyncRequest {
    pub config_keys: Option<Vec<String>>,
}

pub async fn trigger_config_sync(
    State(state): State<Arc<AppState>>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
    Json(_request): Json<TriggerConfigSyncRequest>,
) -> Result<Json<ApiResponse<ConfigSyncResponse>>> {
    // 检查权限
    if current_role != UserRole::SuperAdmin {
        return Err(AppError::Forbidden);
    }

    // 检查 Redis 是否启用
    if state.redis_manager.is_none() {
        return Ok(Json(ApiResponse::success(ConfigSyncResponse {
            synced: false,
            nodes_count: 0,
            synced_nodes: 0,
            failed_nodes: 0,
            message: "Redis is not enabled, config sync unavailable".to_string(),
        })));
    }

    // 通过 ConfigSyncManager 发布同步消息
    let sync_manager = state.config_manager().sync_manager();
    let (nodes_count, synced_nodes, message) = if let Some(ref mgr) = sync_manager {
        let msg = ConfigSyncMessage::reloaded(mgr.node_id().to_string());
        if let Err(e) = mgr.publish_change(msg).await {
            return Ok(Json(ApiResponse::success(ConfigSyncResponse {
                synced: false,
                nodes_count: 0,
                synced_nodes: 0,
                failed_nodes: 1,
                message: format!("Failed to publish sync message: {}", e),
            })));
        }
        let subs = mgr.subscriber_count().await;
        (subs, subs, "Configuration synced to all nodes".to_string())
    } else {
        (1, 1, "Configuration synced locally (no Redis)".to_string())
    };

    Ok(Json(ApiResponse::success(ConfigSyncResponse {
        synced: true,
        nodes_count,
        synced_nodes,
        failed_nodes: 0,
        message,
    })))
}

#[derive(Debug, Serialize)]
pub struct ConfigSyncStatusResponse {
    pub sync_enabled: bool,
    pub last_sync_at: Option<String>,
    pub nodes_total: i32,
    pub nodes_synced: i32,
    pub pending_changes: i32,
    pub sync_latency_ms: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct RespondPendingActionRequest {
    pub action: PendingActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

pub async fn respond_pending_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(CurrentUserId(admin_id)): Extension<CurrentUserId>,
    Json(request): Json<RespondPendingActionRequest>,
) -> Result<Json<ApiResponse<()>>> {
    state
        .notification_service()
        .process_pending_action(admin_id, id, request.action, request.comment)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

pub async fn get_config_sync_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<ConfigSyncStatusResponse>>> {
    let sync_manager = state.config_manager().sync_manager();

    let response = if let Some(ref mgr) = sync_manager {
        let start = std::time::Instant::now();
        let connected = if let Some(ref redis_mgr) = state.redis_manager {
            redis_mgr.is_connected().await
        } else {
            false
        };
        let sync_latency_ms = if connected {
            Some(start.elapsed().as_secs_f64() * 1000.0)
        } else {
            None
        };

        let last_sync_at = mgr.last_sync_at().await
            .map(|t| t.to_rfc3339());

        let subs = if connected {
            mgr.subscriber_count().await
        } else {
            0
        };

        ConfigSyncStatusResponse {
            sync_enabled: connected,
            last_sync_at,
            nodes_total: subs,
            nodes_synced: subs,
            pending_changes: mgr.pending_changes(),
            sync_latency_ms,
        }
    } else {
        ConfigSyncStatusResponse {
            sync_enabled: false,
            last_sync_at: None,
            nodes_total: 0,
            nodes_synced: 0,
            pending_changes: 0,
            sync_latency_ms: None,
        }
    };

    Ok(Json(ApiResponse::success(response)))
}
