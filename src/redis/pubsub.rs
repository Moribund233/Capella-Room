use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::websocket::manager::WebSocketManager;

use super::{RedisManager, RedisPublisher};

/// 房间广播消息
/// 用于跨节点的房间消息广播
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomBroadcastMessage {
    /// 房间 ID
    pub room_id: Uuid,
    /// 消息内容
    pub message: String,
    /// 排除的用户 ID（可选）
    pub exclude_user: Option<Uuid>,
    /// 发送消息的节点 ID
    pub source_node: String,
    /// 消息时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RoomBroadcastMessage {
    /// 创建新的房间广播消息
    ///
    /// # 参数
    /// - `room_id`: 房间 ID
    /// - `message`: 消息内容
    /// - `exclude_user`: 排除的用户 ID
    /// - `source_node`: 发送消息的节点 ID
    ///
    /// # 返回
    /// 新的 RoomBroadcastMessage 实例
    pub fn new(
        room_id: Uuid,
        message: String,
        exclude_user: Option<Uuid>,
        source_node: String,
    ) -> Self {
        Self {
            room_id,
            message,
            exclude_user,
            source_node,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 将消息序列化为 JSON 字符串
    ///
    /// # 返回
    /// - 序列化成功返回 Ok(String)
    /// - 序列化失败返回 Err
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// 从 JSON 字符串反序列化消息
    ///
    /// # 参数
    /// - `json`: JSON 字符串
    ///
    /// # 返回
    /// - 反序列化成功返回 Ok(RoomBroadcastMessage)
    /// - 反序列化失败返回 Err
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// Redis Pub/Sub 管理器
/// 管理 Redis 发布和订阅功能
#[derive(Debug)]
pub struct RedisPubSub {
    manager: Arc<RedisManager>,
    publisher: Option<RedisPublisher>,
}

impl RedisPubSub {
    /// 创建新的 Redis Pub/Sub 管理器
    ///
    /// # 参数
    /// - `manager`: Redis 管理器
    ///
    /// # 返回
    /// - 如果 Redis 可用，返回 Ok(Some(RedisPubSub))
    /// - 如果 Redis 不可用，返回 Ok(None)
    pub async fn new(manager: Arc<RedisManager>) -> anyhow::Result<Option<Self>> {
        let publisher = RedisPublisher::new(manager.clone()).await?;

        Ok(Some(Self { manager, publisher }))
    }

    /// 发布房间广播消息
    ///
    /// # 参数
    /// - `room_id`: 房间 ID
    /// - `message`: 消息内容
    /// - `exclude_user`: 排除的用户 ID
    ///
    /// # 返回
    /// - 发布成功返回 Ok(())
    /// - 发布失败返回 Err
    pub async fn publish_room_message(
        &mut self,
        room_id: Uuid,
        message: String,
        exclude_user: Option<Uuid>,
    ) -> anyhow::Result<()> {
        if let Some(ref mut publisher) = self.publisher {
            let broadcast_msg = RoomBroadcastMessage::new(
                room_id,
                message,
                exclude_user,
                publisher.node_id().to_string(),
            );

            let channel = self.manager.room_channel(room_id);
            let json = broadcast_msg.to_json()?;

            publisher.publish(&channel, &json).await?;
            debug!(
                "Published room message to Redis, room_id: {}, channel: {}",
                room_id, channel
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!("Redis publisher not available"))
        }
    }

    /// 启动订阅监听器
    ///
    /// # 参数
    /// - `ws_manager`: WebSocket 管理器，用于将接收到的消息转发给本地客户端
    /// - `shutdown_rx`: 关闭信号接收器
    ///
    /// # 说明
    /// 该方法会启动一个后台任务，持续监听 Redis 房间广播消息并转发给本地客户端
    pub async fn start_subscriber(
        &self,
        ws_manager: Arc<WebSocketManager>,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> anyhow::Result<()> {
        let client = match self.manager.get_client() {
            Some(client) => client,
            None => {
                warn!("Redis client not available, skipping subscription");
                return Ok(());
            }
        };

        let room_pattern = self.manager.channel_name("room:*");
        let node_id = self.manager.node_id().to_string();
        info!(
            "Redis subscriber starting, node_id: {}, pattern: {}",
            node_id, room_pattern
        );

        tokio::spawn(async move {
            let conn = match client.get_async_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to create pubsub connection: {}", e);
                    return;
                }
            };

            let mut pubsub = conn.into_pubsub();
            if let Err(e) = pubsub.psubscribe(&room_pattern).await {
                error!("Failed to subscribe to room pattern: {}", e);
                return;
            }

            info!("Subscribed to room broadcast pattern: {}", room_pattern);

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

                                match RoomBroadcastMessage::from_json(&payload) {
                                    Ok(broadcast) => {
                                        if broadcast.source_node == node_id {
                                            continue;
                                        }

                                        debug!(
                                            "Received room broadcast from node {}: room_id={}",
                                            broadcast.source_node, broadcast.room_id
                                        );

                                        ws_manager.broadcast_local(
                                            broadcast.room_id,
                                            broadcast.message,
                                            broadcast.exclude_user,
                                        ).await;
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse broadcast message: {}", e);
                                    }
                                }
                            }
                            None => {
                                error!("PubSub message stream ended");
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Redis subscriber received shutdown signal");
                        break;
                    }
                }
            }

            info!("Redis subscriber stopped");
        });

        Ok(())
    }

    /// 检查 Redis 是否可用
    pub fn is_available(&self) -> bool {
        self.publisher.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_broadcast_message_serialization() {
        let msg = RoomBroadcastMessage::new(
            Uuid::new_v4(),
            "Hello, World!".to_string(),
            Some(Uuid::new_v4()),
            "node-1".to_string(),
        );

        let json = msg.to_json().unwrap();
        let deserialized = RoomBroadcastMessage::from_json(&json).unwrap();

        assert_eq!(msg.room_id, deserialized.room_id);
        assert_eq!(msg.message, deserialized.message);
        assert_eq!(msg.exclude_user, deserialized.exclude_user);
        assert_eq!(msg.source_node, deserialized.source_node);
    }

    #[test]
    fn test_room_broadcast_message_without_exclude() {
        let msg = RoomBroadcastMessage::new(
            Uuid::new_v4(),
            "Hello, World!".to_string(),
            None,
            "node-1".to_string(),
        );

        let json = msg.to_json().unwrap();
        let deserialized = RoomBroadcastMessage::from_json(&json).unwrap();

        assert!(deserialized.exclude_user.is_none());
    }
}
