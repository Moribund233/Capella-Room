use redis::AsyncCommands;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::RedisManager;
use crate::config::{ConfigChangeEvent, ConfigManager};

/// 配置变更消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigChangeType {
    /// 单个配置项更新
    Updated,
    /// 整个配置类别更新
    CategoryUpdated,
    /// 配置重载
    Reloaded,
}

/// 配置变更同步消息
/// 用于跨节点广播配置变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSyncMessage {
    /// 消息唯一 ID
    pub id: String,
    /// 发起变更的节点 ID
    pub source_node: String,
    /// 变更类型
    pub change_type: ConfigChangeType,
    /// 配置键（Updated 类型时使用）
    pub key: Option<String>,
    /// 配置值（Updated 类型时使用）
    pub value: Option<String>,
    /// 配置类别（CategoryUpdated 类型时使用）
    pub category: Option<String>,
    /// 消息时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ConfigSyncMessage {
    /// 创建配置更新消息
    pub fn updated(source_node: String, key: String, value: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source_node,
            change_type: ConfigChangeType::Updated,
            key: Some(key),
            value: Some(value),
            category: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 创建配置类别更新消息
    pub fn category_updated(source_node: String, category: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source_node,
            change_type: ConfigChangeType::CategoryUpdated,
            key: None,
            value: None,
            category: Some(category),
            timestamp: chrono::Utc::now(),
        }
    }

    /// 创建配置重载消息
    pub fn reloaded(source_node: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source_node,
            change_type: ConfigChangeType::Reloaded,
            key: None,
            value: None,
            category: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 将消息序列化为 JSON
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// 从 JSON 反序列化消息
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// 配置同步管理器
/// 负责跨节点的配置变更同步
#[derive(Debug)]
pub struct ConfigSyncManager {
    manager: Arc<RedisManager>,
    /// 配置同步频道名称
    channel_name: String,
    /// 当前节点 ID
    node_id: String,
    /// 是否正在运行
    running: RwLock<bool>,
    /// 最后同步时间
    last_sync_at: RwLock<Option<chrono::DateTime<chrono::Utc>>>,
    /// 待处理的配置变更数（本节点发布但尚未被确认的变更数）
    pending_changes: AtomicI32,
}

impl ConfigSyncManager {
    /// 创建新的配置同步管理器
    ///
    /// # 参数
    /// - `manager`: Redis 管理器
    ///
    /// # 返回
    /// - 如果 Redis 可用，返回 Ok(Some(ConfigSyncManager))
    /// - 如果 Redis 不可用，返回 Ok(None)
    pub async fn new(manager: Arc<RedisManager>) -> anyhow::Result<Option<Arc<Self>>> {
        let channel_name = manager.channel_name("config:sync");
        let node_id = manager.node_id().to_string();

        info!(
            "ConfigSyncManager created: node_id={}, channel={}",
            node_id, channel_name
        );

        Ok(Some(Arc::new(Self {
            manager,
            channel_name,
            node_id,
            running: RwLock::new(false),
            last_sync_at: RwLock::new(None),
            pending_changes: AtomicI32::new(0),
        })))
    }

    /// 发布配置变更消息
    ///
    /// # 参数
    /// - `message`: 配置变更消息
    ///
    /// # 返回
    /// - 发布成功返回 Ok(())
    /// - 发布失败返回 Err
    pub async fn publish_change(&self, message: ConfigSyncMessage) -> anyhow::Result<()> {
        // 检查消息是否来自本节点
        if message.source_node != self.node_id {
            warn!(
                "Attempting to publish message from different node: {} (current: {})",
                message.source_node, self.node_id
            );
        }

        let json = message.to_json()?;
        let now = chrono::Utc::now();
        *self.last_sync_at.write().await = Some(now);
        self.pending_changes.fetch_add(1, Ordering::SeqCst);

        if let Some(mut conn) = self.manager.get_connection().await {
            let _: () = conn.publish(&self.channel_name, json).await?;
            debug!(
                "Published config change: type={:?}, node={}",
                message.change_type, self.node_id
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!("Redis connection not available"))
        }
    }

    /// 将 ConfigChangeEvent 转换为 ConfigSyncMessage 并发布
    ///
    /// # 参数
    /// - `event`: 配置变更事件
    ///
    /// # 返回
    /// - 发布成功返回 Ok(())
    /// - 发布失败返回 Err
    pub async fn publish_event(&self, event: &ConfigChangeEvent) -> anyhow::Result<()> {
        let message = match event {
            ConfigChangeEvent::ConfigUpdated { key, new_value, .. } => {
                ConfigSyncMessage::updated(self.node_id.clone(), key.clone(), new_value.clone())
            }
            ConfigChangeEvent::CategoryUpdated { category } => {
                ConfigSyncMessage::category_updated(self.node_id.clone(), category.clone())
            }
            ConfigChangeEvent::ConfigReloaded => ConfigSyncMessage::reloaded(self.node_id.clone()),
        };

        self.publish_change(message).await
    }

    /// 启动配置同步订阅
    ///
    /// # 参数
    /// - `config_manager`: 配置管理器，用于接收变更后更新本地配置
    ///
    /// # 说明
    /// 该方法会启动一个后台任务，持续监听 Redis 配置变更消息
    pub async fn start_subscriber(self: Arc<Self>, config_manager: Arc<ConfigManager>) {
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        info!(
            "Config sync subscriber started: node_id={}, channel={}",
            self.node_id, self.channel_name
        );

        let client = match self.manager.get_client() {
            Some(client) => client,
            None => {
                error!("Redis client not available for subscription");
                return;
            }
        };

        let node_id = self.node_id.clone();
        let channel_name = self.channel_name.clone();

        tokio::spawn(async move {
            let conn = match client.get_async_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to create pubsub connection: {}", e);
                    return;
                }
            };

            let mut pubsub = conn.into_pubsub();
            if let Err(e) = pubsub.subscribe(&channel_name).await {
                error!("Failed to subscribe to channel: {}", e);
                return;
            }

            info!("Subscribed to config sync channel: {}", channel_name);

            loop {
                tokio::select! {
                    result = async { pubsub.on_message().next().await } => {
                        match result {
                            Some(msg) => {
                                let payload: String = match msg.get_payload() {
                                    Ok(p) => p,
                                    Err(e) => {
                                        warn!("Failed to get payload from message: {}", e);
                                        continue;
                                    }
                                };

                                match ConfigSyncMessage::from_json(&payload) {
                                    Ok(remote_msg) => {
                                        if remote_msg.source_node == node_id {
                                            continue;
                                        }

                                        info!(
                                            "Received config change from node {}: key={:?}, type={:?}",
                                            remote_msg.source_node, remote_msg.key, remote_msg.change_type
                                        );

                                        *self.last_sync_at.write().await = Some(chrono::Utc::now());

                                        self.pending_changes.store(0, Ordering::SeqCst);

                                        if let Err(e) = config_manager.reload_from_database().await {
                                            error!("Failed to reload config after remote change: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse config sync message: {}", e);
                                    }
                                }
                            }
                            None => {
                                error!("PubSub message stream ended");
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                        if !*self.running.read().await {
                            break;
                        }
                    }
                }
            }

            info!("Config sync subscriber stopped");
        });
    }

    /// 停止订阅
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Config sync subscriber stopping...");
    }

    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 获取频道名称
    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }

    /// 获取节点 ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// 获取待处理变更数
    pub fn pending_changes(&self) -> i32 {
        self.pending_changes.load(Ordering::SeqCst)
    }

    /// 获取最后同步时间
    pub async fn last_sync_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        *self.last_sync_at.read().await
    }

    /// 获取当前订阅者数量
    pub async fn subscriber_count(&self) -> i32 {
        if let Some(mut conn) = self.manager.get_connection().await {
            match redis::cmd("PUBSUB")
                .arg("NUMSUB")
                .arg(&self.channel_name)
                .query_async::<_, Vec<(String, i32)>>(&mut conn)
                .await
            {
                Ok(results) => results.first().map(|(_, count)| *count).unwrap_or(0),
                Err(_) => 1,
            }
        } else {
            0
        }
    }
}

