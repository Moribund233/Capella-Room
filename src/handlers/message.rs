use axum::{
    extract::{ConnectInfo, Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use std::collections::HashMap;

use crate::{
    error::{AppError, Result},
    models::message::{EditMessageRequest, MessageEditResponse, MessageResponse},
    models::message_reaction::ReactionSummary,
    models::response::ApiResponse,
    services::auth_service::Claims,
    state::AppState,
};

/// 消息列表响应
#[derive(Debug, serde::Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub total: i64,
    pub has_more: bool,
}

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
) -> Result<Json<ApiResponse<MessageListResponse>>> {
    let limit = query.limit.min(100);

    let mut messages = state
        .message_service
        .get_room_messages(room_id, limit, query.before)
        .await?;

    // 批量加载消息反应
    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let reactions = state
        .reaction_service
        .get_messages_reactions(&message_ids)
        .await?;
    attach_reactions(&mut messages, &reactions);

    let total = messages.len() as i64;
    let has_more = total >= limit;

    Ok(Json(ApiResponse::success(MessageListResponse {
        messages,
        total,
        has_more,
    })))
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
    query
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let limit = query.limit.min(100);

    let mut messages = state
        .message_service
        .search_messages(query.room_id, &query.q, limit)
        .await?;

    // 批量加载消息反应
    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let reactions = state
        .reaction_service
        .get_messages_reactions(&message_ids)
        .await?;
    attach_reactions(&mut messages, &reactions);

    Ok(Json(messages))
}

/// 将反应摘要附加到消息响应中
fn attach_reactions(
    messages: &mut [MessageResponse],
    reactions: &HashMap<Uuid, Vec<ReactionSummary>>,
) {
    for msg in messages.iter_mut() {
        if let Some(reaction_list) = reactions.get(&msg.id) {
            msg.reactions = Some(reaction_list.clone());
        }
    }
}

/// 删除消息
pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(claims): Extension<Claims>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .message_service
        .delete_message(message_id, user_id)
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_message_action(user_id, role, message_id, "delete", ip)
            .await;
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "消息已删除"
    })))
}

/// 编辑消息
pub async fn edit_message(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(claims): Extension<Claims>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<EditMessageRequest>,
) -> Result<Json<MessageResponse>> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let message = state
        .message_service
        .edit_message(message_id, user_id, &request.content)
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_message_action(user_id, role, message_id, "edit", ip)
            .await;
    });

    let sender = state
        .message_service
        .get_sender_info(message.sender_id)
        .await?;

    Ok(Json(message.to_response(sender)))
}

/// 获取消息编辑历史
#[derive(Debug, Deserialize)]
pub struct EditHistoryQuery {
    #[serde(default = "default_history_limit")]
    pub limit: i64,
}

fn default_history_limit() -> i64 {
    20
}

pub async fn get_message_edit_history(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<Uuid>,
    Query(query): Query<EditHistoryQuery>,
) -> Result<Json<Vec<MessageEditResponse>>> {
    let limit = query.limit.min(50);

    let history = state
        .message_service
        .get_message_edit_history(message_id, limit)
        .await?;

    Ok(Json(history))
}
