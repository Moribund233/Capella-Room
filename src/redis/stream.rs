use redis::{aio::MultiplexedConnection, AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::RedisManager;

/// Stream 消息 trait
/// 定义可以发送到 Redis Stream 的消息类型
pub trait StreamMessage: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// 获取 Stream 名称
    fn stream_name() -> &'static str;
    /// 获取消息唯一 ID（用于去重）
    fn message_id(&self) -> String;
}

/// 审计日志 Stream 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogStreamMessage {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub actor_id: Option<Uuid>,
    pub actor_role: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub action: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub status: String,
    pub error_message: Option<String>,
    pub node_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl StreamMessage for AuditLogStreamMessage {
    fn stream_name() -> &'static str {
        "capella:stream:audit_logs"
    }

    fn message_id(&self) -> String {
        self.id.to_string()
    }
}

/// 消息 Stream 消息（聊天消息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStreamMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: String,
    pub reply_to: Option<Uuid>,
    pub node_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl StreamMessage for MessageStreamMessage {
    fn stream_name() -> &'static str {
        "capella:stream:messages"
    }

    fn message_id(&self) -> String {
        self.id.to_string()
    }
}

/// Redis Stream 生产者
/// 用于向 Redis Stream 发送消息
#[derive(Debug, Clone)]
pub struct StreamProducer {
    connection: MultiplexedConnection,
    node_id: String,
    max_len: u64,
}

impl StreamProducer {
    /// 创建新的 Stream 生产者
    ///
    /// # 参数
    /// - `manager`: Redis 管理器
    /// - `max_len`: Stream 最大长度，超过后自动裁剪
    ///
    /// # 返回
    /// - 如果连接可用，返回 Ok(Some(StreamProducer))
    /// - 如果连接不可用，返回 Ok(None)
    pub async fn new(manager: Arc<RedisManager>, max_len: u64) -> anyhow::Result<Option<Self>> {
        match manager.get_connection().await {
            Some(conn) => Ok(Some(Self {
                connection: conn,
                node_id: manager.node_id().to_string(),
                max_len,
            })),
            None => Ok(None),
        }
    }

    /// 发送消息到 Stream
    ///
    /// # 参数
    /// - `message`: 实现了 StreamMessage trait 的消息
    ///
    /// # 返回
    /// - 成功返回 Ok(消息 ID)
    /// - 失败返回 Err
    pub async fn send<T: StreamMessage>(&mut self, message: &T) -> anyhow::Result<String> {
        let stream_name = T::stream_name();
        let payload = serde_json::to_string(message)?;

        // 使用 MAXLEN 选项限制 Stream 长度，防止无限增长
        let maxlen = redis::streams::StreamMaxlen::Approx(self.max_len as usize);
        let id: String = self
            .connection
            .xadd_maxlen(stream_name, maxlen, "*", &[("payload", payload)])
            .await?;

        debug!(
            "Sent message to stream: {}, id: {}, node: {}",
            stream_name, id, self.node_id
        );
        Ok(id)
    }

    /// 批量发送消息到 Stream
    ///
    /// # 参数
    /// - `messages`: 消息列表
    ///
    /// # 返回
    /// - 成功返回 Ok(消息 ID 列表)
    /// - 失败返回 Err
    pub async fn send_batch<T: StreamMessage>(
        &mut self,
        messages: &[T],
    ) -> anyhow::Result<Vec<String>> {
        let mut ids = Vec::with_capacity(messages.len());
        for message in messages {
            match self.send(message).await {
                Ok(id) => ids.push(id),
                Err(e) => {
                    error!("Failed to send message to stream: {}", e);
                    // 继续发送其他消息
                }
            }
        }
        Ok(ids)
    }

    /// 检查生产者是否可用
    pub async fn is_available(&mut self) -> bool {
        matches!(
            redis::cmd("PING")
                .query_async::<_, String>(&mut self.connection)
                .await,
            Ok(pong) if pong == "PONG"
        )
    }
}

/// Consumer Group 配置
#[derive(Debug, Clone)]
pub struct ConsumerGroupConfig {
    /// 消费者组名称
    pub group_name: String,
    /// 消费者名称（通常是节点 ID）
    pub consumer_name: String,
    /// 批量消费大小
    pub batch_size: usize,
    /// 消费间隔（毫秒）
    pub poll_interval_ms: u64,
    /// 消息处理超时（毫秒）
    pub claim_timeout_ms: u64,
}

impl Default for ConsumerGroupConfig {
    fn default() -> Self {
        Self {
            group_name: "capella-consumers".to_string(),
            consumer_name: format!("consumer-{}", Uuid::new_v4()),
            batch_size: 100,
            poll_interval_ms: 1000,
            claim_timeout_ms: 30000,
        }
    }
}

/// Stream 消费者 trait
/// 定义消息处理逻辑
#[async_trait::async_trait]
pub trait StreamConsumerHandler: Send + Sync {
    type Message: StreamMessage;

