use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use super::{AppConfig, SharedConfig, SystemConfigItem, SystemConfigRecord};
use crate::db::Database;
use crate::redis::{ConfigSyncBridge, ConfigSyncManager};

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

/// 运行时配置管理器
///
/// # 职责
/// - 维护内存中的当前配置副本，供运行时读取。
/// - 从数据库 `system_configs` 表加载配置并覆盖 `config.toml` 中的值。
/// - 提供配置热重载能力，变更后即时通知订阅者。
/// - （可选）通过 Redis 在多个节点间同步配置变更。
///
/// # 设计约定
/// 服务启动后，运行时的有效配置以数据库 `system_configs` 表为准。
/// `config.toml` 仅在启动加载和初始化数据库默认值时使用。
pub struct ConfigManager {
    db: Database,
    config: SharedConfig,
    /// 配置变更事件广播发送器
    config_change_tx: broadcast::Sender<ConfigChangeEvent>,
    /// 配置同步管理器（可选，用于多节点同步）
    sync_manager: Option<Arc<ConfigSyncManager>>,
    /// 当前节点 ID
    node_id: String,
}

impl ConfigManager {
    /// 创建新的配置管理器
    ///
    /// # 参数
    /// - `db`: 数据库连接
    /// - `config`: 应用配置
    /// - `sync_manager`: 配置同步管理器（可选）
    pub fn new(
        db: Database,
        config: AppConfig,
        sync_manager: Option<Arc<ConfigSyncManager>>,
    ) -> Self {
        // 创建配置变更事件广播通道
        let (tx, _rx) = broadcast::channel::<ConfigChangeEvent>(100);

        let node_id = sync_manager
            .as_ref()
            .map(|s| s.node_id().to_string())
            .unwrap_or_else(|| format!("node-{}", uuid::Uuid::new_v4()));

        if sync_manager.is_some() {
            info!(
                "ConfigManager initialized with Redis sync support, node_id: {}",
                node_id
            );
        } else {
            info!(
                "ConfigManager initialized without Redis sync, node_id: {}",
                node_id
            );
        }

        Self {
            db,
            config: Arc::new(RwLock::new(config)),
            config_change_tx: tx,
            sync_manager,
            node_id,
        }
    }

