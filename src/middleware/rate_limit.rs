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

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub default_requests: u32,
    pub default_window_secs: u64,
    pub auth_requests: u32,
    pub auth_window_secs: u64,
    pub message_requests: u32,
    pub message_window_secs: u64,
    pub room_requests: u32,
    pub room_window_secs: u64,
    /// 清理间隔（秒），默认30秒
    pub cleanup_interval_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_requests: 100,
            default_window_secs: 60,
            auth_requests: 5,
            auth_window_secs: 60,
            message_requests: 30,
            message_window_secs: 60,
            room_requests: 20,
            room_window_secs: 60,
            cleanup_interval_secs: 30,
        }
    }
}

#[derive(Debug)]
struct RequestRecord {
    timestamps: Vec<Instant>,
    last_cleanup: Instant,
}

impl RequestRecord {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            last_cleanup: Instant::now(),
        }
    }

    fn add_request(&mut self) {
        self.timestamps.push(Instant::now());
    }

    fn cleanup(&mut self, window: Duration, cleanup_interval: Duration) {
        let now = Instant::now();
        if now.duration_since(self.last_cleanup) < cleanup_interval {
            return;
        }

        let cutoff = now - window;
        self.timestamps.retain(|&t| t > cutoff);
        self.last_cleanup = now;
    }

    fn count_requests(&self, window: Duration) -> usize {
        let cutoff = Instant::now() - window;
        self.timestamps.iter().filter(|&&t| t > cutoff).count()
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    ip_records: DashMap<String, RwLock<RequestRecord>>,
    user_records: DashMap<String, RwLock<RequestRecord>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            ip_records: DashMap::new(),
            user_records: DashMap::new(),
            config,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(RateLimitConfig::default())
    }

    pub async fn check_ip_limit(&self, ip: &str, limit: u32, window_secs: u64) -> bool {
        let window = Duration::from_secs(window_secs);
        let cleanup_interval = Duration::from_secs(self.config.cleanup_interval_secs);

        let record = self
            .ip_records
            .entry(ip.to_string())
            .or_insert_with(|| RwLock::new(RequestRecord::new()));

        let mut record = record.write().await;

        record.cleanup(window, cleanup_interval);

        let count = record.count_requests(window);
        if count >= limit as usize {
            debug!(
                "IP {} exceeded rate limit: {}/{} in {}s",
                ip, count, limit, window_secs
            );
            return false;
        }

        record.add_request();
        true
    }

    pub async fn check_user_limit(&self, user_id: &str, limit: u32, window_secs: u64) -> bool {
        let window = Duration::from_secs(window_secs);
        let cleanup_interval = Duration::from_secs(self.config.cleanup_interval_secs);

        let record = self
            .user_records
            .entry(user_id.to_string())
            .or_insert_with(|| RwLock::new(RequestRecord::new()));

        let mut record = record.write().await;

        record.cleanup(window, cleanup_interval);

        let count = record.count_requests(window);
        if count >= limit as usize {
            debug!(
                "User {} exceeded rate limit: {}/{} in {}s",
                user_id, count, limit, window_secs
            );
            return false;
        }

        record.add_request();
        true
    }

    /// 更新配置（用于热重载）
    pub fn update_config(&mut self, config: RateLimitConfig) {
        self.config = config;
        debug!("Rate limiter configuration updated");
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &RateLimitConfig {
        &self.config
    }

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

fn extract_client_ip(connect_info: &ConnectInfo<SocketAddr>, request: &Request) -> String {
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip) = forwarded_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    connect_info.ip().to_string()
}

fn extract_user_id(request: &Request) -> Option<String> {
    request.extensions().get::<Claims>().map(|c| c.sub.clone())
}

pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let rate_limiter = match state.rate_limiter() {
        Some(rl) => rl,
        None => {
            return next.run(request).await;
        }
    };

    let path = request.uri().path().to_string();
    let addr = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0);
    let client_ip = if let Some(addr) = addr {
        extract_client_ip(&ConnectInfo(addr), &request)
    } else {
        "127.0.0.1".to_string()
    };

    // 获取读锁来访问 rate_limiter
    let (limit, window_secs) = {
        let limiter = rate_limiter.read().await;
        limiter.get_ip_limit(&path)
    };

    let ip_allowed = {
        let limiter = rate_limiter.read().await;
        limiter.check_ip_limit(&client_ip, limit, window_secs).await
    };

    if !ip_allowed {
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

    if let Some(user_id) = extract_user_id(&request) {
        let user_limit = (limit / 2).max(1);
        let user_allowed = {
            let limiter = rate_limiter.read().await;
            limiter
                .check_user_limit(&user_id, user_limit, window_secs)
                .await
        };

        if !user_allowed {
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

pub async fn strict_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let rate_limiter = match state.rate_limiter() {
        Some(rl) => rl,
        None => {
            return next.run(request).await;
        }
    };

    let addr = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0);
    let client_ip = if let Some(addr) = addr {
        extract_client_ip(&ConnectInfo(addr), &request)
    } else {
        "127.0.0.1".to_string()
    };

    let (limit, window_secs) = {
        let limiter = rate_limiter.read().await;
        (
            limiter.config.auth_requests,
            limiter.config.auth_window_secs,
        )
    };

    let ip_allowed = {
        let limiter = rate_limiter.read().await;
        limiter.check_ip_limit(&client_ip, limit, window_secs).await
    };

    if !ip_allowed {
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

        for i in 0..5 {
            let allowed = limiter.check_ip_limit("127.0.0.1", 10, 60).await;
            assert!(allowed, "Request {} should be allowed", i);
        }

        let limiter2 = RateLimiter::with_default_config();

        for _ in 0..10 {
            limiter2.check_ip_limit("192.168.1.1", 10, 60).await;
        }

        let allowed = limiter2.check_ip_limit("192.168.1.1", 10, 60).await;
        assert!(!allowed, "Request should be denied after limit");
    }

    #[tokio::test]
    async fn test_rate_limiter_user_limit() {
        let limiter = RateLimiter::with_default_config();

        for i in 0..5 {
            let allowed = limiter.check_user_limit("user123", 10, 60).await;
            assert!(allowed, "Request {} should be allowed", i);
        }

        let allowed = limiter.check_user_limit("user456", 10, 60).await;
        assert!(allowed, "Different user should not be affected");
    }

    #[tokio::test]
    async fn test_get_ip_limit() {
        let limiter = RateLimiter::with_default_config();

        let (limit, window) = limiter.get_ip_limit("/api/v1/auth/login");
        assert_eq!(limit, 5);
        assert_eq!(window, 60);

        let (limit, window) = limiter.get_ip_limit("/api/v1/messages");
        assert_eq!(limit, 30);
        assert_eq!(window, 60);

        let (limit, window) = limiter.get_ip_limit("/api/v1/rooms");
        assert_eq!(limit, 20);
        assert_eq!(window, 60);

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
