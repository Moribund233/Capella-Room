use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

/// 获取当前时间对应的日志目录路径
/// 格式: logs/yyyy-mm-dd/hh
fn get_log_dir() -> PathBuf {
    let now = chrono::Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let hour_str = now.format("%H").to_string();
    PathBuf::from("logs").join(date_str).join(hour_str)
}

/// 获取当前时间对应的日志文件名
/// 格式: yyyy-mm-dd-hh-mm.log (Windows 文件名不能包含冒号)
fn get_log_filename() -> String {
    let now = chrono::Local::now();
    format!("{}.log", now.format("%Y-%m-%d-%H-%M"))
}

/// 初始化 Windows 控制台 UTF-8 编码
/// 在 Windows 系统上设置控制台代码页为 UTF-8 (65001)
pub fn init_windows_console() {
    #[cfg(windows)]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(["/C", "chcp", "65001"]).output();
    }
}

/// 初始化日志系统
///
/// # Arguments
/// * `is_maintenance_mode` - 是否在维护模式下（维护模式会打印更详细的日志，包括文件名和行号）
pub fn init_logging(is_maintenance_mode: bool) {
    init_windows_console();

    // 创建日志目录
    let log_dir = get_log_dir();
    let log_file_path = log_dir.join(get_log_filename());

    // 确保日志目录存在
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
    }

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new("info,capella_room=debug,tower_http=debug")
    });

    // 创建日志文件
    let log_file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create log file: {}", e);
            return;
        }
    };

    // 创建非阻塞的文件写入器
    let (non_blocking, guard) = tracing_appender::non_blocking(log_file);

    // 保存 guard 到全局变量，防止被丢弃
    let _ = LOG_GUARD.set(guard);

    // 创建控制台和文件的 Layer
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_thread_ids(is_maintenance_mode)
        .with_file(is_maintenance_mode)
        .with_line_number(is_maintenance_mode)
        .with_ansi(false);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_target(true)
        .with_thread_ids(is_maintenance_mode)
        .with_file(is_maintenance_mode)
        .with_line_number(is_maintenance_mode)
        .with_ansi(false);

    // 构建 subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!(
        target: "logging",
        log_file = %log_file_path.display(),
        "Logging system initialized"
    );
}

/// 全局日志广播器实例
static GLOBAL_LOG_BROADCASTER: OnceLock<LogBroadcaster> = OnceLock::new();

/// 全局日志文件 guard，防止被丢弃导致日志写入失败
static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

/// 初始化全局日志广播器
pub fn init_global_log_broadcaster(broadcaster: LogBroadcaster) {
    let _ = GLOBAL_LOG_BROADCASTER.set(broadcaster);
}

/// 获取全局日志广播器
pub fn get_global_log_broadcaster() -> Option<&'static LogBroadcaster> {
    GLOBAL_LOG_BROADCASTER.get()
}

/// 日志条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub target: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub fields: Option<serde_json::Value>,
}

/// 日志广播器
#[derive(Debug, Clone)]
pub struct LogBroadcaster {
    sender: broadcast::Sender<LogEntry>,
}

impl LogBroadcaster {
    /// 创建新的日志广播器
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// 订阅日志
    pub fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.sender.subscribe()
    }

    /// 广播日志
    pub fn broadcast(&self, entry: LogEntry) {
        // 忽略发送失败（没有订阅者时）
        let _ = self.sender.send(entry);
    }

    /// 获取订阅者数量
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for LogBroadcaster {
    fn default() -> Self {
        Self::new(1000)
    }
}

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
    /// 广播日志条目
    fn broadcast(level: &str, target: &str, message: &str, fields: Option<serde_json::Value>) {
        if let Some(broadcaster) = get_global_log_broadcaster() {
            let entry = LogEntry {
                level: level.to_string(),
                target: target.to_string(),
                message: message.to_string(),
                timestamp: chrono::Utc::now(),
                fields,
            };
            broadcaster.broadcast(entry);
        }
    }

    /// 记录 WebSocket 连接事件
    pub fn websocket_connect(user_id: Uuid, username: &str, ip: Option<&str>) {
        let ip_str = ip.unwrap_or("unknown");
        info!(
            target: "websocket",
            user_id = %user_id,
            username = %username,
            ip = ip_str,
            event = "connect",
            "WebSocket connection established"
        );
        Self::broadcast(
            "info",
            "websocket",
            "WebSocket connection established",
            Some(serde_json::json!({
                "user_id": user_id,
                "username": username,
                "ip": ip_str,
                "event": "connect"
            })),
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
        Self::broadcast(
            "info",
            "websocket",
            "WebSocket connection closed",
            Some(serde_json::json!({
                "user_id": user_id,
                "username": username,
                "reason": reason,
                "event": "disconnect"
            })),
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
        Self::broadcast(
            "info",
            "room",
            "User joined room",
            Some(serde_json::json!({
                "user_id": user_id,
                "username": username,
                "room_id": room_id,
                "event": "join"
            })),
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
        Self::broadcast(
            "info",
            "room",
            "User left room",
            Some(serde_json::json!({
                "user_id": user_id,
                "username": username,
                "room_id": room_id,
                "event": "leave"
            })),
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
        Self::broadcast(
            "info",
            "message",
            "Message sent to room",
            Some(serde_json::json!({
                "message_id": message_id,
                "room_id": room_id,
                "user_id": user_id,
                "username": username,
                "content_length": content_length,
                "latency_ms": latency_ms,
                "event": "message_sent"
            })),
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
        Self::broadcast(
            "warn",
            "error",
            "Error occurred",
            Some(serde_json::json!({
                "error_code": error_code,
                "error_message": error_message,
                "user_id": user_id,
                "room_id": room_id,
                "event": "error"
            })),
        );
    }

    /// 记录性能事件
    pub fn performance_event(event_name: &str, duration_ms: u128, details: &str) {
        let level = if duration_ms > 100 { "warn" } else { "info" };
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
        Self::broadcast(
            level,
            "performance",
            if duration_ms > 100 {
                "Slow operation detected"
            } else {
                "Performance event"
            },
            Some(serde_json::json!({
                "event_name": event_name,
                "duration_ms": duration_ms,
                "details": details,
                "event": if duration_ms > 100 { "slow_operation" } else { "performance" }
            })),
        );
    }
}

/// 性能计时器
pub struct PerformanceTimer {
    name: String,
    start: Instant,
    finished: bool,
}

impl PerformanceTimer {
    /// 创建新的计时器
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            finished: false,
        }
    }

    /// 结束计时并记录
    pub fn finish(&mut self) -> Duration {
        self.finished = true;
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
        // 只有在未调用 finish 的情况下才报告慢操作
        // 这避免了将连接保持时间误报为操作耗时
        if !self.finished {
            let duration = self.start.elapsed();
            if duration.as_millis() > 1000 {
                StructuredLogger::performance_event(
                    &self.name,
                    duration.as_millis(),
                    "Operation took longer than expected (timer dropped without finish)",
                );
            }
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
        let mut timer = PerformanceTimer::new("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        let duration = timer.finish();
        assert!(duration.as_millis() >= 10);
    }
}
