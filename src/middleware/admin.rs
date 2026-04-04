use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{future::Future, pin::Pin, sync::Arc};
use uuid::Uuid;

use crate::{middleware::extract_claims, models::user::UserRole, state::AppState};

type AsyncResponse = Pin<Box<dyn Future<Output = Response> + Send>>;

#[allow(clippy::type_complexity)]
pub struct RoleMiddleware(
    Arc<dyn Fn(State<Arc<AppState>>, Request, Next) -> AsyncResponse + Send + Sync>,
);

impl Clone for RoleMiddleware {
    fn clone(&self) -> Self {
        RoleMiddleware(Arc::clone(&self.0))
    }
}

impl<F: Fn(State<Arc<AppState>>, Request, Next) -> AsyncResponse + Send + Sync + 'static> From<F>
    for RoleMiddleware
{
    fn from(f: F) -> Self {
        RoleMiddleware(Arc::new(f))
    }
}

/// 管理员认证中间件
/// 验证用户是否具有管理员或超级管理员权限
pub async fn admin_auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let claims = match extract_claims(&request) {
        Some(claims) => claims,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "code": "AUTH_ERROR",
                    "error": "认证失败",
                    "message": "未找到认证信息"
                })),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "code": "AUTH_ERROR",
                    "error": "认证失败",
                    "message": "无效的用户ID"
                })),
            )
                .into_response();
        }
    };

    let user = match state.user_service().get_user_by_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "code": "AUTH_ERROR",
                    "error": "认证失败",
                    "message": "用户不存在"
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "code": "INTERNAL_ERROR",
                    "error": "服务器错误",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }
    };

    if !user.role.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "code": "FORBIDDEN",
                "error": "权限不足",
                "message": "需要管理员权限"
            })),
        )
            .into_response();
    }

    let mut request = request;
    request.extensions_mut().insert(CurrentUserRole(user.role));

    next.run(request).await
}

#[derive(Clone, Debug)]
pub struct CurrentUserRole(pub UserRole);

/// 超级管理员认证中间件
/// 验证用户是否具有超级管理员权限
pub async fn super_admin_auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let claims = match extract_claims(&request) {
        Some(claims) => claims,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "code": "AUTH_ERROR",
                    "error": "认证失败",
                    "message": "未找到认证信息"
                })),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "code": "AUTH_ERROR",
                    "error": "认证失败",
                    "message": "无效的用户ID"
                })),
            )
                .into_response();
        }
    };

    let user = match state.user_service().get_user_by_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "code": "AUTH_ERROR",
                    "error": "认证失败",
                    "message": "用户不存在"
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "code": "INTERNAL_ERROR",
                    "error": "服务器错误",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }
    };

    if !user.role.is_super_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "code": "FORBIDDEN",
                "error": "权限不足",
                "message": "需要超级管理员权限"
            })),
        )
            .into_response();
    }

    next.run(request).await
}

/// 角色权限检查中间件
/// 验证用户是否具有指定角色或更高权限
pub fn role_auth_middleware(required_role: UserRole) -> RoleMiddleware {
    RoleMiddleware(Arc::new(
        move |State(state): State<Arc<AppState>>, request: Request, next: Next| {
            let required_role = required_role.clone();
            Box::pin(async move {
                let claims = match extract_claims(&request) {
                    Some(claims) => claims,
                    None => {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({
                                "success": false,
                                "code": "AUTH_ERROR",
                                "error": "认证失败",
                                "message": "未找到认证信息"
                            })),
                        )
                            .into_response();
                    }
                };

                let user_id = match Uuid::parse_str(&claims.sub) {
                    Ok(id) => id,
                    Err(_) => {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({
                                "success": false,
                                "code": "AUTH_ERROR",
                                "error": "认证失败",
                                "message": "无效的用户ID"
                            })),
                        )
                            .into_response();
                    }
                };

                let user = match state.user_service().get_user_by_id(user_id).await {
                    Ok(Some(user)) => user,
                    Ok(None) => {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({
                                "success": false,
                                "code": "AUTH_ERROR",
                                "error": "认证失败",
                                "message": "用户不存在"
                            })),
                        )
                            .into_response();
                    }
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "success": false,
                                "code": "INTERNAL_ERROR",
                                "error": "服务器错误",
                                "message": e.to_string()
                            })),
                        )
                            .into_response();
                    }
                };

                let has_permission = match required_role {
                    UserRole::SuperAdmin => user.role.is_super_admin(),
                    UserRole::Admin => user.role.is_admin(),
                    UserRole::User => true,
                };

                if !has_permission {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(json!({
                            "success": false,
                            "code": "FORBIDDEN",
                            "error": "权限不足",
                            "message": format!("需要{}权限", match required_role {
                                UserRole::SuperAdmin => "超级管理员",
                                UserRole::Admin => "管理员",
                                UserRole::User => "用户",
                            })
                        })),
                    )
                        .into_response();
                }

                next.run(request).await
            })
        },
    ))
}
