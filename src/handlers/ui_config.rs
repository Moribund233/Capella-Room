//! 用户 UI 配置处理器
//!
//! 提供用户界面配置的 REST API 接口

use axum::{
    extract::{Extension, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::{
    error::AppError,
    models::{
        response::ApiResponse,
        ui_config::{SaveUIConfigRequest, UIConfigResponse},
    },
    services::{auth_service::Claims, ui_config_service::UIConfigService},
    state::AppState,
};

/// 获取用户 UI 配置
///
/// GET /api/v1/ui/config
pub async fn get_user_config(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let service = UIConfigService::new(state.db.pool().clone());
    let user_id = claims
        .sub
        .parse()
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    match service.get_user_config(user_id).await? {
        Some(config) => Ok(Json(ApiResponse::success(config))),
        None => Ok(Json(ApiResponse::<UIConfigResponse>::success(
            UIConfigResponse::default(),
        ))),
    }
}

/// 保存用户 UI 配置
///
/// POST /api/v1/ui/config
pub async fn save_user_config(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<SaveUIConfigRequest>,
) -> Result<impl IntoResponse, AppError> {
    let service = UIConfigService::new(state.db.pool().clone());
    let user_id = claims
        .sub
        .parse()
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    service.save_user_config(user_id, request).await?;

    Ok(Json(ApiResponse::<()>::success_with_message("配置已保存")))
}

/// 重置用户 UI 配置（删除云端配置）
///
/// DELETE /api/v1/ui/config
pub async fn reset_user_config(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let service = UIConfigService::new(state.db.pool().clone());
    let user_id = claims
        .sub
        .parse()
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    service.delete_user_config(user_id).await?;

    Ok(Json(ApiResponse::<()>::success_with_message("配置已重置")))
}
