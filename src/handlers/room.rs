use axum::{
    extract::{ConnectInfo, Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::response::ApiResponse,
    models::room::{
        CreateDirectRoomRequest, CreateInvitationRequest, CreateRoomRequest, DirectRoomResponse,
        JoinByInviteRequest, MemberRole, RoomInvitationResponse, RoomResponse, UpdateRoomRequest,
    },
    services::auth_service::Claims,
    services::room_service::RoomMemberWithUser,
    state::AppState,
    utils::permission::is_admin,
};

/// 查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct ListRoomsQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 设置成员角色请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SetRoleRequest {
    pub role: MemberRole,
}

/// 创建聊天室
pub async fn create_room(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<ApiResponse<RoomResponse>>> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    let max_members = request.max_members.unwrap_or(100);

    let room = state
        .room_service()
        .create_room(
            &request.name,
            request.description.as_deref(),
            user_id,
            request.is_private,
            max_members,
        )
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let room_id = room.id;
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(user_id, role, room_id, "create", ip)
            .await;
    });

    let room_detail = state
        .room_service()
        .get_room_detail(room.id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(room_detail)))
}

/// 获取聊天室列表
pub async fn list_rooms(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRoomsQuery>,
) -> Result<Json<ApiResponse<Vec<RoomResponse>>>> {
    let user_id = state.auth_service().extract_user_id(&claims).ok();

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let rooms = state
        .room_service()
        .list_rooms(user_id, query.search.as_deref(), limit, offset)
        .await?;

    Ok(Json(ApiResponse::success(rooms)))
}

/// 获取最近更新的聊天室列表
/// 按 updated_at 降序排序，返回最近活跃的房间
pub async fn list_recent_rooms(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRoomsQuery>,
) -> Result<Json<ApiResponse<Vec<RoomResponse>>>> {
    let user_id = state.auth_service().extract_user_id(&claims).ok();

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let rooms = state
        .room_service()
        .list_recent_rooms(user_id, limit, offset)
        .await?;

    Ok(Json(ApiResponse::success(rooms)))
}

/// 获取聊天室详情
pub async fn get_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<ApiResponse<RoomResponse>>> {
    let room = state
        .room_service()
        .get_room_detail(room_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(room)))
}

/// 更新聊天室信息
pub async fn update_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<UpdateRoomRequest>,
) -> Result<Json<ApiResponse<RoomResponse>>> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    // 检查权限（只有 Owner 或 Admin 可以更新）
    let can_manage = state
        .room_service()
        .can_manage_room(room_id, user_id)
        .await?;

    if !can_manage {
        return Err(AppError::Forbidden);
    }

    let _room = state
        .room_service()
        .update_room(
            room_id,
            request.name.as_deref(),
            request.description.as_deref(),
            request.is_private,
            request.max_members,
        )
        .await?;

    let room_detail = state
        .room_service()
        .get_room_detail(room_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(room_detail)))
}

/// 删除聊天室
pub async fn delete_room(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(room_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    // 检查权限（Owner 或管理员可以删除）
    let is_owner = state.room_service().is_room_owner(room_id, user_id).await?;
    let is_admin_user = is_admin(&claims.role);

    if !is_owner && !is_admin_user {
        return Err(AppError::Forbidden);
    }

    state.room_service().delete_room(room_id).await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(user_id, role, room_id, "delete", ip)
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("聊天室已删除")))
}

/// 加入聊天室
pub async fn join_room(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(room_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state.room_service().join_room(room_id, user_id).await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(user_id, role, room_id, "member_add", ip)
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("成功加入聊天室")))
}

/// 离开聊天室
pub async fn leave_room(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(room_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state.room_service().leave_room(room_id, user_id).await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(user_id, role, room_id, "member_remove", ip)
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("已离开聊天室")))
}

/// 获取聊天室成员列表
pub async fn get_room_members(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<RoomMemberWithUser>>>> {
    // 检查房间是否存在
    let _room = state
        .room_service()
        .get_room_by_id(room_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let members = state
        .room_service()
        .get_room_members_with_users(room_id)
        .await?;

    Ok(Json(ApiResponse::success(members)))
}

/// 踢出成员
pub async fn kick_member(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path((room_id, target_user_id)): Path<(Uuid, Uuid)>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let operator_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .room_service()
        .kick_member(room_id, target_user_id, operator_id)
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(operator_id, role, room_id, "member_remove", ip)
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("成员已被踢出")))
}

