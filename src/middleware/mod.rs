use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

/// 认证中间件
/// TODO: 实现JWT认证中间件
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Response {
    // TODO: 1. 从请求头提取Token
    // TODO: 2. 验证Token有效性
    // TODO: 3. 将用户信息添加到请求扩展
    // TODO: 4. 继续处理请求或返回401
    
    next.run(request).await
}

/// 日志中间件
/// TODO: 实现请求日志记录中间件
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    // TODO: 记录请求信息
    let response = next.run(request).await;
    // TODO: 记录响应信息
    response
}

/// 速率限制中间件
/// TODO: 实现API速率限制中间件
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Response {
    // TODO: 实现基于IP的速率限制
    next.run(request).await
}
