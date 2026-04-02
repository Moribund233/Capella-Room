use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// WebSocket连接管理器
/// 管理所有活跃的WebSocket连接和房间订阅
#[derive(Debug)]
pub struct WebSocketManager {
    /// 用户ID到发送通道的映射
    connections: DashMap<Uuid, mpsc::UnboundedSender<String>>,
    /// 房间ID到用户ID集合的映射
    room_subscribers: DashMap<Uuid, Vec<Uuid>>,
}

impl WebSocketManager {
    /// 创建新的WebSocket管理器
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: DashMap::new(),
            room_subscribers: DashMap::new(),
        })
    }

    /// 注册新连接
    pub fn connect(&self, user_id: Uuid, sender: mpsc::UnboundedSender<String>) {
        debug!("User {} connected to WebSocket", user_id);
        self.connections.insert(user_id, sender);
    }

    /// 断开连接
    pub fn disconnect(&self, user_id: Uuid) {
        debug!("User {} disconnected from WebSocket", user_id);
        self.connections.remove(&user_id);

        // 从所有房间中移除该用户
        for mut entry in self.room_subscribers.iter_mut() {
            entry.value_mut().retain(|&id| id != user_id);
        }
    }

    /// 加入房间
    pub fn join_room(&self, room_id: Uuid, user_id: Uuid) {
        debug!("User {} joined room {}", user_id, room_id);

        self.room_subscribers
            .entry(room_id)
            .or_insert_with(Vec::new)
            .push(user_id);
    }

    /// 离开房间
    pub fn leave_room(&self, room_id: Uuid, user_id: Uuid) {
        debug!("User {} left room {}", user_id, room_id);

        if let Some(mut subscribers) = self.room_subscribers.get_mut(&room_id) {
            subscribers.retain(|&id| id != user_id);
        }
    }

    /// 广播消息到房间
    pub async fn broadcast_to_room(&self, room_id: Uuid, message: String) {
        if let Some(subscribers) = self.room_subscribers.get(&room_id) {
            for user_id in subscribers.iter() {
                if let Err(e) = self.send_to_user(*user_id, message.clone()).await {
                    warn!("Failed to send message to user {}: {}", user_id, e);
                }
            }
        }
    }

    /// 发送消息给指定用户
    pub async fn send_to_user(&self, user_id: Uuid, message: String) -> anyhow::Result<()> {
        if let Some(sender) = self.connections.get(&user_id) {
            sender
                .send(message)
                .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User {} is not connected", user_id))
        }
    }

    /// 获取房间在线用户数量
    pub fn get_room_user_count(&self, room_id: Uuid) -> usize {
        self.room_subscribers
            .get(&room_id)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// 获取总连接数
    pub fn get_total_connections(&self) -> usize {
        self.connections.len()
    }

    /// 检查用户是否在线
    pub fn is_user_online(&self, user_id: Uuid) -> bool {
        self.connections.contains_key(&user_id)
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self {
            connections: DashMap::new(),
            room_subscribers: DashMap::new(),
        }
    }
}
