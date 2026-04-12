use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{
    config::ClientConfig,
    error::Result,
    models::response::ApiResponse,
    state::AppState,
};

/// 获取客户端配置
/// 公开端点，供前端应用获取必要的服务端配置
pub async fn get_client_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<ClientConfig>>> {
    let config = state.config_manager().get_config().await;
    let client_config = ClientConfig::from_app_config(&config);

    Ok(Json(ApiResponse::success(client_config)))
}
