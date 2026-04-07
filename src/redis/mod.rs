use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::config::RedisConfig;

pub mod config_sync;
pub mod pubsub;
pub mod stream;

pub use config_sync::{ConfigChangeType, ConfigSyncBridge, ConfigSyncManager, ConfigSyncMessage};
pub use pubsub::{RedisPubSub, RoomBroadcastMessage};
pub use stream::{
    AuditLogStreamMessage, ConsumerGroupConfig, MessageStreamMessage, StreamConsumer,
    StreamConsumerHandler, StreamManager, StreamMessage, StreamProducer,
};

/// Redis 连接管理器
/// 管理 Redis 连接和连接池
#[derive(Debug)]
pub struct RedisManager {
    client: Option<Client>,
    connection: RwLock<Option<MultiplexedConnection>>,
    config: RedisConfig,
    node_id: String,
}

impl RedisManager {
    /// 创建新的 Redis 管理器
    ///
    /// # 参数
    /// - `config`: Redis 配置
    ///
    /// # 返回
    /// - 如果 Redis 未启用，返回 Ok(None)
    /// - 如果连接成功，返回 Ok(Some(Arc<RedisManager>))
    /// - 如果连接失败，返回 Err
    pub async fn new(config: RedisConfig) -> anyhow::Result<Option<Arc<Self>>> {
        if !config.enabled {
            info!("Redis is disabled, skipping connection");
            return Ok(None);
        }

        let client = Client::open(config.url.clone())?;
        let connection = client.get_multiplexed_async_connection().await?;

        let node_id = format!("node-{}", Uuid::new_v4());

        info!(
            "Redis connected successfully, node_id: {}, url: {}",
            node_id, config.url
        );

        Ok(Some(Arc::new(Self {
            client: Some(client),
            connection: RwLock::new(Some(connection)),
            config,
            node_id,
        })))
    }

    /// 获取当前节点 ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// 获取 Redis 连接
    ///
    /// # 返回
    /// - 如果连接可用，返回 Some(MultiplexedConnection)
    /// - 如果连接不可用，返回 None
    pub async fn get_connection(&self) -> Option<MultiplexedConnection> {
        let conn = self.connection.read().await;
        conn.clone()
    }

    /// 检查 Redis 是否可用
    pub async fn is_connected(&self) -> bool {
        if let Some(ref mut conn) = *self.connection.write().await {
            matches!(
                redis::cmd("PING").query_async::<_, String>(conn).await,
                Ok(pong) if pong == "PONG"
            )
        } else {
            false
        }
    }

    /// 重新连接 Redis
    ///
    /// # 说明
    /// 当连接断开时，尝试重新建立连接
    pub async fn reconnect(&self) -> anyhow::Result<()> {
        if let Some(ref client) = self.client {
            let connection = client.get_multiplexed_async_connection().await?;
            let mut conn = self.connection.write().await;
            *conn = Some(connection);
            info!("Redis reconnected successfully");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Redis client not initialized"))
        }
    }

    /// 获取频道名称
    ///
    /// # 参数
    /// - `suffix`: 频道后缀
    ///
    /// # 返回
    /// 完整的频道名称，格式为: {prefix}:{suffix}
    pub fn channel_name(&self, suffix: &str) -> String {
        format!("{}:{}", self.config.channel_prefix, suffix)
    }

    /// 获取房间广播频道名称
    ///
    /// # 参数
    /// - `room_id`: 房间 ID
    ///
    /// # 返回
    /// 房间广播频道名称，格式为: {prefix}:room:{room_id}
    pub fn room_channel(&self, room_id: Uuid) -> String {
        self.channel_name(&format!("room:{}", room_id))
    }

    /// 获取全局广播频道名称
    ///
    /// # 返回
    /// 全局广播频道名称，格式为: {prefix}:broadcast
    pub fn broadcast_channel(&self) -> String {
        self.channel_name("broadcast")
    }

    /// 获取 Redis 客户端
    ///
    /// # 返回
    /// - 如果客户端可用，返回 Some(Client)
    /// - 如果客户端不可用，返回 None
    pub fn get_client(&self) -> Option<Client> {
        self.client.clone()
    }
}

/// Redis 发布者
/// 用于向 Redis 发布消息
#[derive(Debug, Clone)]
pub struct RedisPublisher {
    connection: MultiplexedConnection,
    node_id: String,
}

impl RedisPublisher {
    /// 创建新的 Redis 发布者
    ///
    /// # 参数
    /// - `manager`: Redis 管理器
    ///
    /// # 返回
    /// - 如果连接可用，返回 Ok(Some(RedisPublisher))
    /// - 如果连接不可用，返回 Ok(None)
    pub async fn new(manager: Arc<RedisManager>) -> anyhow::Result<Option<Self>> {
        match manager.get_connection().await {
            Some(conn) => Ok(Some(Self {
                connection: conn,
                node_id: manager.node_id().to_string(),
            })),
            None => Ok(None),
        }
    }

    /// 发布消息到指定频道
    ///
    /// # 参数
    /// - `channel`: 频道名称
    /// - `message`: 消息内容
    ///
    /// # 返回
    /// - 发布成功返回 Ok(())
    /// - 发布失败返回 Err
    pub async fn publish(&mut self, channel: &str, message: &str) -> anyhow::Result<()> {
        let _: () = self.connection.publish(channel, message).await?;
        debug!("Published message to channel: {}", channel);
        Ok(())
    }

    /// 获取节点 ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

/// Redis 订阅者
/// 用于从 Redis 订阅消息
#[derive(Debug)]
pub struct RedisSubscriber {
    #[allow(dead_code)]
    client: Client,
    node_id: String,
}

impl RedisSubscriber {
    /// 创建新的 Redis 订阅者
    ///
    /// # 参数
    /// - `manager`: Redis 管理器
    ///
    /// # 返回
    /// - 如果客户端可用，返回 Ok(Some(RedisSubscriber))
    /// - 如果客户端不可用，返回 Ok(None)
    pub fn new(manager: Arc<RedisManager>) -> anyhow::Result<Option<Self>> {
        match &manager.client {
            Some(client) => Ok(Some(Self {
                client: client.clone(),
                node_id: manager.node_id().to_string(),
            })),
            None => Ok(None),
        }
    }

    /// 获取节点 ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_name() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            enabled: true,
            pool_size: 10,
            timeout_secs: 5,
            channel_prefix: "test".to_string(),
            config_sync_enabled: true,
            consumer_batch_size: 100,
            consumer_poll_interval_ms: 1000,
            stream_max_len: 10000,
        };

        let manager = Arc::new(RedisManager {
            client: None,
            connection: RwLock::new(None),
            config,
            node_id: "test-node".to_string(),
        });

        assert_eq!(manager.channel_name("room:123"), "test:room:123");
        assert_eq!(manager.broadcast_channel(), "test:broadcast");
    }
}
