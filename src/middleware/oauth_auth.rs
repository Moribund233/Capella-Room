use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    services::auth_service::Claims,
    services::oauth_service::OAuthClaims,
    state::AppState,
};

/// 认证信息：OAuth access_token 或 CapellaRoom 系统 JWT
#[derive(Clone)]
pub enum AppAuth {
    /// OAuth access_token（来自 OAuth 授权码 / client_credentials 流程）
    OAuth(OAuthClaims),
    /// CapellaRoom 系统 JWT（内部用户）
    User(Claims),
}

/// 从 AppAuth 中提取用户 ID
pub fn extract_user_id_from_auth(auth: &AppAuth) -> Result<Uuid> {
    match auth {
        AppAuth::User(claims) => {
            Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))
        }
        AppAuth::OAuth(claims) => {
            Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的 OAuth 用户 ID".to_string()))
        }
    }
}

/// 从 AppAuth 中提取 app_id（仅 OAuth token 有）
pub fn extract_app_id_from_auth(auth: &AppAuth) -> Option<Uuid> {
    match auth {
        AppAuth::OAuth(claims) => {
            Uuid::parse_str(&claims.aud).ok()
        }
        AppAuth::User(_) => None,
    }
}

/// OAuth 兼容认证中间件
///
/// 验证顺序：
/// 1. 先尝试解析为 OAuth access_token（适用于外部服务）
/// 2. 回退到 CapellaRoom 系统 JWT（适用于内部用户）
///
/// 通过后注入 `Extension<AppAuth>`
pub async fn oauth_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(value) if value.starts_with("Bearer ") => &value[7..],
        _ => {
            return unauthorized("缺少 Authorization 头或格式错误");
        }
    };

    // 先尝试 OAuth access_token
    if let Ok(oauth_claims) = state.oauth_service().verify_access_token(token) {
        request.extensions_mut().insert(AppAuth::OAuth(oauth_claims));
        return next.run(request).await;
    }

    // 回退到 CapellaRoom 系统 JWT
    match state.auth_service().verify_access_token(token) {
        Ok(claims) => {
            match Uuid::parse_str(&claims.sub) {
                Ok(user_id) => {
                    match state.user_service().get_user_by_id(user_id).await {
                        Ok(Some(_)) => {
                            request.extensions_mut().insert(AppAuth::User(claims));
                            next.run(request).await
                        }
                        Ok(None) => unauthorized("用户不存在或已被删除"),
                        Err(_) => internal_error("验证用户信息时出错"),
                    }
                }
                Err(_) => unauthorized("令牌中包含无效的用户 ID"),
            }
        }
        Err(_) => unauthorized("认证失败，无法解析 access token"),
    }
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "success": false,
            "code": "AUTH_ERROR",
            "error": "认证失败",
            "message": message
        })),
    )
        .into_response()
}

fn internal_error(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "code": "INTERNAL_ERROR",
            "error": "服务器错误",
            "message": message
        })),
    )
        .into_response()
}
