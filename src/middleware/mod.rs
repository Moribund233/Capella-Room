use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::{
    error::AppError,
    services::auth_service::Claims,
    state::AppState,
};

pub mod admin;
pub mod rate_limit;

/// 认证中间件
/// 从Authorization头中提取JWT Token并验证
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    // 从请求头提取Token
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(value) if value.starts_with("Bearer ") => &value[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "code": "AUTH_ERROR",
                    "error": "认证失败",
                    "message": "缺少Authorization头或格式错误"
                })),
            )
                .into_response();
        }
    };

    // 验证Token有效性
    let claims = match state.auth_service().verify_access_token(token) {
        Ok(claims) => claims,
        Err(e) => {
            let (code, message) = match &e {
                AppError::Auth(msg) if msg.contains("过期") => ("TOKEN_EXPIRED", msg.clone()),
                AppError::Auth(msg) => ("AUTH_ERROR", msg.clone()),
                _ => ("AUTH_ERROR", "认证失败".to_string()),
            };

            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "code": code,
                    "error": "认证失败",
                    "message": message
                })),
            )
                .into_response();
        }
    };

    // 将用户信息添加到请求扩展
    request.extensions_mut().insert(claims);

    // 继续处理请求
    next.run(request).await
}

/// 可选认证中间件
/// 验证Token如果存在，但不强制要求
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    // 从请求头提取Token
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    if let Some(value) = auth_header {
        if let Some(token) = value.strip_prefix("Bearer ") {
            // 验证Token，但不强制要求成功
            if let Ok(claims) = state.auth_service().verify_access_token(token) {
                request.extensions_mut().insert(claims);
            }
        }
    }

    next.run(request).await
}

/// 日志中间件
/// 记录请求和响应信息
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    tracing::info!("{} {}", method, uri);

    let response = next.run(request).await;

    let status = response.status();
    tracing::info!("{} {} - {}", method, uri, status);

    response
}

/// 速率限制中间件（可选功能）
/// 当前为透传实现，如需启用可集成 tower-governor 等限流库
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Response {
    // 可选：实现基于IP的速率限制
    // 推荐方案：使用 tower-governor 或自定义限流逻辑
    next.run(request).await
}

/// 从请求扩展中提取Claims
pub fn extract_claims(request: &Request) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}
