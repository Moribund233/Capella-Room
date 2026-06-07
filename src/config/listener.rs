use std::sync::Arc;
use tracing::{debug, info};

use crate::config::{ConfigChangeEvent, ConfigManager};
use crate::services::batch_message_service::BatchMessageService;
use crate::websocket::manager::WebSocketManager;

/// WebSocket 配置监听器
/// 监听 WebSocket 相关配置变更并动态更新
pub struct WebSocketConfigListener {
    config_manager: Arc<ConfigManager>,
    ws_manager: Arc<WebSocketManager>,
}

impl WebSocketConfigListener {
    pub fn new(config_manager: Arc<ConfigManager>, ws_manager: Arc<WebSocketManager>) -> Self {
        Self {
            config_manager,
            ws_manager,
        }
    }

    /// 启动配置监听任务
    pub async fn run(self) {
        let mut rx = self.config_manager.subscribe_config_changes();

        info!("WebSocket config listener started");

        while let Ok(event) = rx.recv().await {
            match event {
                ConfigChangeEvent::ConfigUpdated { key, .. } => {
                    self.handle_config_change(&key).await;
                }
                ConfigChangeEvent::ConfigReloaded => {
                    self.reload_all_configs().await;
                }
                _ => {}
            }
        }

        info!("WebSocket config listener stopped");
    }

    async fn handle_config_change(&self, key: &str) {
        match key {
            "websocket.heartbeat_interval_secs" => {
                let config = self.config_manager.get_config().await;
                let secs = config.websocket.heartbeat_interval_secs;
                self.ws_manager.set_heartbeat_interval(secs).await;
                info!("WebSocket heartbeat interval updated to {}s", secs);
            }
            "websocket.heartbeat_timeout_secs" => {
                let config = self.config_manager.get_config().await;
                let secs = config.websocket.heartbeat_timeout_secs;
                self.ws_manager.set_heartbeat_timeout(secs).await;
                info!("WebSocket heartbeat timeout updated to {}s", secs);
            }
            "websocket.auth_timeout_secs" => {
                // 认证超时只对新连接生效，记录日志即可
                let config = self.config_manager.get_config().await;
                let secs = config.websocket.auth_timeout_secs;
                debug!(
                    "WebSocket auth timeout updated to {}s (will apply to new connections)",
                    secs
                );
            }
            "websocket.message_buffer_size" => {
                let config = self.config_manager.get_config().await;
                let size = config.websocket.message_buffer_size;
                self.ws_manager.set_message_buffer_size(size).await;
                info!(
                    "WebSocket message buffer size updated to {} (will apply to new connections)",
                    size
                );
            }
            _ => {}
        }
    }

    async fn reload_all_configs(&self) {
        let config = self.config_manager.get_config().await;
        self.ws_manager
            .set_heartbeat_interval(config.websocket.heartbeat_interval_secs)
            .await;
        self.ws_manager
            .set_heartbeat_timeout(config.websocket.heartbeat_timeout_secs)
            .await;
        self.ws_manager
            .set_message_buffer_size(config.websocket.message_buffer_size)
            .await;
        info!("All WebSocket configurations reloaded");
    }
}

/// 日志配置监听器
/// 监听日志级别配置变更并动态更新
pub struct LoggingConfigListener {
    config_manager: Arc<ConfigManager>,
}

impl LoggingConfigListener {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self { config_manager }
    }

    /// 启动配置监听任务
    pub async fn run(self) {
        let mut rx = self.config_manager.subscribe_config_changes();

        info!("Logging config listener started");

        while let Ok(event) = rx.recv().await {
            match event {
                ConfigChangeEvent::ConfigUpdated { key, new_value, .. } => {
                    if key == "logging.level" {
                        Self::update_log_level(&new_value).await;
                    }
                }
                ConfigChangeEvent::ConfigReloaded => {
                    let config = self.config_manager.get_config().await;
                    Self::update_log_level(&config.logging.level).await;
                }
                _ => {}
            }
        }

        info!("Logging config listener stopped");
    }

    async fn update_log_level(level: &str) {
        // 注意：tracing 的日志级别在初始化后难以动态修改
        // 这里记录日志，实际应用可以使用 tracing-subscriber 的 reload 功能
        info!(
            "Log level configuration changed to: {} (requires restart to take full effect)",
            level
        );
    }
}

/// 批量消息写入配置监听器
/// 监听 batch_message.* 配置变更并动态更新 BatchMessageService
pub struct BatchMessageConfigListener {
    config_manager: Arc<ConfigManager>,
    batch_service: Arc<BatchMessageService>,
}

impl BatchMessageConfigListener {
    pub fn new(
        config_manager: Arc<ConfigManager>,
        batch_service: Arc<BatchMessageService>,
    ) -> Self {
        Self {
            config_manager,
            batch_service,
        }
    }

    /// 启动配置监听任务
    pub async fn run(self) {
        let mut rx = self.config_manager.subscribe_config_changes();

        info!("Batch message config listener started");

        while let Ok(event) = rx.recv().await {
            match event {
                ConfigChangeEvent::ConfigUpdated { key, .. } => {
                    self.handle_config_change(&key).await;
                }
                ConfigChangeEvent::ConfigReloaded => {
                    self.reload_all_configs().await;
                }
                _ => {}
            }
        }

        info!("Batch message config listener stopped");
    }

    async fn handle_config_change(&self, key: &str) {
        match key {
            "batch_message.batch_size"
            | "batch_message.flush_interval_ms"
            | "batch_message.max_queue_size" => {
                let config = self.config_manager.get_config().await;
                self.batch_service
                    .update_config(config.batch_message.clone())
                    .await;
                info!(
                    "Batch message configuration '{}' updated, new config: batch_size={}, flush_interval={}ms, max_queue_size={}",
                    key,
                    config.batch_message.batch_size,
                    config.batch_message.flush_interval_ms,
                    config.batch_message.max_queue_size,
                );
            }
            _ => {}
        }
    }

    async fn reload_all_configs(&self) {
        let config = self.config_manager.get_config().await;
        self.batch_service
            .update_config(config.batch_message.clone())
            .await;
        info!("All batch message configurations reloaded");
    }
}

/// 启动所有配置监听器
pub fn start_config_listeners(
    config_manager: Arc<ConfigManager>,
    ws_manager: Arc<WebSocketManager>,
    batch_service: Arc<BatchMessageService>,
) {
    // 启动 WebSocket 配置监听器
    let ws_listener = WebSocketConfigListener::new(Arc::clone(&config_manager), ws_manager);
    tokio::spawn(async move {
        ws_listener.run().await;
    });

    // 启动日志配置监听器
    let logging_listener = LoggingConfigListener::new(Arc::clone(&config_manager));
    tokio::spawn(async move {
        logging_listener.run().await;
    });

    // 启动批量消息配置监听器
    let batch_listener =
        BatchMessageConfigListener::new(Arc::clone(&config_manager), batch_service);
    tokio::spawn(async move {
        batch_listener.run().await;
    });

    info!("All config listeners started");
}
