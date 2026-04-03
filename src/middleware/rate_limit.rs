//! 速率限制中间件
//!
//! 提供基于IP和用户的请求速率限制功能
//! 支持针对不同接口设置不同的限制策略

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use serde::Serialize;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::services::auth_service::Claims;
use crate::state::AppState;

/// 速率限制配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 默认限制：时间窗口内的最大请求数
    pub default_requests: u32,
    /// 默认限制：时间窗口（秒）
    pub default_window_secs: u64,
    /// 认证接口限制（登录、注册等）
    pub auth_requests: u32,
    /// 认证接口时间窗口（秒）
    pub auth_window_secs: u64,
    /// 消息接口限制
    pub message_requests: u32,
    /// 消息接口时间窗口（秒）
    pub message_window_secs: u64,
    /// 房间接口限制
    pub room_requests: u32,
    /// 房间接口时间窗口（秒）
    pub room_window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            // 默认：100请求/分钟
            default_requests: 100,
            default_window_secs: 60,
            // 认证接口：5请求/分钟（防止暴力破解）
            auth_requests: 5,
            auth_window_secs: 60,
            // 消息接口：30请求/分钟
            message_requests: 30,
            message_window_secs: 60,
            // 房间接口：20请求/分钟
            room_requests: 20,
            room_window_secs: 60,
        }
    }
}

/// 请求记录
#[derive(Debug)]
struct RequestRecord {
    /// 请求时间戳列表
    timestamps: Vec<Instant>,
    /// 最后清理时间
    last_cleanup: Instant,
}

impl RequestRecord {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// 添加请求记录
    fn add_request(&mut self) {
        self.timestamps.push(Instant::now());
    }

    /// 清理过期的请求记录
    fn cleanup(&mut self, window: Duration) {
        let now = Instant::now();
        // 每30秒执行一次清理
        if now.duration_since(self.last_cleanup) < Duration::from_secs(30) {
            return;
        }

        let cutoff = now - window;
        self.timestamps.retain(|&t| t > cutoff);
        self.last_cleanup = now;
    }

    /// 获取时间窗口内的请求数
    fn count_requests(&self, window: Duration) -> usize {
        let cutoff = Instant::now() - window;
        self.timestamps.iter().filter(|&&t| t > cutoff).count()
    }
}

/// 速率限制器
#[derive(Debug)]
pub struct RateLimiter {
    /// IP级别的请求记录
    ip_records: DashMap<String, RwLock<RequestRecord>>,
    /// 用户级别的请求记录
    user_records: DashMap<String, RwLock<RequestRecord>>,
    /// 配置
    config: RateLimitConfig,
}

impl RateLimiter {
    /// 创建新的速率限制器
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            ip_records: DashMap::new(),
            user_records: DashMap::new(),
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// 检查IP是否超过限制
    pub async fn check_ip_limit(&self, ip: &str, limit: u32, window_secs: u64) -> bool {
        let window = Duration::from_secs(window_secs);

        // 获取或创建记录
        let record = self
            .ip_records
            .entry(ip.to_string())
            .or_insert_with(|| RwLock::new(RequestRecord::new()));

        let mut record = record.write().await;

        // 清理过期记录
        record.cleanup(window);

        // 检查是否超过限制
        let count = record.count_requests(window);
        if count >= limit as usize {
            debug!(
                "IP {} exceeded rate limit: {}/{} in {}s",
                ip, count, limit, window_secs
            );
            return false;
        }

        // 添加当前请求
        record.add_request();
        true
    }

    /// 检查用户是否超过限制
    pub async fn check_user_limit(&self, user_id: &str, limit: u32, window_secs: u64) -> bool {
        let window = Duration::from_secs(window_secs);

        // 获取或创建记录
        let record = self
            .user_records
            .entry(user_id.to_string())
            .or_insert_with(|| RwLock::new(RequestRecord::new()));

        let mut record = record.write().await;

        // 清理过期记录
        record.cleanup(window);

        // 检查是否超过限制
        let count = record.count_requests(window);
        if count >= limit as usize {
            debug!(
                "User {} exceeded rate limit: {}/{} in {}s",
                user_id, count, limit, window_secs
            );
            return false;
        }

        // 添加当前请求
        record.add_request();
        true
    }

    /// 获取IP限制配置
    pub fn get_ip_limit(&self, path: &str) -> (u32, u64) {
        if path.contains("/auth/") || path.contains("/login") || path.contains("/register") {
            (self.config.auth_requests, self.config.auth_window_secs)
        } else if path.contains("/messages") {
            (
                self.config.message_requests,
                self.config.message_window_secs,
            )
        } else if path.contains("/rooms") {
            (self.config.room_requests, self.config.room_window_secs)
        } else {
            (
                self.config.default_requests,
                self.config.default_window_secs,
            )
        }
    }
}

/// 速率限制错误响应
#[derive(Debug, Serialize)]
pub struct RateLimitError {
    pub success: bool,
    pub code: String,
    pub error: String,
    pub message: String,
    pub retry_after: u64,
}

impl RateLimitError {
    pub fn new(retry_after: u64) -> Self {
        Self {
            success: false,
            code: "RATE_LIMIT_EXCEEDED".to_string(),
            error: "请求过于频繁".to_string(),
            message: format!("请等待 {} 秒后重试", retry_after),
            retry_after,
        }
    }
}

