use axum::{extract::Path, Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::response::ApiResponse;
use crate::models::user_settings::{
    UpdateRoomSettingsRequest, UpdateUserSettingsRequest, UserRoomSettingsResponse,
    UserSettingsResponse,
};
use crate::services::auth_service::Claims;
use crate::state::AppState;

/// GET /api/v1/users/me/settings
/// 获取当前用户的完整设置
pub async fn get_user_settings(
    state: axum::extract::State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<UserSettingsResponse>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let settings = state
        .user_settings_service()
        .get_user_settings(user_id)
        .await?;

    Ok(Json(ApiResponse::success(settings)))
}

/// PATCH /api/v1/users/me/settings
/// 部分更新当前用户的设置（按分组覆盖）
pub async fn update_user_settings(
    state: axum::extract::State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<UpdateUserSettingsRequest>,
) -> Result<Json<ApiResponse<UserSettingsResponse>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let settings = state
        .user_settings_service()
        .update_user_settings(user_id, request)
        .await?;

    Ok(Json(ApiResponse::success(settings)))
}

/// GET /api/v1/users/me/rooms/settings
/// 获取当前用户所有房间的设置列表
pub async fn list_room_settings(
    state: axum::extract::State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<UserRoomSettingsResponse>>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let settings = state
        .user_settings_service()
        .list_room_settings(user_id)
        .await?;

    Ok(Json(ApiResponse::success(settings)))
}

/// GET /api/v1/users/me/rooms/:room_id/settings
/// 获取当前用户在指定房间的设置
pub async fn get_room_settings(
    state: axum::extract::State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserRoomSettingsResponse>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let settings = state
        .user_settings_service()
        .get_room_settings(user_id, room_id)
        .await?;

    Ok(Json(ApiResponse::success(settings)))
}

/// PATCH /api/v1/users/me/rooms/:room_id/settings
/// 更新当前用户在指定房间的设置
pub async fn update_room_settings(
    state: axum::extract::State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<Uuid>,
    Json(request): Json<UpdateRoomSettingsRequest>,
) -> Result<Json<ApiResponse<UserRoomSettingsResponse>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    // 验证请求数据
    if let Err(msg) = request.validate() {
        return Err(AppError::Validation(msg));
    }

    let settings = state
        .user_settings_service()
        .update_room_settings(user_id, room_id, request)
        .await?;

    Ok(Json(ApiResponse::success(settings)))
}

/// DELETE /api/v1/users/me/settings
/// 删除当前用户的设置（重置为默认）
pub async fn reset_user_settings(
    state: axum::extract::State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .user_settings_service()
        .delete_user_settings(user_id)
        .await?;

    Ok(Json(ApiResponse::success_with_message("设置已重置为默认值")))
}

/// DELETE /api/v1/users/me/rooms/:room_id/settings
/// 删除当前用户在指定房间的设置（恢复默认）
pub async fn delete_room_settings(
    state: axum::extract::State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service()
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .user_settings_service()
        .delete_room_settings(user_id, room_id)
        .await?;

    Ok(Json(ApiResponse::success_with_message("房间设置已重置")))
}
