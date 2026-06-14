use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{pinned_message::PinnedMessageWithContent, response::ApiResponse},
    services::auth_service::Claims,
    state::AppState,
};

pub async fn pin_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let message = state
        .message_service()
        .get_message_by_id(message_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let room_id = message.room_id;

    state
        .pin_message_service()
        .pin_message(message_id, room_id, user_id)
        .await?;

    Ok(Json(ApiResponse::success_with_message("消息已置顶")))
}

pub async fn unpin_message(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    state
        .pin_message_service()
        .unpin_message(message_id)
        .await?;

    Ok(Json(ApiResponse::success_with_message("消息已取消置顶")))
}

pub async fn get_room_pinned_messages(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<PinnedMessageWithContent>>>> {
    let messages = state
        .pin_message_service()
        .get_room_pinned_messages(room_id)
        .await?;

    Ok(Json(ApiResponse::success(messages)))
}
