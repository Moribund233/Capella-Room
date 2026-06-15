use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::models::account_security::{CreateSessionRequest, LoginStatus, RecordLoginRequest};
use crate::models::response::ApiResponse;
use crate::models::user::UserResponse;
use crate::models::verification_code::VerificationPurpose;
use crate::state::AppState;

/// 发送验证码请求
#[derive(Debug, Deserialize)]
pub struct SendCodeRequest {
    pub email: String,
}

/// 注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub code: String,
    pub username: String,
    pub password: String,
}

/// 登录验证码请求
#[derive(Debug, Deserialize)]
pub struct LoginCodeRequest {
    pub email: String,
    pub code: String,
}

/// 密码重置请求
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub code: String,
    pub new_password: String,
}

/// 密码登录请求
#[derive(Debug, Deserialize)]
pub struct LoginWithPasswordRequest {
    pub email: String,
    pub password: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
}

/// Token 刷新请求
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 认证响应数据
#[derive(Debug, Serialize)]
pub struct AuthData {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub user: UserResponse,
}

// ─── 发送验证码 ───

/// 发送注册验证码
pub async fn register_send_code(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendCodeRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let email = request.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("邮箱不能为空".to_string()));
    }

    let email_exists = state.user_service().email_exists(&email).await?;
    if email_exists {
        return Err(AppError::Conflict("该邮箱已被注册".to_string()));
    }

    state
        .verification_code_service()
        .create_code(&email, VerificationPurpose::Register)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "验证码已发送",
        "code_length": 6
    }))))
}

/// 发送登录验证码
pub async fn login_send_code(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendCodeRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let email = request.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("邮箱不能为空".to_string()));
    }

    let user = state.user_service().get_user_by_email(&email).await?;
    if user.is_none() {
        return Err(AppError::Auth("该邮箱未注册".to_string()));
    }

    state
        .verification_code_service()
        .create_code(&email, VerificationPurpose::Login)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "验证码已发送",
        "code_length": 6
    }))))
}

/// 发送密码重置验证码
pub async fn reset_password_send_code(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendCodeRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let email = request.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("邮箱不能为空".to_string()));
    }

    let user = state.user_service().get_user_by_email(&email).await?;
    if user.is_none() {
        return Err(AppError::Auth("该邮箱未注册".to_string()));
    }

    state
        .verification_code_service()
        .create_code(&email, VerificationPurpose::ResetPassword)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "验证码已发送",
        "code_length": 6
    }))))
}

// ─── 验证码验证 + 操作 ───

/// 注册
pub async fn register(
    State(state): State<Arc<AppState>>,
    connect_info: Option<axum::extract::ConnectInfo<SocketAddr>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthData>>> {
    let email = request.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("邮箱不能为空".to_string()));
    }

    let code_valid = state
        .verification_code_service()
        .verify_code(&email, &request.code, VerificationPurpose::Register)
        .await?;
    if !code_valid {
        return Err(AppError::Auth("验证码无效或已过期".to_string()));
    }

    if state.user_service().email_exists(&email).await? {
        return Err(AppError::Conflict("该邮箱已被注册".to_string()));
    }
    if state.user_service().username_exists(&request.username).await? {
        return Err(AppError::Conflict("用户名已被使用".to_string()));
    }

    let password_hash = state.auth_service().hash_password(&request.password)?;
    let user = state.user_service().create_user(&request.username, &email, &password_hash).await?;
    let token_pair = state.auth_service().generate_token_pair(user.id, &user.username, user.role.clone())?;

    let ip = connect_info.map(|ci| ci.0.ip()).unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
    let audit_service = Arc::clone(&state.audit_service);
    let user_id = user.id;
    tokio::spawn(async move {
        let _ = audit_service.log_user_register(user_id, ip).await;
    });

    Ok(Json(ApiResponse::success(AuthData {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
        token_type: "Bearer".to_string(),
        user: user.to_response(),
    })))
}

/// 验证码登录
pub async fn login(
    State(state): State<Arc<AppState>>,
    connect_info: Option<axum::extract::ConnectInfo<SocketAddr>>,
    Json(request): Json<LoginCodeRequest>,
) -> Result<Json<ApiResponse<AuthData>>> {
    let email = request.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("邮箱不能为空".to_string()));
    }

    let code_valid = state
        .verification_code_service()
        .verify_code(&email, &request.code, VerificationPurpose::Login)
        .await?;
    if !code_valid {
        return Err(AppError::Auth("验证码无效或已过期".to_string()));
    }

    let user = state.user_service().get_user_by_email(&email).await?
        .ok_or_else(|| AppError::Auth("用户不存在".to_string()))?;

    let ip = connect_info.map(|ci| ci.0.ip()).unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
    issue_auth_tokens(&state, user, ip).await
}