    /// 处理单条消息
    ///
    /// # 参数
    /// - `message`: 反序列化后的消息
    /// - `message_id`: Stream 消息 ID
    ///
    /// # 返回
    /// - 处理成功返回 Ok(())
    /// - 处理失败返回 Err（消息会进入 Pending List）
    async fn handle_message(
        &self,
        message: Self::Message,
        message_id: String,
    ) -> anyhow::Result<()>;

    /// 处理批量消息
    ///
    /// # 参数
    /// - `messages`: 消息列表
    ///
    /// # 返回
    /// - 处理成功返回 Ok(())
    /// - 处理失败返回 Err
    async fn handle_batch(&self, messages: Vec<(Self::Message, String)>) -> anyhow::Result<()> {
        for (message, message_id) in messages {
            if let Err(e) = self.handle_message(message, message_id).await {
                error!("Failed to handle message: {}", e);
            }
        }
        Ok(())
    }
}

/// Redis Stream 消费者
/// 使用 Consumer Group 实现负载均衡消费
#[derive(Debug)]
pub struct StreamConsumer {
    manager: Arc<RedisManager>,
    config: ConsumerGroupConfig,
    running: RwLock<bool>,
}

impl StreamConsumer {
    /// 创建新的 Stream 消费者
    ///
    /// # 参数
    /// - `manager`: Redis 管理器
    /// - `config`: 消费者组配置
    pub fn new(manager: Arc<RedisManager>, config: ConsumerGroupConfig) -> Self {
        Self {
            manager,
            config,
            running: RwLock::new(false),
        }
    }

