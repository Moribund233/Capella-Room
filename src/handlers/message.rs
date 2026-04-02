use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::Result,
    models::message::{MessageResponse, SendMessageRequest},
    state::AppState,
};

/// 获取聊天室消息历史
/// TODO: 实现获取消息历史逻辑（支持分页、游标分页）
pub async fn get_room_messages(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    // TODO: 添加分页参数
) -> Result<Json<Vec<MessageResponse>>> {
    // TODO: 1. 验证用户是否在房间中
    // TODO: 2. 查询消息历史
    // TODO: 3. 返回消息列表
    
    todo!("实现获取消息历史逻辑")
}

/// 搜索消息
/// TODO: 实现搜索消息逻辑
pub async fn search_messages(
    State(state): State<Arc<AppState>>,
    // TODO: 添加搜索参数
) -> Result<Json<Vec<MessageResponse>>> {
    todo!("实现搜索消息逻辑")
}

/// 删除消息
/// TODO: 实现删除消息逻辑（软删除）
pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    todo!("实现删除消息逻辑")
}
