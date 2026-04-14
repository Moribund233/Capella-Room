use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::response::ApiResponse,
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
    connect_info: Option<axum::extract::ConnectInfo<SocketAddr>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserResponse>>> {
    // 获取客户端IP（可选）
    let ip = connect_info
        .map(|ci| ci.0.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));

    // 验证请求
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 检查邮箱是否已存在
    let email_exists = state.user_service().email_exists(&request.email).await?;
    if email_exists {
        // 记录注册失败审计日志
        let audit_service = Arc::clone(&state.audit_service);
        tokio::spawn(async move {
            let _ = audit_service.log_user_register(uuid::Uuid::nil(), ip).await;
        });
        return Err(AppError::Conflict("邮箱已被注册".to_string()));
    }

    // 检查用户名是否已存在
    let username_exists = state
        .user_service()
        .username_exists(&request.username)
        .await?;
    if username_exists {
        // 记录注册失败审计日志
        let audit_service = Arc::clone(&state.audit_service);
        tokio::spawn(async move {
            let _ = audit_service.log_user_register(uuid::Uuid::nil(), ip).await;
        });
        return Err(AppError::Conflict("用户名已被使用".to_string()));
    }

    // 密码哈希处理
    let password_hash = state.auth_service().hash_password(&request.password)?;

    // 创建用户记录
    let user = state
        .user_service()
        .create_user(&request.username, &request.email, &password_hash)
        .await?;

    // 记录注册成功审计日志
    let user_id = user.id;
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service.log_user_register(user_id, ip).await;
    });

    // 返回用户信息
    Ok(Json(ApiResponse::success(user.to_response())))
}

/// 登录响应数据
#[derive(Debug, Serialize)]
pub struct LoginData {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub user: UserResponse,
}

/// 用户登录
pub async fn login(
    State(state): State<Arc<AppState>>,
    connect_info: Option<axum::extract::ConnectInfo<SocketAddr>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginData>>> {
    // 获取客户端IP（可选）
    let ip = connect_info
        .map(|ci| ci.0.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));

    // 验证请求
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 查找用户
    let user = state
        .user_service()
        .get_user_by_email(&request.email)
        .await?;

    let user = match user {
        Some(u) => u,
        None => {
            // 记录登录失败审计日志
            let email = request.email.clone();
            let audit_service = Arc::clone(&state.audit_service);
            tokio::spawn(async move {
                let _ = audit_service
                    .log_login_failure(&email, ip, "用户不存在")
                    .await;
            });
            return Err(AppError::Auth("邮箱或密码错误".to_string()));
        }
    };

    // 检查用户是否被禁用
    if user.status.is_disabled() {
        // 记录登录失败审计日志
        let email = request.email.clone();
        let audit_service = Arc::clone(&state.audit_service);
        tokio::spawn(async move {
            let _ = audit_service
                .log_login_failure(&email, ip, "账号已被禁用")
                .await;
        });
        return Err(AppError::Auth("账号已被禁用，请联系管理员".to_string()));
    }

    // 验证密码
    let password_valid = state
        .auth_service()
        .verify_password(&request.password, &user.password_hash)?;

    if !password_valid {
        // 记录登录失败审计日志
        let email = request.email.clone();
        let audit_service = Arc::clone(&state.audit_service);
        tokio::spawn(async move {
            let _ = audit_service
                .log_login_failure(&email, ip, "密码错误")
                .await;
        });
        return Err(AppError::Auth("邮箱或密码错误".to_string()));
    }

    // 更新用户状态为在线
    let _ = state
        .user_service()
        .update_user_status(user.id, crate::models::user::UserStatus::Online)
        .await;

    // 生成 JWT Token 对
    let token_pair =
        state
            .auth_service()
            .generate_token_pair(user.id, &user.username, user.role.clone())?;

    // 记录登录成功审计日志
    let user_id = user.id;
    let role = user.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_user_login(user_id, role, ip, None, true)
            .await;
    });

    // 获取更新后的用户信息（包含最新状态）
    let updated_user = state
        .user_service()
        .get_user_by_id(user.id)
        .await?
        .unwrap_or(user);

    // 返回 Token 和用户信息
    Ok(Json(ApiResponse::success(LoginData {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
        token_type: "Bearer".to_string(),
        user: updated_user.to_response(),
    })))
}

/// 刷新 Token 响应数据
#[derive(Debug, Serialize)]
pub struct RefreshTokenData {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

/// 刷新 Token
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<RefreshTokenData>>> {
    // 验证刷新令牌
    let claims = state
        .auth_service()
        .verify_refresh_token(&request.refresh_token)?;

    // 提取用户 ID
    let user_id = state.auth_service().extract_user_id(&claims)?;

    // 验证用户是否存在
    let user = state
        .user_service()
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::Auth("用户不存在".to_string()))?;

    // 生成新的 Token 对
    let token_pair =
        state
            .auth_service()
            .generate_token_pair(user_id, &user.username, user.role)?;

    Ok(Json(ApiResponse::success(RefreshTokenData {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
        token_type: "Bearer".to_string(),
    })))
}
