use axum::{
    extract::{ConnectInfo, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::response::ApiResponse,
    models::user::{ChangePasswordRequest, UpdateUserRequest, UserResponse},
    services::auth_service::Claims,
    state::AppState,
};

/// 获取用户列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    /// 搜索关键词（用户名或邮箱）
    pub search: Option<String>,
    /// 每页数量（默认20，最大100）
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// 偏移量（默认0）
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

fn default_offset() -> i64 {
    0
}

/// 获取当前用户信息
pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    // 从 Claims 中提取用户 ID
    let user_id = state.auth_service().extract_user_id(&claims)?;

    // 查询用户信息
    let user = state.user_service().get_user_by_id(user_id).await?;

    match user {
        Some(user) => Ok(Json(ApiResponse::success(user.to_response()))),
        None => Err(AppError::NotFound),
    }
}

/// 更新用户信息
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    // 验证请求
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 从 Claims 中提取用户 ID
    let user_id = state.auth_service().extract_user_id(&claims)?;

    // 更新用户信息
    let updated_user = state
        .user_service()
        .update_user(
            user_id,
            request.username.as_deref(),
            request.avatar_url.as_deref(),
        )
        .await?;

    Ok(Json(ApiResponse::success(updated_user.to_response())))
}

/// 获取用户列表
/// 支持搜索和分页
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<ListUsersResponse>>> {
    // 限制每页数量
    let limit = query.limit.clamp(1, 100);
    let offset = query.offset.max(0);

    // 查询用户列表
    let (users, total) = if let Some(search) = query.search {
        // 搜索模式
        state
            .user_service()
            .search_users(&search, limit, offset)
            .await?
    } else {
        // 普通列表模式
        let users = state.user_service().list_users(limit, offset).await?;
        let total = state.user_service().count_users().await?;
        (users, total)
    };

    // 转换为响应格式
    let user_responses: Vec<UserResponse> = users.into_iter().map(|u| u.to_response()).collect();

    Ok(Json(ApiResponse::success(ListUsersResponse {
        users: user_responses,
        total,
        limit,
        offset,
    })))
}

/// 用户列表响应
#[derive(Debug, serde::Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// 获取指定用户信息
pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<Uuid>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    let user = state.user_service().get_user_by_id(user_id).await?;

    match user {
        Some(user) => Ok(Json(ApiResponse::success(user.to_response()))),
        None => Err(AppError::NotFound),
    }
}

/// 修改密码
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let user = state
        .user_service()
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let old_password_valid = state
        .auth_service()
        .verify_password(&request.old_password, &user.password_hash)?;

    if !old_password_valid {
        return Err(AppError::Auth("原密码错误".to_string()));
    }

    if request.old_password == request.new_password {
        return Err(AppError::Validation("新密码不能与原密码相同".to_string()));
    }

    let new_password_hash = state.auth_service().hash_password(&request.new_password)?;

    state
        .user_service()
        .update_password(user_id, &new_password_hash)
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service.log_password_change(user_id, role, ip).await;
    });

    Ok(Json(ApiResponse::success_with_message("密码修改成功")))
}

/// 用户登出
pub async fn logout(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let ip = addr.ip();
    let role = claims.role.clone();

    // 更新用户状态为离线
    let _ = state
        .user_service()
        .update_user_status(user_id, crate::models::user::UserStatus::Offline)
        .await;

    // 记录审计日志
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service.log_user_logout(user_id, role, ip).await;
    });

    Ok(Json(ApiResponse::success_with_message("登出成功")))
}
