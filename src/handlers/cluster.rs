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

/// 返回集群标识信息，供客户端判断是否需要清空本地缓存。
/// 所有动态字段（node_id、started_at）在进程启动时确定，请求间不变。
pub async fn get_cluster_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<ClusterInfoResponse>>> {
    let config = state.config_manager().get_config().await;

    let resp = ClusterInfoResponse {
        cluster_id: config.cluster.id,
        cluster_name: config.cluster.name,
        node_id: state.node_id.clone(),
        version: config.system.version,
        started_at: state.started_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(resp)))
}
