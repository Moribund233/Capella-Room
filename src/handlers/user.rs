use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{
    error::Result,
    models::user::UserResponse,
    state::AppState,
};

/// 获取当前用户信息
/// TODO: 实现获取用户信息逻辑
pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    // TODO: 从JWT获取用户ID
) -> Result<Json<UserResponse>> {
    todo!("实现获取当前用户信息逻辑")
}

/// 更新用户信息
/// TODO: 实现更新用户信息逻辑
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    // TODO: 添加更新请求体
) -> Result<Json<UserResponse>> {
    todo!("实现更新用户信息逻辑")
}

/// 获取用户列表
/// TODO: 实现获取用户列表逻辑（支持分页和搜索）
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    // TODO: 添加查询参数
) -> Result<Json<Vec<UserResponse>>> {
    todo!("实现获取用户列表逻辑")
}
