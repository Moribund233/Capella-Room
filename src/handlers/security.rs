use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::Result,
    middleware::admin::CurrentUserId,
    models::response::ApiResponse,
    models::security::{
        BatchIpListRequest, CreateIpListRequest, IpCheckRequest, IpListQuery, UpdateIpListRequest,
    },
    state::AppState,
};

/// 查询 IP 列表
/// GET /api/v1/admin/security/ip-list
pub async fn list_ip_entries(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IpListQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let (entries, total) = state
        .ip_security_service()
        .query_ip_list(query.clone())
        .await?;

    let response = serde_json::json!({
        "entries": entries,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0),
    });

    Ok(Json(ApiResponse::success(response)))
}

/// 添加 IP 到列表
/// POST /api/v1/admin/security/ip-list
pub async fn add_ip_entry(
    State(state): State<Arc<AppState>>,
    Extension(CurrentUserId(user_id)): Extension<CurrentUserId>,
    Json(request): Json<CreateIpListRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let entry = state
        .ip_security_service()
        .add_ip_to_list(request, user_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "entry": entry,
        "message": "IP entry added successfully"
    }))))
}

/// 批量添加 IP 到列表
/// POST /api/v1/admin/security/ip-list/batch
pub async fn batch_add_ip_entries(
    State(state): State<Arc<AppState>>,
    Extension(CurrentUserId(user_id)): Extension<CurrentUserId>,
    Json(request): Json<BatchIpListRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let result = state
        .ip_security_service()
        .batch_add_to_list(request, user_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "success_count": result.success_count,
        "failed_count": result.failed_count,
        "failed_ips": result.failed_ips,
        "message": format!("Added {} entries, {} failed", result.success_count, result.failed_count)
    }))))
}

/// 更新 IP 列表项
/// PUT /api/v1/admin/security/ip-list/:id
pub async fn update_ip_entry(
    State(state): State<Arc<AppState>>,
    Extension(CurrentUserId(user_id)): Extension<CurrentUserId>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateIpListRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let entry = state
        .ip_security_service()
        .update_ip_entry(id, request, user_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "entry": entry,
        "message": "IP entry updated successfully"
    }))))
}

/// 从列表中移除 IP
/// DELETE /api/v1/admin/security/ip-list/:id
pub async fn remove_ip_entry(
    State(state): State<Arc<AppState>>,
    Extension(CurrentUserId(user_id)): Extension<CurrentUserId>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    state
        .ip_security_service()
        .remove_from_list(id, user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 检查 IP 状态
/// POST /api/v1/admin/security/ip-check
pub async fn check_ip(
    State(state): State<Arc<AppState>>,
    Json(request): Json<IpCheckRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let result = state.ip_security_service().check_ip_status(request).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "ip_address": result.ip_address.to_string(),
        "allowed": result.allowed,
        "reason": result.reason,
        "list_type": result.list_type.map(|t| t.to_string()),
    }))))
}

/// 获取 IP 安全统计信息
/// GET /api/v1/admin/security/stats
pub async fn get_security_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let stats = state.ip_security_service().get_stats().await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "total_whitelist": stats.total_whitelist,
        "total_blacklist": stats.total_blacklist,
        "expired_entries": stats.expired_entries,
        "active_entries": stats.active_entries,
    }))))
}

/// 刷新 IP 列表缓存
/// POST /api/v1/admin/security/refresh-cache
pub async fn refresh_cache(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    state.ip_security_service().refresh_cache().await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "IP security cache refreshed successfully"
    }))))
}

/// 清理过期的 IP 条目
/// POST /api/v1/admin/security/cleanup-expired
pub async fn cleanup_expired(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let deleted = state
        .ip_security_service()
        .cleanup_expired_entries()
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "deleted_count": deleted,
        "message": format!("Cleaned up {} expired entries", deleted)
    }))))
}

/// 设置白名单模式
/// POST /api/v1/admin/security/whitelist-mode
#[derive(Debug, Deserialize)]
pub struct SetWhitelistModeRequest {
    pub enabled: bool,
}

pub async fn set_whitelist_mode(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SetWhitelistModeRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    state
        .ip_security_service()
        .set_whitelist_mode(request.enabled)
        .await;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "enabled": request.enabled,
        "message": if request.enabled {
            "Whitelist mode enabled"
        } else {
            "Whitelist mode disabled"
        }
    }))))
}

/// 获取白名单模式状态
/// GET /api/v1/admin/security/whitelist-mode
pub async fn get_whitelist_mode(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let enabled = state
        .ip_security_service()
        .is_whitelist_mode_enabled()
        .await;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "enabled": enabled,
    }))))
}