    /// 初始化 Consumer Group
    ///
    /// # 参数
    /// - `stream_name`: Stream 名称
    ///
    /// # 说明
    /// 如果 Group 已存在则忽略错误
    async fn init_consumer_group(&self, stream_name: &str) -> anyhow::Result<()> {
        if let Some(mut conn) = self.manager.get_connection().await {
            let result: RedisResult<String> = conn
                .xgroup_create_mkstream(stream_name, &self.config.group_name, "$")
                .await;

            match result {
                Ok(_) => info!(
                    "Created consumer group: {} for stream: {}",
                    self.config.group_name, stream_name
                ),
                Err(e) => {
                    if e.to_string().contains("BUSYGROUP") {
                        debug!(
                            "Consumer group already exists: {} for stream: {}",
                            self.config.group_name, stream_name
                        );
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
        Ok(())
    }

    /// 启动消费者
    ///
    /// # 参数
    /// - `handler`: 消息处理器
    /// - `stream_name`: Stream 名称
    pub async fn start<H, T>(self: Arc<Self>, handler: Arc<H>, stream_name: &'static str)
    where
        H: StreamConsumerHandler<Message = T>,
        T: StreamMessage,
    {
        // 初始化 Consumer Group
        if let Err(e) = self.init_consumer_group(stream_name).await {
            error!("Failed to init consumer group: {}", e);
            return;
        }

        // 设置运行状态
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        info!(
            "Stream consumer started: group={}, consumer={}, stream={}",
            self.config.group_name, self.config.consumer_name, stream_name
        );

        let mut tick = interval(Duration::from_millis(self.config.poll_interval_ms));

        loop {
            tick.tick().await;

            // 检查是否停止
            if !*self.running.read().await {
                break;
            }

            // 获取连接
            let mut conn = match self.manager.get_connection().await {
                Some(conn) => conn,
                None => {
                    warn!("Redis connection not available, retrying...");
                    continue;
                }
            };

            // 读取消息
            let opts = redis::streams::StreamReadOptions::default()
                .group(&self.config.group_name, &self.config.consumer_name)
                .count(self.config.batch_size)
                .block(self.config.poll_interval_ms as usize);

            let result: redis::RedisResult<redis::streams::StreamReadReply> =
                conn.xread_options(&[stream_name], &[">"], &opts).await;

            let messages = match result {
                Ok(reply) => reply,
                Err(e) => {
                    error!("Failed to read from stream: {}", e);
                    continue;
                }
            };

            if messages.keys.is_empty() {
                continue;
            }

            // 处理消息
            let mut to_ack = Vec::new();
            let mut parsed_messages = Vec::new();

            for stream_key in &messages.keys {
                for entry in &stream_key.ids {
                    let message_id = entry.id.clone();

                    // 解析 payload 字段
                    if let Some(value) = entry.map.get("payload") {
                        let payload_str = match value {
                            redis::Value::Data(bytes) => String::from_utf8_lossy(bytes).to_string(),
                            _ => continue,
                        };

                        match serde_json::from_str::<T>(&payload_str) {
                            Ok(msg) => {
                                parsed_messages.push((msg, message_id.clone()));
                            }
                            Err(e) => {
                                error!("Failed to parse message: {}, value: {}", e, payload_str);
                                // 解析失败也 ACK，避免死循环
                                to_ack.push(message_id.clone());
                            }
                        }
                    }
                    to_ack.push(message_id);
                }
            }

            // 批量处理消息
            if !parsed_messages.is_empty() {
                if let Err(e) = handler.handle_batch(parsed_messages).await {
                    error!("Failed to handle batch: {}", e);
                    // 处理失败，不 ACK，让消息进入 Pending List
                    continue;
                }
            }

            // ACK 消息
            if !to_ack.is_empty() {
                let ids: Vec<&str> = to_ack.iter().map(|s| s.as_str()).collect();
                match conn
                    .xack::<&str, &str, &str, ()>(stream_name, &self.config.group_name, &ids)
                    .await
                {
                    Ok(_) => {
                        debug!("ACKed {} messages", to_ack.len());
                    }
                    Err(e) => {
                        error!("Failed to ACK messages: {}", e);
                    }
                }
            }
        }

        info!("Stream consumer stopped");
    }

    /// 停止消费者
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stream consumer stopping...");
    }

    /// 处理 Pending List 中的消息（死信队列处理）
    ///
    /// # 参数
    /// - `stream_name`: Stream 名称
    /// - `min_idle_time`: 最小空闲时间（毫秒）
    ///
    /// # 返回
    /// 转移的消息数量
    pub async fn claim_pending_messages(
        &self,
        stream_name: &str,
        min_idle_time: usize,
    ) -> anyhow::Result<usize> {
        let mut conn = match self.manager.get_connection().await {
            Some(conn) => conn,
            None => return Err(anyhow::anyhow!("Redis connection not available")),
        };

        // 获取 Pending 消息
        let pending: Vec<(String, String, usize, usize)> = conn
            .xpending_count(
                stream_name,
                &self.config.group_name,
                "-",
                "+",
                self.config.batch_size,
            )
            .await?;

        if pending.is_empty() {
            return Ok(0);
        }

        let message_ids: Vec<String> = pending.iter().map(|(id, _, _, _)| id.clone()).collect();

        // 转移消息给当前消费者
        let claimed: Vec<(String, std::collections::HashMap<String, redis::Value>)> = conn
            .xclaim(
                stream_name,
                &self.config.group_name,
                &self.config.consumer_name,
                min_idle_time,
                &message_ids,
            )
            .await?;

        info!(
            "Claimed {} pending messages from stream: {}",
            claimed.len(),
            stream_name
        );

        Ok(claimed.len())
    }
}

/// Stream 管理器
/// 统一管理生产者和消费者
#[derive(Debug)]
pub struct StreamManager {
    manager: Arc<RedisManager>,
    producer: RwLock<Option<StreamProducer>>,
    max_len: u64,
}

impl StreamManager {
    /// 创建新的 Stream 管理器
    ///
    /// # 参数
    /// - `manager`: Redis 管理器
    /// - `max_len`: Stream 最大长度
    pub async fn new(
        manager: Arc<RedisManager>,
        max_len: u64,
    ) -> anyhow::Result<Option<Arc<Self>>> {
        let producer = StreamProducer::new(manager.clone(), max_len).await?;

        Ok(Some(Arc::new(Self {
            manager,
            producer: RwLock::new(producer),
            max_len,
        })))
    }

    /// 获取生产者
    pub async fn get_producer(&self) -> Option<StreamProducer> {
        self.producer.read().await.clone()
    }

    /// 重新创建生产者（连接断开后）
    pub async fn reconnect_producer(&self) -> anyhow::Result<()> {
        let producer = StreamProducer::new(self.manager.clone(), self.max_len).await?;
        let mut p = self.producer.write().await;
        *p = producer;
        Ok(())
    }

    /// 创建消费者
    ///
    /// # 参数
    /// - `config`: 消费者组配置
    pub fn create_consumer(&self, config: ConsumerGroupConfig) -> Arc<StreamConsumer> {
        Arc::new(StreamConsumer::new(self.manager.clone(), config))
    }

    /// 检查 Stream 健康状态
    pub async fn health_check(&self) -> bool {
        if let Some(mut producer) = self.get_producer().await {
            producer.is_available().await
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_message_serialization() {
        let msg = AuditLogStreamMessage {
            id: Uuid::new_v4(),
            event_type: "user_login".to_string(),
            severity: "info".to_string(),
            actor_id: Some(Uuid::new_v4()),
            actor_role: Some("user".to_string()),
            target_type: None,
            target_id: None,
            action: "login".to_string(),
            description: "User logged in".to_string(),
            metadata: None,
            status: "success".to_string(),
            error_message: None,
            node_id: "node-test".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: AuditLogStreamMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.event_type, deserialized.event_type);
    }

    #[test]
    fn test_message_stream_message_serialization() {
        let msg = MessageStreamMessage {
            id: Uuid::new_v4(),
            room_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            content: "Hello".to_string(),
            message_type: "text".to_string(),
            reply_to: None,
            node_id: "node-test".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: MessageStreamMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.content, deserialized.content);
    }

    #[test]
    fn test_consumer_group_config_default() {
        let config = ConsumerGroupConfig::default();
        assert_eq!(config.group_name, "capella-consumers");
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.poll_interval_ms, 1000);
        assert_eq!(config.claim_timeout_ms, 30000);
    }
}
