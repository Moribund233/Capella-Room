use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{
        account_security::{
            CreateSessionRequest, DeviceInfo, LocationInfo, LoginHistoryEntry, LoginStatus,
            RecordLoginRequest, RiskLevel,
        },
        response::ApiResponse,
    },
    services::{account_security_service::AccountSecurityService, auth_service::Claims},
    state::AppState,
};

/// 分页查询参数
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

fn default_offset() -> i64 {
    0
}

/// 获取账号安全概览
/// GET /api/v1/users/me/security/overview
pub async fn get_security_overview(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    // 获取用户通知设置
    let settings = state
        .user_settings_service()
        .get_notification_settings(user_id)
        .await?;

    // 获取安全概览
    let overview = state
        .account_security_service()
        .get_security_overview(user_id, &settings)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "active_devices_count": overview.active_devices_count,
        "recent_logins": overview.recent_logins,
        "has_suspicious_activity": overview.has_suspicious_activity,
        "abnormal_login_alert": overview.abnormal_login_alert,
    }))))
}

/// 获取登录设备列表
/// GET /api/v1/users/me/devices
pub async fn list_devices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<DeviceInfo>>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let devices = state
        .account_security_service()
        .list_user_devices(user_id)
        .await?;

    Ok(Json(ApiResponse::success(devices)))
}

/// 终止指定设备会话（远程登出）
/// DELETE /api/v1/users/me/devices/:device_id
pub async fn terminate_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .account_security_service()
        .terminate_session(user_id, device_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "设备已成功登出"
    }))))
}

/// 禁用指定设备
/// POST /api/v1/users/me/devices/:device_id/block
/// 被禁用的设备无法使用旧 Token 登录
pub async fn block_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .account_security_service()
        .block_device(user_id, device_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "设备已禁用，该设备无法再使用旧 Token 登录"
    }))))
}

/// 启用被禁用的设备
/// POST /api/v1/users/me/devices/:device_id/unblock
/// 将设备从禁用状态恢复，但用户需要重新登录
pub async fn unblock_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .account_security_service()
        .unblock_device(user_id, device_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "设备已启用，需要重新登录"
    }))))
}

/// 终止所有其他设备会话
/// DELETE /api/v1/users/me/devices
pub async fn terminate_all_other_devices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    // 获取当前用户的所有活跃会话
    let devices = state
        .account_security_service()
        .list_user_devices(user_id)
        .await?;

    // 找到当前会话（is_current = true）
    let current_device = devices
        .iter()
        .find(|d| d.is_current)
        .ok_or_else(|| AppError::Auth("无法获取当前会话信息".to_string()))?;

    // 终止其他所有会话
    let terminated_count = state
        .account_security_service()
        .terminate_other_sessions(user_id, current_device.id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "其他设备已成功登出",
        "terminated_count": terminated_count
    }))))
}

/// 获取登录历史
/// GET /api/v1/users/me/login-history
pub async fn get_login_history(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    // 限制最大查询数量
    let limit = query.limit.min(100);

    let history = state
        .account_security_service()
        .get_login_history(user_id, limit, query.offset)
        .await?;

    let total = state
        .account_security_service()
        .get_login_history_count(user_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "data": history,
        "pagination": {
            "total": total,
            "limit": limit,
            "offset": query.offset,
        }
    }))))
}

/// 获取最近的可疑登录记录
/// GET /api/v1/users/me/login-history/suspicious
pub async fn get_suspicious_logins(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<LoginHistoryEntry>>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    // 获取最近7天的可疑登录
    let suspicious = state
        .account_security_service()
        .get_recent_suspicious_logins(user_id, 7)
        .await?;

    Ok(Json(ApiResponse::success(suspicious)))
}

/// 记录登录（内部使用，不对外暴露）
/// 这个方法用于在登录流程中记录登录历史
pub async fn record_login_internal(
    service: &AccountSecurityService,
    user_id: Uuid,
    ip: std::net::IpAddr,
    user_agent: Option<&str>,
    location_info: Option<LocationInfo>,
    login_status: LoginStatus,
    failure_reason: Option<String>,
) -> Result<()> {
    let device_name = AccountSecurityService::parse_device_name(user_agent);
    let device_type =
        crate::models::account_security::DeviceType::from_user_agent(user_agent.unwrap_or(""));

    let device_info = serde_json::json!({
        "device_name": device_name,
        "device_type": device_type.as_str(),
        "user_agent": user_agent.unwrap_or(""),
    });

    let location_json = location_info.map(|l| l.to_json_value());

    // 检测是否为异地登录
    let (is_suspicious, risk_level) = if login_status == LoginStatus::Success {
        service
            .detect_abnormal_login(user_id, ip, device_type.as_str())
            .await?
    } else {
        (false, RiskLevel::Low)
    };

    let request = RecordLoginRequest {
        ip_address: ip,
        device_info: Some(device_info),
        location_info: location_json,
        login_status,
        failure_reason,
        is_suspicious,
        risk_level,
    };

    service.record_login(user_id, request).await?;

    Ok(())
}

/// 创建设备会话（内部使用，不对外暴露）
/// 这个方法用于在登录成功后创建设备会话
pub async fn create_session_internal(
    service: &AccountSecurityService,
    user_id: Uuid,
    token_hash: &str,
    ip: std::net::IpAddr,
    user_agent: Option<&str>,
    location_info: Option<LocationInfo>,
    expires_hours: i64,
) -> Result<crate::models::account_security::UserSession> {
    let device_name = AccountSecurityService::parse_device_name(user_agent);
    let device_type =
        crate::models::account_security::DeviceType::from_user_agent(user_agent.unwrap_or(""));

    let request = CreateSessionRequest {
        session_token_hash: token_hash.to_string(),
        device_name: Some(device_name),
        device_type: Some(device_type.as_str().to_string()),
        ip_address: ip,
        user_agent: user_agent.map(|s| s.to_string()),
        location_info: location_info.map(|l| l.to_json_value()),
        expires_at: AccountSecurityService::calculate_session_expiry(expires_hours),
    };

    let session = service.create_session(user_id, request).await?;

    Ok(session)
}
