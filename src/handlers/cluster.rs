use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    error::Result,
    models::response::ApiResponse,
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct ClusterInfoResponse {
    pub cluster_id: String,
    pub cluster_name: String,
    pub node_id: String,
    pub version: String,
    pub started_at: String,
}

pub async fn get_cluster_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<ClusterInfoResponse>>> {
    let config = state.config_manager().get_config().await;
    let now = chrono::Utc::now().to_rfc3339();

    let node_id = state
        .redis_manager
        .as_ref()
        .map(|r| r.node_id().to_string())
        .unwrap_or_else(|| format!("node-{}", uuid::Uuid::new_v4()));

    let resp = ClusterInfoResponse {
        cluster_id: config.cluster.id,
        cluster_name: config.cluster.name,
        node_id,
        version: config.system.version,
        started_at: now,
    };

    Ok(Json(ApiResponse::success(resp)))
}
