use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::message::MessageResponse,
    services::auth_service::Claims,
    state::AppState,
};

/// 获取聊天室消息历史查询参数
#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    /// 每页数量，默认 50，最大 100
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// 游标分页：获取此 ID 之前的消息
    pub before: Option<Uuid>,
}

fn default_limit() -> i64 {
    50
}

/// 获取聊天室消息历史
pub async fn get_room_messages(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    Query(query): Query<GetMessagesQuery>,
) -> Result<Json<Vec<MessageResponse>>> {
    let limit = query.limit.min(100);
    
    let messages = state
        .message_service
        .get_room_messages(room_id, limit, query.before)
        .await?;
    
    Ok(Json(messages))
}

/// 搜索消息查询参数
#[derive(Debug, Deserialize, Validate)]
pub struct SearchMessagesQuery {
    /// 搜索关键词
    #[validate(length(min = 1, max = 100))]
    pub q: String,
    /// 限定在某个聊天室搜索
    pub room_id: Option<Uuid>,
    /// 结果数量限制，默认 50，最大 100
    #[serde(default = "default_limit")]
    pub limit: i64,
}

/// 搜索消息
pub async fn search_messages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchMessagesQuery>,
) -> Result<Json<Vec<MessageResponse>>> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let limit = query.limit.min(100);
    
    let messages = state
        .message_service
        .search_messages(query.room_id, &query.q, limit)
        .await?;
    
    Ok(Json(messages))
}

/// 删除消息
pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    
    state.message_service.delete_message(message_id, user_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "消息已删除"
    })))
}
