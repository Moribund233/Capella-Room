use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::Result, models::response::ApiResponse, services::auth_service::Claims, state::AppState,
    websocket::protocol::Notification,
};

/// 获取通知列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    /// 是否只返回未读通知（默认true）
    #[serde(default = "default_unread_only")]
    pub unread_only: bool,
    /// 每页数量（默认20，最大100）
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// 偏移量（默认0）
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_unread_only() -> bool {
    true
}

fn default_limit() -> i64 {
    20
}

fn default_offset() -> i64 {
    0
}

/// 通知列表响应
#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    /// 通知列表
    pub notifications: Vec<Notification>,
    /// 未读通知总数
    pub unread_count: i64,
    /// 是否还有更多
    pub has_more: bool,
}

/// 获取当前用户的通知列表
///
/// # 路径
/// GET /api/notifications
///
/// # 查询参数
/// - `unread_only`: 是否只返回未读通知（默认true）
/// - `limit`: 每页数量（默认20，最大100）
/// - `offset`: 偏移量（默认0）
///
/// # 返回
/// - 通知列表和未读计数
pub async fn get_notifications(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<ApiResponse<NotificationListResponse>>> {
    let user_id = state.auth_service().extract_user_id(&claims)?;

    let limit = query.limit.min(100);
    let offset = query.offset.max(0);

    let notifications = if query.unread_only {
        state
            .notification_service()
            .get_unread_notifications(user_id, limit)
            .await?
    } else {
        state
            .notification_service()
            .get_notifications(user_id, limit, offset)
            .await?
    };

    let unread_count = state
        .notification_service()
        .get_unread_count(user_id)
        .await?;

    let has_more = notifications.len() as i64 >= limit;

    let response = NotificationListResponse {
        notifications,
        unread_count,
        has_more,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 获取未读通知数量
///
/// # 路径
/// GET /api/notifications/unread-count
///
/// # 返回
/// - 未读通知数量
pub async fn get_unread_count(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<i64>>> {
    let user_id = state.auth_service().extract_user_id(&claims)?;

    let count = state
        .notification_service()
        .get_unread_count(user_id)
        .await?;

    Ok(Json(ApiResponse::success(count)))
}

/// 标记单个通知为已读
///
/// # 路径
/// POST /api/notifications/:id/read
///
/// # 路径参数
/// - `id`: 通知ID
///
/// # 返回
/// - 成功确认
pub async fn mark_as_read(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id = state.auth_service().extract_user_id(&claims)?;

    state
        .notification_service()
        .mark_as_read(user_id, notification_id)
        .await?;

    Ok(Json(ApiResponse::success_with_message("通知已标记为已读")))
}

/// 标记所有通知为已读
///
/// # 路径
/// POST /api/notifications/read-all
///
/// # 返回
/// - 成功确认
pub async fn mark_all_as_read(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id = state.auth_service().extract_user_id(&claims)?;

    state
        .notification_service()
        .mark_all_as_read(user_id)
        .await?;

    Ok(Json(ApiResponse::success_with_message(
        "所有通知已标记为已读",
    )))
}
