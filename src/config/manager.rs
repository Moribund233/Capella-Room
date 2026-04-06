use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use super::{AppConfig, SharedConfig, SystemConfigItem, SystemConfigRecord};
use crate::db::Database;

/// 配置变更事件
#[derive(Debug, Clone)]
pub enum ConfigChangeEvent {
    /// 单个配置项变更
    ConfigUpdated {
        key: String,
        old_value: String,
        new_value: String,
    },
    /// 整个配置类别变更
    CategoryUpdated { category: String },
    /// 配置重载
    ConfigReloaded,
}

pub struct ConfigManager {
    db: Database,
    config: SharedConfig,
    /// 配置变更事件广播发送器
    config_change_tx: broadcast::Sender<ConfigChangeEvent>,
}

impl ConfigManager {
    pub fn new(db: Database, config: AppConfig) -> Self {
        // 创建配置变更事件广播通道
        let (tx, _rx) = broadcast::channel::<ConfigChangeEvent>(100);

        Self {
            db,
            config: Arc::new(RwLock::new(config)),
            config_change_tx: tx,
        }
    }

    /// 订阅配置变更事件
    pub fn subscribe_config_changes(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.config_change_tx.subscribe()
    }

    /// 广播配置变更事件
    fn notify_config_change(&self, event: ConfigChangeEvent) {
        let _ = self.config_change_tx.send(event);
    }

    pub fn shared_config(&self) -> SharedConfig {
        self.config.clone()
    }

    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    pub async fn reload_from_database(&self) -> Result<()> {
        info!("Reloading configuration from database...");

        let db_configs = self.load_all_from_database().await?;

        let mut config = self.config.write().await;
        super::loader::ConfigLoader::apply_database_overrides(&mut config, &db_configs);

        // 广播配置重载事件
        self.notify_config_change(ConfigChangeEvent::ConfigReloaded);

        info!("Configuration reloaded from database");
        Ok(())
    }

    async fn load_all_from_database(&self) -> Result<HashMap<String, String>> {
        let records = sqlx::query_as::<_, SystemConfigRecord>(
            "SELECT key, value, value_type, description, category, is_editable, is_hot_reloadable FROM system_configs"
        )
        .fetch_all(self.db.pool())
        .await?;

        let mut configs = HashMap::new();
        for record in records {
            configs.insert(record.key, record.value);
        }

        Ok(configs)
    }

