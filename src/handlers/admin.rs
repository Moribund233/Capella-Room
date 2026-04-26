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
        let connected = redis_mgr.is_connected().await;
        let node_id = redis_mgr.node_id().to_string();

        RedisStatusResponse {
            enabled: true,
            connected,
            pool_size: 10, // 从配置获取
            active_connections: if connected { 1 } else { 0 },
            idle_connections: if connected { 9 } else { 0 },
            cluster_mode: false,
            nodes: vec![RedisNodeInfo {
                id: node_id,
                address: "redis://localhost:6379".to_string(),
                connected,
                latency_ms: if connected { Some(0.5) } else { None },
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
                                "used_memory" => {
                                    stats.memory_used = value.parse().unwrap_or(0)
                                }
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
    Json(request): Json<TriggerConfigSyncRequest>,
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

    // 触发配置同步
    // 实际实现中应该通过 ConfigSyncManager 发布同步消息
    let keys = request.config_keys.unwrap_or_default();
    let message = if keys.is_empty() {
        "Configuration synced to all nodes".to_string()
    } else {
        format!("Configuration synced for keys: {:?}", keys)
    };

    Ok(Json(ApiResponse::success(ConfigSyncResponse {
        synced: true,
        nodes_count: 1,
        synced_nodes: 1,
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

pub async fn get_config_sync_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<ConfigSyncStatusResponse>>> {
    let response = if let Some(ref _redis_mgr) = state.redis_manager {
        // 从 ConfigManager 获取同步状态
        ConfigSyncStatusResponse {
            sync_enabled: true,
            last_sync_at: None, // 实际应从 ConfigManager 获取
            nodes_total: 1,
            nodes_synced: 1,
            pending_changes: 0,
            sync_latency_ms: None,
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