/// 配置同步桥接器
/// 将本地 ConfigChangeEvent 桥接到 Redis 同步
#[derive(Debug)]
pub struct ConfigSyncBridge {
    sync_manager: Arc<ConfigSyncManager>,
}

impl ConfigSyncBridge {
    /// 创建新的配置同步桥接器
    pub fn new(sync_manager: Arc<ConfigSyncManager>) -> Self {
        Self { sync_manager }
    }

    /// 启动桥接
    ///
    /// # 参数
    /// - `config_manager`: 配置管理器
    ///
    /// # 说明
    /// 监听本地配置变更事件，并同步到 Redis
    pub async fn start(self: Arc<Self>, config_manager: Arc<ConfigManager>) {
        let mut rx = config_manager.subscribe_config_changes();
        let node_id = self.sync_manager.node_id().to_string();

        info!("Config sync bridge started for node: {}", node_id);

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                // 发布到 Redis
                if let Err(e) = self.sync_manager.publish_event(&event).await {
                    error!("Failed to publish config change to Redis: {}", e);
                } else {
                    debug!("Bridged config change to Redis: {:?}", event);
                }
            }

            info!("Config sync bridge stopped");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;

    fn make_redis_manager(prefix: &str, node_id: &str) -> Arc<RedisManager> {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            enabled: true,
            pool_size: 10,
            timeout_secs: 5,
            channel_prefix: prefix.to_string(),
            config_sync_enabled: true,
            consumer_batch_size: 100,
            consumer_poll_interval_ms: 1000,
            stream_max_len: 10000,
        };
        Arc::new(RedisManager {
            client: None,
            connection: RwLock::new(None),
            config,
            node_id: node_id.to_string(),
        })
    }

    #[test]
    fn test_config_sync_message_serialization() {
        let msg = ConfigSyncMessage::updated(
            "node-1".to_string(),
            "jwt.expiration_hours".to_string(),
            "24".to_string(),
        );

        let json = msg.to_json().unwrap();
        let deserialized = ConfigSyncMessage::from_json(&json).unwrap();

        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.source_node, deserialized.source_node);
        assert_eq!(msg.change_type, deserialized.change_type);
        assert_eq!(msg.key, deserialized.key);
        assert_eq!(msg.value, deserialized.value);
    }

    #[test]
    fn test_config_sync_message_category_updated() {
        let msg =
            ConfigSyncMessage::category_updated("node-1".to_string(), "websocket".to_string());

        assert_eq!(msg.change_type, ConfigChangeType::CategoryUpdated);
        assert_eq!(msg.category, Some("websocket".to_string()));
        assert!(msg.key.is_none());
        assert!(msg.value.is_none());
    }

    #[test]
    fn test_config_sync_message_reloaded() {
        let msg = ConfigSyncMessage::reloaded("node-1".to_string());

        assert_eq!(msg.change_type, ConfigChangeType::Reloaded);
        assert!(msg.key.is_none());
        assert!(msg.value.is_none());
        assert!(msg.category.is_none());
    }

    #[tokio::test]
    async fn test_config_sync_manager_initial_state() {
        let manager = make_redis_manager("test", "node-1");
        let sync_mgr = ConfigSyncManager::new(manager).await.unwrap().unwrap();

        assert_eq!(sync_mgr.node_id(), "node-1");
        assert_eq!(sync_mgr.channel_name(), "test:config:sync");
        assert_eq!(sync_mgr.pending_changes(), 0);
        assert!(sync_mgr.last_sync_at().await.is_none());
    }

    #[tokio::test]
    async fn test_publish_change_increments_pending() {
        let manager = make_redis_manager("test", "node-1");
        let sync_mgr = ConfigSyncManager::new(manager).await.unwrap().unwrap();

        let msg = ConfigSyncMessage::updated(
            "node-1".to_string(),
            "test.key".to_string(),
            "test.value".to_string(),
        );

        assert!(sync_mgr.publish_change(msg).await.is_err());
        assert_eq!(sync_mgr.pending_changes(), 1);
    }

    #[tokio::test]
    async fn test_publish_event_increments_pending() {
        let manager = make_redis_manager("test", "node-1");
        let sync_mgr = ConfigSyncManager::new(manager).await.unwrap().unwrap();

        let event = ConfigChangeEvent::ConfigUpdated {
            key: "jwt.expiration_hours".to_string(),
            old_value: String::new(),
            new_value: "48".to_string(),
        };

        assert!(sync_mgr.publish_event(&event).await.is_err());
        assert_eq!(sync_mgr.pending_changes(), 1);
    }

    #[tokio::test]
    async fn test_pending_changes_accumulation() {
        let manager = make_redis_manager("test", "node-1");
        let sync_mgr = ConfigSyncManager::new(manager).await.unwrap().unwrap();

        for i in 0..3 {
            let msg = ConfigSyncMessage::updated(
                "node-1".to_string(),
                format!("key.{}", i),
                format!("value.{}", i),
            );
            let _ = sync_mgr.publish_change(msg).await;
        }

        assert_eq!(sync_mgr.pending_changes(), 3);
    }

    #[tokio::test]
    async fn test_last_sync_at_updated_on_publish() {
        let manager = make_redis_manager("test", "node-1");
        let sync_mgr = ConfigSyncManager::new(manager).await.unwrap().unwrap();

        assert!(sync_mgr.last_sync_at().await.is_none());

        let msg = ConfigSyncMessage::updated(
            "node-1".to_string(),
            "test.key".to_string(),
            "test.value".to_string(),
        );
        let _ = sync_mgr.publish_change(msg).await;

        assert!(sync_mgr.last_sync_at().await.is_some());
    }

    #[tokio::test]
    async fn test_config_sync_bridge_creation() {
        let manager = make_redis_manager("test", "node-1");
        let sync_mgr = ConfigSyncManager::new(manager).await.unwrap().unwrap();
        let bridge = ConfigSyncBridge::new(sync_mgr);
        let _ = Arc::new(bridge);
    }
}
