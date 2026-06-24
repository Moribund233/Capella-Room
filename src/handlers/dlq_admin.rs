use axum::{extract::{Path, State}, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

use crate::{
    error::{AppError, Result},
    middleware::admin::CurrentUserRole,
    models::response::ApiResponse,
    models::user::UserRole,
    redis::DLQManager,
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct DLQMessageResponse {
    pub id: String,
    pub original_stream: String,
    pub original_id: String,
    pub error_type: String,
    pub error_message: String,
    pub retry_count: u32,
    pub failed_at: String,
    pub source_node: String,
}

#[derive(Debug, Serialize)]
pub struct DLQListResponse {
    pub messages: Vec<DLQMessageResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct DLQStatsResponse {
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct DLQQuery {
    pub stream: Option<String>,
    pub page: Option<usize>,
    pub size: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct BatchRequeueRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchRequeueResponse {
    pub success_count: usize,
    pub failed_ids: Vec<String>,
}

fn dlq_manager(state: &Arc<AppState>) -> Result<Arc<DLQManager>> {
    let redis_mgr = state
        .redis_manager
        .as_ref()
        .ok_or(AppError::NotFound)?;

    let max_retries = {
        let config = state.config.blocking_read();
        config.redis.dlq_max_retries
    };

    Ok(Arc::new(DLQManager::new(redis_mgr.clone(), max_retries)))
}

/// GET /api/v1/admin/dlq/messages
pub async fn list_dlq_messages(
    State(state): State<Arc<AppState>>,
    Extension(current_role): Extension<CurrentUserRole>,
    query: axum::extract::Query<DLQQuery>,
) -> Result<Json<ApiResponse<DLQListResponse>>> {
    if current_role.0 != UserRole::SuperAdmin {
        return Err(AppError::Forbidden);
    }

    let mgr = dlq_manager(&state)?;
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(20).min(100);

    let start = if page == 1 { "-".to_string() } else { "+".to_string() };
    let messages = mgr.list_messages(&start, "+", size).await.map_err(|e| {
        error!("Failed to list DLQ messages: {}", e);
        AppError::Internal
    })?;

    let total = mgr.message_count().await;

    let items: Vec<DLQMessageResponse> = messages
        .into_iter()
        .map(|(id, msg)| DLQMessageResponse {
            id,
            original_stream: msg.original_stream,
            original_id: msg.original_id,
            error_type: msg.error_type,
            error_message: msg.error_message,
            retry_count: msg.retry_count,
            failed_at: msg.failed_at.to_rfc3339(),
            source_node: msg.source_node,
        })
        .collect();

    Ok(Json(ApiResponse::success(DLQListResponse {
        messages: items,
        total,
    })))
}

/// GET /api/v1/admin/dlq/stats
pub async fn dlq_stats(
    State(state): State<Arc<AppState>>,
    Extension(current_role): Extension<CurrentUserRole>,
) -> Result<Json<ApiResponse<DLQStatsResponse>>> {
    if current_role.0 != UserRole::SuperAdmin {
        return Err(AppError::Forbidden);
    }

    let mgr = dlq_manager(&state)?;
    let total = mgr.message_count().await;

    Ok(Json(ApiResponse::success(DLQStatsResponse { total })))
}

/// POST /api/v1/admin/dlq/{id}/requeue
pub async fn requeue_dlq_message(
    State(state): State<Arc<AppState>>,
    Extension(current_role): Extension<CurrentUserRole>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>> {
    if current_role.0 != UserRole::SuperAdmin {
        return Err(AppError::Forbidden);
    }

    let mgr = dlq_manager(&state)?;
    mgr.requeue(&id).await.map_err(|e| {
        error!("Failed to requeue DLQ message {}: {}", id, e);
        AppError::Internal
    })?;

    Ok(Json(ApiResponse::success(())))
}

/// POST /api/v1/admin/dlq/batch-requeue
pub async fn batch_requeue_dlq(
    State(state): State<Arc<AppState>>,
    Extension(current_role): Extension<CurrentUserRole>,
    Json(request): Json<BatchRequeueRequest>,
) -> Result<Json<ApiResponse<BatchRequeueResponse>>> {
    if current_role.0 != UserRole::SuperAdmin {
        return Err(AppError::Forbidden);
    }

    let mgr = dlq_manager(&state)?;
    let (success_count, failed_ids) = mgr.batch_requeue(&request.ids).await.map_err(|e| {
        error!("Failed to batch requeue: {}", e);
        AppError::Internal
    })?;

    Ok(Json(ApiResponse::success(BatchRequeueResponse {
        success_count,
        failed_ids,
    })))
}

/// DELETE /api/v1/admin/dlq/{id}
pub async fn delete_dlq_message(
    State(state): State<Arc<AppState>>,
    Extension(current_role): Extension<CurrentUserRole>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>> {
    if current_role.0 != UserRole::SuperAdmin {
        return Err(AppError::Forbidden);
    }

    let mgr = dlq_manager(&state)?;
    mgr.delete(&id).await.map_err(|e| {
        error!("Failed to delete DLQ message {}: {}", id, e);
        AppError::Internal
    })?;

    Ok(Json(ApiResponse::success(())))
}
