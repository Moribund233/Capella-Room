use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;

/// 性能指标收集器
pub struct MetricsCollector {
    /// 总消息数
    total_messages: AtomicU64,
    /// 总连接数
    total_connections: AtomicU64,
    /// 当前在线用户数
    current_online_users: AtomicU64,
    /// 活跃房间数
    active_rooms: AtomicU64,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            total_messages: AtomicU64::new(0),
            total_connections: AtomicU64::new(0),
            current_online_users: AtomicU64::new(0),
            active_rooms: AtomicU64::new(0),
        }
    }

    /// 记录消息发送
    pub fn record_message(&self) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录新连接
    pub fn record_connection(&self) {
        self.total_connections.fetch_add(1, Ordering::Relaxed);
        self.current_online_users.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录连接断开
    pub fn record_disconnect(&self) {
        self.current_online_users.fetch_sub(1, Ordering::Relaxed);
    }

    /// 更新活跃房间数
    pub fn update_active_rooms(&self, count: u64) {
        self.active_rooms.store(count, Ordering::Relaxed);
    }

    /// 获取当前指标快照
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_messages: self.total_messages.load(Ordering::Relaxed),
            total_connections: self.total_connections.load(Ordering::Relaxed),
            current_online_users: self.current_online_users.load(Ordering::Relaxed),
            active_rooms: self.active_rooms.load(Ordering::Relaxed),
            timestamp: chrono::Utc::now(),
        }
    }

    /// 记录周期性指标日志
    pub fn log_periodic_metrics(&self) {
        let snapshot = self.get_snapshot();
        info!(
            target: "metrics",
            total_messages = snapshot.total_messages,
            total_connections = snapshot.total_connections,
            current_online_users = snapshot.current_online_users,
            active_rooms = snapshot.active_rooms,
            "Performance metrics snapshot"
        );
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 指标快照
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_messages: u64,
    pub total_connections: u64,
    pub current_online_users: u64,
    pub active_rooms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 结构化日志助手
pub struct StructuredLogger;

impl StructuredLogger {
    /// 记录 WebSocket 连接事件
    pub fn websocket_connect(user_id: Uuid, username: &str, ip: Option<&str>) {
        info!(
            target: "websocket",
            user_id = %user_id,
            username = %username,
            ip = ip.unwrap_or("unknown"),
            event = "connect",
            "WebSocket connection established"
        );
    }

    /// 记录 WebSocket 断开事件
    pub fn websocket_disconnect(user_id: Uuid, username: &str, reason: &str) {
        info!(
            target: "websocket",
            user_id = %user_id,
            username = %username,
            reason = %reason,
            event = "disconnect",
            "WebSocket connection closed"
        );
    }

    /// 记录房间加入事件
    pub fn room_join(user_id: Uuid, username: &str, room_id: Uuid) {
        info!(
            target: "room",
            user_id = %user_id,
            username = %username,
            room_id = %room_id,
            event = "join",
            "User joined room"
        );
    }

    /// 记录房间离开事件
    pub fn room_leave(user_id: Uuid, username: &str, room_id: Uuid) {
        info!(
            target: "room",
            user_id = %user_id,
            username = %username,
            room_id = %room_id,
            event = "leave",
            "User left room"
        );
    }

    /// 记录消息发送事件
    pub fn message_sent(
        message_id: Uuid,
        room_id: Uuid,
        user_id: Uuid,
        username: &str,
        content_length: usize,
        latency_ms: u128,
    ) {
        info!(
            target: "message",
            message_id = %message_id,
            room_id = %room_id,
            user_id = %user_id,
            username = %username,
            content_length,
            latency_ms,
            event = "message_sent",
            "Message sent to room"
        );
    }

    /// 记录错误事件
    pub fn error_occurred(
        error_code: &str,
        error_message: &str,
        user_id: Option<Uuid>,
        room_id: Option<Uuid>,
    ) {
        warn!(
            target: "error",
            error_code = %error_code,
            error_message = %error_message,
            user_id = ?user_id,
            room_id = ?room_id,
            event = "error",
            "Error occurred"
        );
    }

    /// 记录性能事件
    pub fn performance_event(event_name: &str, duration_ms: u128, details: &str) {
        if duration_ms > 100 {
            warn!(
                target: "performance",
                event_name = %event_name,
                duration_ms,
                details = %details,
                event = "slow_operation",
                "Slow operation detected"
            );
        } else {
            info!(
                target: "performance",
                event_name = %event_name,
                duration_ms,
                details = %details,
                event = "performance",
                "Performance event"
            );
        }
    }
}

/// 性能计时器
pub struct PerformanceTimer {
    name: String,
    start: Instant,
}

impl PerformanceTimer {
    /// 创建新的计时器
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
        }
    }

    /// 结束计时并记录
    pub fn finish(&self) -> Duration {
        let duration = self.start.elapsed();
        StructuredLogger::performance_event(
            &self.name,
            duration.as_millis(),
            &format!("Operation completed in {}ms", duration.as_millis()),
        );
        duration
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        if duration.as_millis() > 1000 {
            StructuredLogger::performance_event(
                &self.name,
                duration.as_millis(),
                "Operation took longer than expected",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        collector.record_connection();
        collector.record_message();
        collector.record_message();

        let snapshot = collector.get_snapshot();
        assert_eq!(snapshot.current_online_users, 1);
        assert_eq!(snapshot.total_messages, 2);
        assert_eq!(snapshot.total_connections, 1);
    }

    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::new("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        let duration = timer.finish();
        assert!(duration.as_millis() >= 10);
    }
}
