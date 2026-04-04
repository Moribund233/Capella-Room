use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::SystemConfigItem,
    error::{AppError, Result},
    middleware::admin::CurrentUserRole,
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
    Path(user_id): Path<Uuid>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
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

    Ok(Json(ApiResponse::success(user.to_response())))
}

pub async fn set_user_status(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
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

    Ok(Json(ApiResponse::success(user.to_response())))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
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

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ListConfigsQuery {
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub value: String,
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
    Path(key): Path<String>,
    Extension(CurrentUserRole(current_role)): Extension<CurrentUserRole>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<ApiResponse<SystemConfigItem>>> {
    if !current_role.is_super_admin() {
        return Err(AppError::Forbidden);
    }

    let config = state
        .config_manager()
        .set_config(&key, &request.value)
        .await?;

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
    Path(room_id): Path<Uuid>,
) -> Result<StatusCode> {
    let room = state
        .room_service()
        .get_room_by_id(room_id)
        .await?
        .ok_or(AppError::NotFound)?;

    state.room_service().force_delete_room(room.id).await?;

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
    Path(message_id): Path<Uuid>,
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

// ==================== 日志查看接口 ====================

#[derive(Debug, Deserialize)]
pub struct ListLogsQuery {
    pub level: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub target: Option<String>,
    pub fields: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct LogListResponse {
    pub logs: Vec<LogEntry>,
    pub total: i64,
}

pub async fn list_logs(
    Query(_query): Query<ListLogsQuery>,
) -> Result<Json<ApiResponse<LogListResponse>>> {
    let logs = Vec::new();
    let total = 0;

    Ok(Json(ApiResponse::success(LogListResponse { logs, total })))
}

pub async fn download_logs() -> Result<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}