/// 设置成员角色
pub async fn set_member_role(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path((room_id, target_user_id)): Path<(Uuid, Uuid)>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<SetRoleRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let operator_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;

    state
        .room_service()
        .set_member_role(room_id, target_user_id, request.role, operator_id)
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(operator_id, role, room_id, "member_role_change", ip)
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("成员角色已更新")))
}

/// 获取用户加入的聊天室列表
pub async fn get_my_rooms(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<RoomResponse>>>> {
    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    let rooms = state.room_service().get_user_rooms(user_id).await?;

    Ok(Json(ApiResponse::success(rooms)))
}

/// 创建房间邀请
pub async fn create_invitation(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(room_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateInvitationRequest>,
) -> Result<Json<ApiResponse<RoomInvitationResponse>>> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    let invitation = state
        .room_service()
        .create_invitation(room_id, user_id, request.expires_in_hours, request.max_uses)
        .await?;

    // 获取邀请者信息
    let inviter = crate::models::user::UserInfo::new(
        user_id,
        claims.sub.clone(),
        None, // 头像URL需要从用户服务获取，这里简化处理
    );

    let response = invitation.to_response(inviter);

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(user_id, role, room_id, "invitation_create", ip)
            .await;
    });

    Ok(Json(ApiResponse::success(response)))
}

/// 获取房间邀请列表
pub async fn get_room_invitations(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<RoomInvitationResponse>>>> {
    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    let invitations = state
        .room_service()
        .get_room_invitations(room_id, user_id)
        .await?;

    Ok(Json(ApiResponse::success(invitations)))
}

/// 撤销房间邀请
pub async fn revoke_invitation(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path((room_id, invitation_id)): Path<(Uuid, Uuid)>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    state
        .room_service()
        .revoke_invitation(room_id, invitation_id, user_id)
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(user_id, role, room_id, "invitation_revoke", ip)
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("邀请已撤销")))
}

/// 通过邀请码加入房间
pub async fn join_by_invite(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<JoinByInviteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    let room_id = state
        .room_service()
        .join_by_invite_code(&request.invite_code, user_id)
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(user_id, role, room_id, "member_add_by_invite", ip)
            .await;
    });

    Ok(Json(ApiResponse::success_with_message("成功加入聊天室")))
}

/// 验证邀请码（不加入，仅验证）
pub async fn validate_invite_code(
    State(state): State<Arc<AppState>>,
    Query(query): Query<JoinByInviteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    query
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let invitation = state
        .room_service()
        .validate_invite_code(&query.invite_code)
        .await?;

    match invitation {
        Some(inv) => Ok(Json(ApiResponse::success(serde_json::json!({
            "valid": true,
            "room_id": inv.room_id,
            "expires_at": inv.expires_at,
            "max_uses": inv.max_uses,
            "used_count": inv.used_count,
        })))),
        None => Ok(Json(ApiResponse::success(serde_json::json!({
            "valid": false,
        })))),
    }
}

/// 创建或获取私聊房间
pub async fn create_direct_room(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateDirectRoomRequest>,
) -> Result<Json<ApiResponse<DirectRoomResponse>>> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    // 不能和自己创建私聊
    if user_id == request.target_user_id {
        return Err(AppError::Validation("不能和自己创建私聊".to_string()));
    }

    let room = state
        .room_service()
        .get_or_create_direct_room(user_id, request.target_user_id)
        .await?;

    // 记录审计日志
    let ip = addr.ip();
    let role = claims.role.clone();
    let room_id = room.id;
    let audit_service = Arc::clone(&state.audit_service);
    tokio::spawn(async move {
        let _ = audit_service
            .log_room_action(user_id, role, room_id, "direct_room_create", ip)
            .await;
    });

    Ok(Json(ApiResponse::success(room)))
}

/// 获取用户的私聊房间列表
pub async fn get_direct_rooms(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<DirectRoomResponse>>>> {
    let user_id = state
        .auth_service
        .extract_user_id(&claims)
        .map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    let rooms = state.room_service().get_user_direct_rooms(user_id).await?;

    Ok(Json(ApiResponse::success(rooms)))
}