    pub async fn get_all_configs(&self) -> Result<Vec<SystemConfigItem>> {
        let records = sqlx::query_as::<_, SystemConfigRecord>(
            "SELECT key, value, value_type, description, category, is_editable, is_hot_reloadable FROM system_configs ORDER BY category, key"
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_config_by_key(&self, key: &str) -> Result<Option<SystemConfigItem>> {
        let record = sqlx::query_as::<_, SystemConfigRecord>(
            "SELECT key, value, value_type, description, category, is_editable, is_hot_reloadable FROM system_configs WHERE key = $1"
        )
        .bind(key)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(record.map(|r| r.into()))
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<SystemConfigItem> {
        let existing = self.get_config_by_key(key).await?;

        if let Some(ref item) = existing {
            if !item.is_editable {
                return Err(anyhow::anyhow!("Configuration '{}' is not editable", key));
            }
        }

        let record = sqlx::query_as::<_, SystemConfigRecord>(
            r#"
            INSERT INTO system_configs (key, value, value_type, description, category, is_editable, is_hot_reloadable)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
            RETURNING key, value, value_type, description, category, is_editable, is_hot_reloadable
            "#
        )
        .bind(key)
        .bind(value)
        .bind(existing.as_ref().map(|i| i.value_type.as_str()).unwrap_or("string"))
        .bind(existing.as_ref().and_then(|i| i.description.clone()))
        .bind(existing.as_ref().and_then(|i| i.category.clone()))
        .bind(existing.as_ref().map(|i| i.is_editable).unwrap_or(true))
        .bind(existing.as_ref().map(|i| i.is_hot_reloadable).unwrap_or(true))
        .fetch_one(self.db.pool())
        .await?;

        let item: SystemConfigItem = record.into();

        // 获取旧值用于事件通知
        let old_value = existing
            .as_ref()
            .map(|i| i.value.clone())
            .unwrap_or_default();

        if item.is_hot_reloadable {
            self.apply_hot_reload(&item).await?;
        }

        // 广播配置变更事件
        self.notify_config_change(ConfigChangeEvent::ConfigUpdated {
            key: key.to_string(),
            old_value,
            new_value: value.to_string(),
        });

        info!("Configuration '{}' updated", key);
        Ok(item)
    }

    async fn apply_hot_reload(&self, item: &SystemConfigItem) -> Result<()> {
        let mut config = self.config.write().await;

        match item.key.as_str() {
            "jwt.expiration_hours" => {
                if let Ok(hours) = item.value.parse() {
                    config.jwt.expiration_hours = hours;
                    debug!("Hot reloaded jwt.expiration_hours = {}", hours);
                }
            }
            "upload.max_file_size" => {
                if let Ok(size) = item.value.parse() {
                    config.upload.max_file_size = size;
                    debug!("Hot reloaded upload.max_file_size = {}", size);
                }
            }
            "upload.base_url" => {
                config.upload.base_url = item.value.clone();
                debug!("Hot reloaded upload.base_url = {}", item.value);
            }
            "websocket.heartbeat_interval_secs" => {
                if let Ok(secs) = item.value.parse() {
                    config.websocket.heartbeat_interval_secs = secs;
                    debug!("Hot reloaded websocket.heartbeat_interval_secs = {}", secs);
                }
            }
            "websocket.heartbeat_timeout_secs" => {
                if let Ok(secs) = item.value.parse() {
                    config.websocket.heartbeat_timeout_secs = secs;
                    debug!("Hot reloaded websocket.heartbeat_timeout_secs = {}", secs);
                }
            }
            "websocket.auth_timeout_secs" => {
                if let Ok(secs) = item.value.parse() {
                    config.websocket.auth_timeout_secs = secs;
                    debug!("Hot reloaded websocket.auth_timeout_secs = {}", secs);
                }
            }
            "logging.level" => {
                config.logging.level = item.value.clone();
                debug!("Hot reloaded logging.level = {}", item.value);
            }
            "system.maintenance_mode" => {
                if let Ok(mode) = item.value.parse() {
                    config.system.maintenance_mode = mode;
                    debug!("Hot reloaded system.maintenance_mode = {}", mode);
                }
            }
            "system.maintenance_message" => {
                config.system.maintenance_message = item.value.clone();
                debug!("Hot reloaded system.maintenance_message = {}", item.value);
            }
            _ => {
                warn!("Unknown configuration key for hot reload: {}", item.key);
            }
        }

        Ok(())
    }

    pub async fn delete_config(&self, key: &str) -> Result<()> {
        let result =
            sqlx::query("DELETE FROM system_configs WHERE key = $1 AND is_editable = true")
                .bind(key)
                .execute(self.db.pool())
                .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!(
                "Configuration '{}' not found or not deletable",
                key
            ));
        }

        info!("Configuration '{}' deleted", key);
        Ok(())
    }

    pub async fn reset_to_defaults(&self) -> Result<Vec<SystemConfigItem>> {
        info!("Resetting all configurations to defaults...");

        sqlx::query("DELETE FROM system_configs WHERE is_editable = true")
            .execute(self.db.pool())
            .await?;

        self.reload_from_database().await?;

        info!("Configurations reset to defaults");
        self.get_all_configs().await
    }

    pub async fn get_configs_by_category(&self, category: &str) -> Result<Vec<SystemConfigItem>> {
        let records = sqlx::query_as::<_, SystemConfigRecord>(
            "SELECT key, value, value_type, description, category, is_editable, is_hot_reloadable FROM system_configs WHERE category = $1 ORDER BY key"
        )
        .bind(category)
        .fetch_all(self.db.pool())
        .await?;

        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    pub async fn initialize_default_configs(&self) -> Result<()> {
        info!("Initializing default system configurations...");

        let defaults = Self::get_default_configs();

        for (key, value, value_type, description, category, is_editable, is_hot_reloadable) in
            defaults
        {
            let exists: Option<String> =
                sqlx::query_scalar("SELECT key FROM system_configs WHERE key = $1")
                    .bind(key)
                    .fetch_optional(self.db.pool())
                    .await?;

            if exists.is_none() {
                sqlx::query(
                    r#"
                    INSERT INTO system_configs (key, value, value_type, description, category, is_editable, is_hot_reloadable)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#
                )
                .bind(key)
                .bind(value)
                .bind(value_type)
                .bind(description)
                .bind(category)
                .bind(is_editable)
                .bind(is_hot_reloadable)
                .execute(self.db.pool())
                .await?;

                debug!("Initialized default config: {} = {}", key, value);
            }
        }

        info!("Default system configurations initialized");
        Ok(())
    }

    fn get_default_configs() -> Vec<(
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        bool,
        bool,
    )> {
        vec![
            (
                "server.host",
                "0.0.0.0",
                "string",
                "服务器监听地址",
                "server",
                false,
                false,
            ),
            (
                "server.port",
                "3000",
                "int",
                "服务器端口",
                "server",
                false,
                false,
            ),
            (
                "jwt.expiration_hours",
                "24",
                "int",
                "JWT Token 过期时间（小时）",
                "security",
                true,
                true,
            ),
            (
                "upload.max_file_size",
                "10485760",
                "int",
                "最大文件大小（字节）",
                "upload",
                true,
                true,
            ),
            (
                "upload.base_url",
                "/uploads",
                "string",
                "文件访问基础URL路径",
                "upload",
                true,
                true,
            ),
            (
                "rate_limit.enabled",
                "true",
                "bool",
                "是否启用速率限制",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.default_requests",
                "100",
                "int",
                "默认限制：时间窗口内的最大请求数",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.default_window_secs",
                "60",
                "int",
                "默认限制：时间窗口（秒）",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.auth_requests",
                "5",
                "int",
                "认证接口限制",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.auth_window_secs",
                "60",
                "int",
                "认证接口时间窗口（秒）",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.message_requests",
                "30",
                "int",
                "消息接口限制",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.message_window_secs",
                "60",
                "int",
                "消息接口时间窗口（秒）",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.room_requests",
                "20",
                "int",
                "房间接口限制",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.room_window_secs",
                "60",
                "int",
                "房间接口时间窗口（秒）",
                "rate_limit",
                true,
                true,
            ),
            (
                "rate_limit.cleanup_interval_secs",
                "30",
                "int",
                "速率限制清理间隔（秒）",
                "rate_limit",
                true,
                true,
            ),
            (
                "websocket.heartbeat_interval_secs",
                "30",
                "int",
                "WebSocket 心跳间隔（秒）",
                "websocket",
                true,
                true,
            ),
            (
                "websocket.heartbeat_timeout_secs",
                "90",
                "int",
                "WebSocket 心跳超时（秒）",
                "websocket",
                true,
                true,
            ),
            (
                "websocket.auth_timeout_secs",
                "30",
                "int",
                "WebSocket 认证超时（秒）",
                "websocket",
                true,
                true,
            ),
            (
                "websocket.message_buffer_size",
                "100",
                "int",
                "WebSocket 消息缓冲区大小",
                "websocket",
                true,
                true,
            ),
            (
                "logging.level",
                "info",
                "string",
                "日志级别",
                "logging",
                true,
                true,
            ),
            (
                "logging.structured",
                "true",
                "bool",
                "是否启用结构化日志",
                "logging",
                true,
                true,
            ),
            (
                "system.name",
                "Seredeli Room",
                "string",
                "系统名称",
                "system",
                true,
                true,
            ),
            (
                "system.version",
                "1.0.0",
                "string",
                "系统版本",
                "system",
                false,
                false,
            ),
            (
                "system.maintenance_mode",
                "false",
                "bool",
                "维护模式",
                "system",
                true,
                true,
            ),
            (
                "system.maintenance_message",
                "System is under maintenance, please try again later.",
                "string",
                "维护模式提示信息",
                "system",
                true,
                true,
            ),
        ]
    }
}