/// 从请求中提取客户端IP
fn extract_client_ip(connect_info: &ConnectInfo<SocketAddr>, request: &Request) -> String {
    // 首先检查 X-Forwarded-For 头（代理场景）
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // 取第一个IP（客户端真实IP）
            if let Some(ip) = forwarded_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // 检查 X-Real-IP 头
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // 使用直接连接的IP
    connect_info.ip().to_string()
}

/// 从请求中提取用户ID
fn extract_user_id(request: &Request) -> Option<String> {
    request.extensions().get::<Claims>().map(|c| c.sub.clone())
}

/// 速率限制中间件
///
/// 使用方式：
/// ```rust,ignore
/// .layer(middleware::from_fn_with_state(
///     state.clone(),
///     rate_limit_middleware,
/// ))
/// ```
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    // 获取速率限制器
    let rate_limiter = match state.rate_limiter() {
        Some(rl) => rl,
        None => {
            // 如果没有配置速率限制器，直接放行
            return next.run(request).await;
        }
    };

    let path = request.uri().path().to_string();
    let client_ip = extract_client_ip(&ConnectInfo(addr), &request);

    // 获取该路径的限制配置
    let (limit, window_secs) = rate_limiter.get_ip_limit(&path);

    // 检查IP限制
    if !rate_limiter
        .check_ip_limit(&client_ip, limit, window_secs)
        .await
    {
        warn!(
            "Rate limit exceeded for IP: {} on path: {}",
            client_ip, path
        );
        let retry_after = window_secs;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(RateLimitError::new(retry_after)),
        )
            .into_response();
    }

    // 如果已登录，同时检查用户级别的限制
    if let Some(user_id) = extract_user_id(&request) {
        // 用户级别限制更严格（默认限制的一半）
        let user_limit = (limit / 2).max(1);
        if !rate_limiter
            .check_user_limit(&user_id, user_limit, window_secs)
            .await
        {
            warn!(
                "Rate limit exceeded for user: {} on path: {}",
                user_id, path
            );
            let retry_after = window_secs;
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(RateLimitError::new(retry_after)),
            )
                .into_response();
        }
    }

    next.run(request).await
}

/// 严格的速率限制中间件（用于认证接口）
///
/// 使用更严格的限制策略
pub async fn strict_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    // 获取速率限制器
    let rate_limiter = match state.rate_limiter() {
        Some(rl) => rl,
        None => {
            return next.run(request).await;
        }
    };

    let client_ip = extract_client_ip(&ConnectInfo(addr), &request);

    // 使用认证接口的严格限制
    let limit = rate_limiter.config.auth_requests;
    let window_secs = rate_limiter.config.auth_window_secs;

    // 检查IP限制
    if !rate_limiter
        .check_ip_limit(&client_ip, limit, window_secs)
        .await
    {
        warn!(
            "Strict rate limit exceeded for IP: {} on auth endpoint",
            client_ip
        );
        let retry_after = window_secs;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(RateLimitError::new(retry_after)),
        )
            .into_response();
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_ip_limit() {
        let limiter = RateLimiter::with_default_config();

        // 在限制范围内应该通过
        for i in 0..5 {
            let allowed = limiter.check_ip_limit("127.0.0.1", 10, 60).await;
            assert!(allowed, "Request {} should be allowed", i);
        }

        // 清理之前的记录，重新测试限制
        let limiter2 = RateLimiter::with_default_config();

        // 超过限制应该被拒绝
        for _ in 0..10 {
            limiter2.check_ip_limit("192.168.1.1", 10, 60).await;
        }

        // 第 11 个请求应该被拒绝
        let allowed = limiter2.check_ip_limit("192.168.1.1", 10, 60).await;
        assert!(!allowed, "Request should be denied after limit");
    }

    #[tokio::test]
    async fn test_rate_limiter_user_limit() {
        let limiter = RateLimiter::with_default_config();

        // 用户级别限制
        for i in 0..5 {
            let allowed = limiter.check_user_limit("user123", 10, 60).await;
            assert!(allowed, "Request {} should be allowed", i);
        }

        // 不同用户互不影响
        let allowed = limiter.check_user_limit("user456", 10, 60).await;
        assert!(allowed, "Different user should not be affected");
    }

    #[tokio::test]
    async fn test_get_ip_limit() {
        let limiter = RateLimiter::with_default_config();

        // 认证接口
        let (limit, window) = limiter.get_ip_limit("/api/v1/auth/login");
        assert_eq!(limit, 5);
        assert_eq!(window, 60);

        // 消息接口
        let (limit, window) = limiter.get_ip_limit("/api/v1/messages");
        assert_eq!(limit, 30);
        assert_eq!(window, 60);

        // 房间接口
        let (limit, window) = limiter.get_ip_limit("/api/v1/rooms");
        assert_eq!(limit, 20);
        assert_eq!(window, 60);

        // 默认接口
        let (limit, window) = limiter.get_ip_limit("/api/v1/users");
        assert_eq!(limit, 100);
        assert_eq!(window, 60);
    }

    #[test]
    fn test_rate_limit_error() {
        let error = RateLimitError::new(60);
        assert!(!error.success);
        assert_eq!(error.code, "RATE_LIMIT_EXCEEDED");
        assert_eq!(error.retry_after, 60);
    }
}
