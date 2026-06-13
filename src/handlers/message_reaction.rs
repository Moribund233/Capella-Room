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
        message_reaction::{MessageReaction, ReactionRequest, ReactionSummary},
        response::ApiResponse,
    },
    services::auth_service::Claims,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct RemoveReactionQuery {
    pub emoji: String,
}

pub async fn add_reaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<ReactionRequest>,
) -> Result<Json<ApiResponse<MessageReaction>>> {
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let reaction = state
        .reaction_service
        .add_reaction(message_id, user_id, &request.emoji)
        .await?;

    Ok(Json(ApiResponse::success(reaction)))
}

pub async fn remove_reaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(message_id): Path<Uuid>,
    Query(query): Query<RemoveReactionQuery>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .reaction_service
        .remove_reaction(message_id, user_id, &query.emoji)
        .await?;

    Ok(Json(ApiResponse::success_with_message("反应已移除")))
}

pub async fn get_message_reactions(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<ReactionSummary>>> {
    let reactions = state
        .reaction_service
        .get_message_reactions(message_id)
        .await?;

    Ok(Json(reactions))
}
