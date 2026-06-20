use axum::{
    extract::{Extension, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    middleware::oauth_auth::{extract_user_id_from_auth, AppAuth},
    models::{
        custom_event::CustomEventHttpRequest,
        response::ApiResponse,
    },
    state::AppState,
    websocket::protocol::{CustomEventForwardPayload, WebSocketMessage},
};

#[derive(Debug, Deserialize)]
pub struct GetMissedEventsParams {
    pub since: chrono::DateTime<chrono::Utc>,
}

pub async fn send_custom_event(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Json(request): Json<CustomEventHttpRequest>,
) -> Result<Json<ApiResponse<()>>> {
    if !request.event_name.contains(':') {
        return Err(AppError::Validation("事件名需使用 'namespace:event' 格式".to_string()));
    }

    let user_id = extract_user_id_from_auth(&auth)?;
    let is_member = state.room_service().is_user_in_room(request.room_id, user_id).await?;
    if !is_member {
        return Err(AppError::Forbidden);
    }

    // OAuth 应用发送事件时，source_app 用应用名，事件名 namespace 需匹配
    let source_app = match &auth {
        AppAuth::OAuth(oauth_claims) => {
            let app_id = Uuid::parse_str(&oauth_claims.aud)
                .map_err(|_| AppError::Auth("无效的 OAuth app ID".to_string()))?;
            let app = state.oauth_service().get_app(app_id).await?;
            let namespace = request.event_name.split(':').next().unwrap_or("");
            if namespace != app.name {
                return Err(AppError::Validation("事件名 namespace 与 OAuth 应用名不匹配".to_string()));
            }
            app.name
        }
        AppAuth::User(_) => {
            request.event_name.split(':').next().unwrap_or("unknown").to_string()
        }
    };

    if request.persistent.unwrap_or(false) {
        state.custom_event_service().store_event(
            &request.event_name, request.room_id, &source_app, &request.data,
        ).await?;
    }

    let forward = WebSocketMessage::CustomEventForward {
        event_name: request.event_name,
        room_id: request.room_id,
        source_app,
        data: request.data,
        timestamp: chrono::Utc::now(),
    };
    let json = forward.to_json().map_err(|_| AppError::Internal)?;
    state.ws_manager().broadcast_to_room(request.room_id, json, None).await;

    Ok(Json(ApiResponse::success_with_message("自定义事件已发送")))
}

pub async fn get_missed_events(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    axum::extract::Path(room_id): axum::extract::Path<Uuid>,
    Query(params): Query<GetMissedEventsParams>,
) -> Result<Json<ApiResponse<Vec<CustomEventForwardPayload>>>> {
    let user_id = extract_user_id_from_auth(&auth)?;

    let is_member = state.room_service().is_user_in_room(room_id, user_id).await?;
    if !is_member {
        return Err(AppError::Forbidden);
    }

    let events = state.custom_event_service()
        .get_missed_events(room_id, params.since, 100).await?;

    let payloads: Vec<_> = events.into_iter().map(|e| {
        CustomEventForwardPayload {
            id: e.id,
            event_name: e.event_name,
            source_app: e.source_app,
            data: e.data,
            timestamp: e.created_at,
        }
    }).collect();

    Ok(Json(ApiResponse::success(payloads)))
}
