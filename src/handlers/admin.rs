use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    Extension, Json,
};

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::SystemConfigItem,
    error::{AppError, Result},
    middleware::admin::{CurrentUserId, CurrentUserRole},
    models::{
        message::MessageResponse,
        response::ApiResponse,
        room::RoomResponse,
        user::{UserResponse, UserRole},
    },
    state::AppState,
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

    state
        .message_service()
        .admin_delete_message(message.id)
        .await?;

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