/// 密码登录（兼容模式）
pub async fn login_with_password(
    State(state): State<Arc<AppState>>,
    connect_info: Option<axum::extract::ConnectInfo<SocketAddr>>,
    Json(request): Json<LoginWithPasswordRequest>,
) -> Result<Json<ApiResponse<AuthData>>> {
    let email = request.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("邮箱不能为空".to_string()));
    }

    let user = state.user_service().get_user_by_email(&email).await?
        .ok_or_else(|| AppError::Auth("邮箱或密码错误".to_string()))?;

    if user.is_account_disabled() {
        return Err(AppError::Auth("账号已被禁用，请联系管理员".to_string()));
    }

    let password_valid = state.auth_service().verify_password(&request.password, &user.password_hash)?;
    if !password_valid {
        return Err(AppError::Auth("邮箱或密码错误".to_string()));
    }

    let ip = connect_info.map(|ci| ci.0.ip()).unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
    issue_auth_tokens(&state, user, ip).await
}

/// 密码重置
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let email = request.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(AppError::Validation("邮箱不能为空".to_string()));
    }

    let code_valid = state
        .verification_code_service()
        .verify_code(&email, &request.code, VerificationPurpose::ResetPassword)
        .await?;
    if !code_valid {
        return Err(AppError::Auth("验证码无效或已过期".to_string()));
    }

    let user = state.user_service().get_user_by_email(&email).await?
        .ok_or_else(|| AppError::Auth("用户不存在".to_string()))?;

    let password_hash = state.auth_service().hash_password(&request.new_password)?;

    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&password_hash)
        .bind(user.id)
        .execute(state.db().pool())
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "密码已重置"
    }))))
}

/// Token 刷新
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let claims = state.auth_service().verify_refresh_token(&request.refresh_token)?;
    let user_id = state.auth_service().extract_user_id(&claims)?;

    let user = state.user_service().get_user_by_id(user_id).await?
        .ok_or_else(|| AppError::Auth("用户不存在".to_string()))?;

    let token_pair = state.auth_service().generate_token_pair(user_id, &user.username, user.role)?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "access_token": token_pair.access_token,
        "refresh_token": token_pair.refresh_token,
        "expires_in": token_pair.expires_in,
        "token_type": "Bearer"
    }))))
}

/// 登出
pub async fn logout(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    // TODO: invalidate refresh token in v2
    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "已登出"
    }))))
}

// ─── 内部辅助 ───

async fn issue_auth_tokens(
    state: &AppState,
    user: crate::models::user::User,
    ip: std::net::IpAddr,
) -> Result<Json<ApiResponse<AuthData>>> {
    let _ = state
        .user_service()
        .update_user_status(user.id, crate::models::user::UserStatus::Online)
        .await;

    let token_pair = state.auth_service().generate_token_pair(user.id, &user.username, user.role.clone())?;

    let session = state.account_security_service().create_session(
        user.id,
        CreateSessionRequest {
            session_token_hash: {
                let mut hasher = sha2::Sha256::new();
                hasher.update(&token_pair.access_token);
                format!("{:x}", hasher.finalize())
            },
            device_name: None,
            device_type: None,
            ip_address: ip,
            user_agent: None,
            location_info: None,
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        },
    ).await?;

    let settings = state.user_settings_service().get_user_settings(user.id).await?;
    if settings.privacy.single_device_login {
        let _ = state.account_security_service()
            .terminate_other_sessions(user.id, session.id).await;
    }

    let _ = state.account_security_service().record_login(
        user.id,
        RecordLoginRequest {
            ip_address: ip,
            device_info: None,
            location_info: None,
            login_status: LoginStatus::Success,
            failure_reason: None,
            is_suspicious: false,
            risk_level: crate::models::account_security::RiskLevel::Low,
        },
    ).await;

    Ok(Json(ApiResponse::success(AuthData {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
        token_type: "Bearer".to_string(),
        user: user.to_response(),
    })))
}
