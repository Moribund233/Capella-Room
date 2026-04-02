use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::user::{LoginRequest, RegisterRequest, UserResponse},
    state::AppState,
};

/// 刷新Token请求
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 用户注册
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<UserResponse>> {
    // 验证请求
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // 检查邮箱是否已存在
    let email_exists = state.user_service().email_exists(&request.email).await?;
    if email_exists {
        return Err(AppError::Conflict("邮箱已被注册".to_string()));
    }

    // 检查用户名是否已存在
    let username_exists = state
        .user_service()
        .username_exists(&request.username)
        .await?;
    if username_exists {
        return Err(AppError::Conflict("用户名已被使用".to_string()));
    }

    // 密码哈希处理
    let password_hash = state.auth_service().hash_password(&request.password)?;

    // 创建用户记录
    let user = state
        .user_service()
        .create_user(&request.username, &request.email, &password_hash)
        .await?;

    // 返回用户信息
    Ok(Json(user.to_response()))
}

/// 用户登录
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>> {
    // 验证请求
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // 查找用户
    let user = state
        .user_service()
        .get_user_by_email(&request.email)
        .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(AppError::Auth("邮箱或密码错误".to_string())),
    };

    // 验证密码
    let password_valid = state
        .auth_service()
        .verify_password(&request.password, &user.password_hash)?;

    if !password_valid {
        return Err(AppError::Auth("邮箱或密码错误".to_string()));
    }

    // 生成JWT Token对
    let token_pair = state.auth_service().generate_token_pair(user.id)?;

    // 返回Token和用户信息
    Ok(Json(json!({
        "success": true,
        "data": {
            "access_token": token_pair.access_token,
            "refresh_token": token_pair.refresh_token,
            "expires_in": token_pair.expires_in,
            "token_type": "Bearer",
            "user": user.to_response(),
        }
    })))
}

/// 刷新Token
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<serde_json::Value>> {
    // 验证刷新令牌
    let claims = state
        .auth_service()
        .verify_refresh_token(&request.refresh_token)?;

    // 提取用户ID
    let user_id = state.auth_service().extract_user_id(&claims)?;

    // 验证用户是否存在
    let user = state.user_service().get_user_by_id(user_id).await?;
    if user.is_none() {
        return Err(AppError::Auth("用户不存在".to_string()));
    }

    // 生成新的Token对
    let token_pair = state.auth_service().generate_token_pair(user_id)?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "access_token": token_pair.access_token,
            "refresh_token": token_pair.refresh_token,
            "expires_in": token_pair.expires_in,
            "token_type": "Bearer",
        }
    })))
}
