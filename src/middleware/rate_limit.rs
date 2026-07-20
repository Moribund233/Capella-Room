use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Extension, Request};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// 基于客户端 IP 的内存限流状态
///
/// 阈值和窗口使用原子变量存储，支持运行时热更新
#[derive(Clone)]
pub struct RateLimitState {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    window_secs: Arc<AtomicU64>,
    max_requests: Arc<AtomicUsize>,
}

impl RateLimitState {
    /// 创建新的限流状态
    ///
    /// # 参数
    /// - `max_requests`: 每个时间窗口内允许的最大请求数
    /// - `window_secs`: 时间窗口长度（秒）
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            window_secs: Arc::new(AtomicU64::new(window_secs)),
            max_requests: Arc::new(AtomicUsize::new(max_requests)),
        }
    }

    /// 更新限流配置
    ///
    /// 新的配置不会影响已记录的请求时间戳，仅在下次 `check` 时生效
    pub fn update_config(&self, max_requests: usize, window_secs: u64) {
        self.max_requests.store(max_requests, Ordering::Relaxed);
        self.window_secs.store(window_secs, Ordering::Relaxed);
    }

    fn window(&self) -> Duration {
        Duration::from_secs(self.window_secs.load(Ordering::Relaxed))
    }

    fn max_requests(&self) -> usize {
        self.max_requests.load(Ordering::Relaxed)
    }

    /// 检查并记录一次请求
    ///
    /// # 返回
    /// - `Ok(())`: 请求通过
    /// - `Err(retry_after_secs)`: 被限流，返回建议重试等待秒数
    pub fn check(&self, ip: &str) -> Result<(), u64> {
        let now = Instant::now();
        let window = self.window();
        let max_requests = self.max_requests();
        let mut requests = self.requests.lock().expect("rate limit mutex poisoned");
        let entries = requests.entry(ip.to_string()).or_default();

        // 清理过期记录
        entries.retain(|t| now.duration_since(*t) < window);

        if entries.len() >= max_requests {
            let retry_after = entries
                .first()
                .map(|first| {
                    let elapsed = now.duration_since(*first).as_secs();
                    window.as_secs().saturating_sub(elapsed)
                })
                .unwrap_or(window.as_secs())
                .max(1);
            return Err(retry_after);
        }

        entries.push(now);
        Ok(())
    }
}

/// 登录接口限流中间件
///
/// 基于客户端 IP 限制请求频率，超过阈值返回 HTTP 429
/// 并通过 `Retry-After` 响应头提示客户端等待时间
pub async fn rate_limit_middleware(
    Extension(state): Extension<RateLimitState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let ip = addr.ip().to_string();

    match state.check(&ip) {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let mut headers = HeaderMap::new();
            if let Ok(value) = retry_after.to_string().parse() {
                headers.insert("Retry-After", value);
            }

            (
                StatusCode::TOO_MANY_REQUESTS,
                headers,
                Json(json!({
                    "success": false,
                    "code": "RATE_LIMIT_EXCEEDED",
                    "error": "请求过于频繁",
                    "message": format!("登录尝试过于频繁，请 {} 秒后重试", retry_after)
                })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RateLimitState;

    #[test]
    fn test_rate_limit_allows_under_threshold() {
        let state = RateLimitState::new(3, 60);
        assert!(state.check("192.168.1.1").is_ok());
        assert!(state.check("192.168.1.1").is_ok());
        assert!(state.check("192.168.1.1").is_ok());
    }

    #[test]
    fn test_rate_limit_blocks_over_threshold() {
        let state = RateLimitState::new(2, 60);
        assert!(state.check("10.0.0.1").is_ok());
        assert!(state.check("10.0.0.1").is_ok());
        assert!(state.check("10.0.0.1").is_err());
    }

    #[test]
    fn test_rate_limit_tracks_ips_independently() {
        let state = RateLimitState::new(1, 60);
        assert!(state.check("1.1.1.1").is_ok());
        assert!(state.check("2.2.2.2").is_ok());
    }

    #[test]
    fn test_rate_limit_config_update() {
        let state = RateLimitState::new(1, 60);
        assert!(state.check("3.3.3.3").is_ok());
        assert!(state.check("3.3.3.3").is_err());

        // 放宽阈值后应允许再次请求
        state.update_config(3, 60);
        assert!(state.check("3.3.3.3").is_ok());
        assert!(state.check("3.3.3.3").is_ok());
        assert!(state.check("3.3.3.3").is_err());
    }
}