    /// 启动配置同步
    ///
    /// # 说明
    /// 启动 Redis 订阅和桥接，实现跨节点配置同步
    pub async fn start_sync(self: Arc<Self>) {
        if let Some(ref sync_manager) = self.sync_manager {
            // 启动订阅器（接收其他节点的配置变更）
            let manager_clone = self.clone();
            sync_manager.clone().start_subscriber(manager_clone).await;

            // 启动桥接器（将本地变更广播到其他节点）
            let bridge = Arc::new(ConfigSyncBridge::new(sync_manager.clone()));
            bridge.start(self.clone()).await;

            info!("Config sync started for node: {}", self.node_id);
        } else {
            debug!("Config sync not enabled, skipping sync start");
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

    pub fn sync_manager(&self) -> Option<Arc<ConfigSyncManager>> {
        self.sync_manager.clone()
    }

    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// 从数据库重新加载配置
    ///
    /// # 说明
    /// 读取 `system_configs` 表中的所有配置项，通过 `ConfigLoader::apply_database_overrides`
    /// 覆盖内存配置，并广播 `ConfigReloaded` 事件通知所有订阅者刷新。
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

    /// 将单个数据库配置项热重载到内存配置
    ///
    /// # 说明
    /// 当 `set_config` 修改了一个可热重载的配置项后，立即更新内存中的 `AppConfig`，
    /// 无需重启服务即可生效。
    ///
    /// # 注意
    /// 新增支持热重载的配置项时，必须在此处添加对应的处理分支。
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
            "upload.chunked_upload_enabled" => {
                if let Ok(enabled) = item.value.parse() {
                    config.upload.chunked_upload_enabled = enabled;
                    debug!("Hot reloaded upload.chunked_upload_enabled = {}", enabled);
                }
            }
            "upload.default_chunk_size" => {
                if let Ok(size) = item.value.parse() {
                    config.upload.default_chunk_size = size;
                    debug!("Hot reloaded upload.default_chunk_size = {}", size);
                }
            }
            "upload.session_ttl_hours" => {
                if let Ok(hours) = item.value.parse() {
                    config.upload.session_ttl_hours = hours;
                    debug!("Hot reloaded upload.session_ttl_hours = {}", hours);
                }
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
            "websocket.message_buffer_size" => {
                if let Ok(size) = item.value.parse() {
                    config.websocket.message_buffer_size = size;
                    debug!("Hot reloaded websocket.message_buffer_size = {}", size);
                }
            }
            "logging.level" => {
                config.logging.level = item.value.clone();
                debug!("Hot reloaded logging.level = {}", item.value);
            }
            "server.login_rate_limit.max_requests" => {
                if let Ok(max_requests) = item.value.parse() {
                    config.server.login_rate_limit.max_requests = max_requests;
                    debug!("Hot reloaded server.login_rate_limit.max_requests = {}", max_requests);
                }
            }
            "server.login_rate_limit.window_secs" => {
                if let Ok(window_secs) = item.value.parse() {
                    config.server.login_rate_limit.window_secs = window_secs;
                    debug!("Hot reloaded server.login_rate_limit.window_secs = {}", window_secs);
                }
            }
            "batch_message.batch_size" => {
                if let Ok(size) = item.value.parse() {
                    config.batch_message.batch_size = size;
                    debug!("Hot reloaded batch_message.batch_size = {}", size);
                }
            }
            "batch_message.flush_interval_ms" => {
                if let Ok(ms) = item.value.parse() {
                    config.batch_message.flush_interval_ms = ms;
                    debug!("Hot reloaded batch_message.flush_interval_ms = {}ms", ms);
                }
            }
            "batch_message.max_queue_size" => {
                if let Ok(size) = item.value.parse() {
                    config.batch_message.max_queue_size = size;
                    debug!("Hot reloaded batch_message.max_queue_size = {}", size);
                }
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
            key if key.starts_with("database.") => {
                info!(
                    "Database configuration '{}' updated, restart required to take effect",
                    key
                );
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

        // 删除所有可编辑的配置
        sqlx::query("DELETE FROM system_configs WHERE is_editable = true")
            .execute(self.db.pool())
            .await?;

        // 重新插入默认配置
        let defaults = Self::get_default_configs();
        for (key, value, value_type, description, category, is_editable, is_hot_reloadable) in
            defaults
        {
            if is_editable {
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
            }
        }

        // 重新加载配置
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

    /// 将默认配置初始化到数据库
    ///
    /// # 设计约定
    /// `config.toml` 是项目默认值的唯一来源。本方法在启动时读取已经加载到内存的
    /// `AppConfig`，将支持的配置项插入 `system_configs` 表（仅当该 key 不存在时）。
    /// 这样部署人员只需修改 `config.toml` 即可调整后续实例的默认值，无需修改代码。
    ///
    /// # 注意
    /// 新增非敏感配置项时，应将其加入此处，使其能被持久化到数据库并支持热重载。
    pub async fn initialize_default_configs(&self) -> Result<()> {
        info!("Initializing default system configurations...");

        // 从当前已加载的配置中读取限流默认值，避免在代码中硬编码
        let config = self.get_config().await;
        let max_requests_str = config.server.login_rate_limit.max_requests.to_string();
        let window_secs_str = config.server.login_rate_limit.window_secs.to_string();

        let mut defaults = Self::get_default_configs();
        defaults.push((
            "server.login_rate_limit.max_requests",
            &max_requests_str,
            "int",
            "登录接口每个窗口期允许的最大请求数",
            "security",
            true,
            true,
        ));
        defaults.push((
            "server.login_rate_limit.window_secs",
            &window_secs_str,
            "int",
            "登录接口限流时间窗口（秒）",
            "security",
            true,
            true,
        ));

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

    /// 返回代码中保留的默认配置项列表
    ///
    /// # 设计约定
    /// 本方法仅用于 `initialize_default_configs` 和 `reset_to_defaults`，将默认值写入数据库。
    /// 业务默认值应来自 `config.toml`，并在 `initialize_default_configs` 中通过读取内存配置
    /// 动态注入（如限流配置）。避免在此处直接硬编码业务默认值，以保持运维人员可通过
    /// 修改 `config.toml` 调整默认行为。
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
                "database.max_connections",
                "10",
                "int",
                "数据库连接池最大连接数（需要重启服务生效）",
                "database",
                true,
                false,
            ),
            (
                "database.acquire_timeout_secs",
                "30",
                "int",
                "数据库连接获取超时时间（秒，需要重启服务生效）",
                "database",
                true,
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
                "upload.chunked_upload_enabled",
                "true",
                "bool",
                "是否启用分片上传",
                "upload",
                true,
                true,
            ),
            (
                "upload.default_chunk_size",
                "5242880",
                "int",
                "默认分片大小（字节），默认5MB",
                "upload",
                true,
                true,
            ),
            (
                "upload.session_ttl_hours",
                "24",
                "int",
                "分片上传会话过期时间（小时）",
                "upload",
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
                "Capella Room",
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
            // 审计配置
            (
                "audit.enabled",
                "true",
                "bool",
                "启用审计日志",
                "audit",
                true,
                true,
            ),
            (
                "audit.log_retention_days",
                "90",
                "int",
                "审计日志保留天数",
                "audit",
                true,
                true,
            ),
            (
                "audit.buffer_size",
                "100",
                "int",
                "审计日志缓冲区大小",
                "audit",
                true,
                true,
            ),
            (
                "audit.flush_interval_seconds",
                "5",
                "int",
                "审计日志缓冲区刷新间隔（秒）",
                "audit",
                true,
                true,
            ),
            (
                "audit.alert_enabled",
                "true",
                "bool",
                "启用审计告警检测",
                "audit",
                true,
                true,
            ),
            (
                "audit.alert_cooldown_minutes",
                "10",
                "int",
                "审计告警冷却时间（分钟）",
                "audit",
                true,
                true,
            ),
            (
                "audit.auto_archive_enabled",
                "true",
                "bool",
                "启用审计日志自动归档",
                "audit",
                true,
                true,
            ),
            (
                "audit.archive_hour",
                "3",
                "int",
                "审计日志自动归档时间（小时，0-23）",
                "audit",
                true,
                true,
            ),
            // 批量消息写入配置
            (
                "batch_message.batch_size",
                "500",
                "int",
                "批量消息写入大小（达到此数量立即写入数据库）",
                "batch_message",
                true,
                true,
            ),
            (
                "batch_message.flush_interval_ms",
                "50",
                "int",
                "批量写入刷新间隔（毫秒）",
                "batch_message",
                true,
                true,
            ),
            (
                "batch_message.max_queue_size",
                "100000",
                "int",
                "批量写入队列上限（超过后丢弃最旧消息）",
                "batch_message",
                true,
                true,
            ),
            // 注意：Redis 配置完全通过环境变量控制，不在数据库中管理
        ]
    }
}
