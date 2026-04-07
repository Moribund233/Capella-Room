use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::models::room::MemberRole;
use crate::redis::pubsub::RedisPubSub;

/// WebSocket 消息通道默认缓冲区大小
pub const DEFAULT_WS_MESSAGE_BUFFER_SIZE: usize = 100;

/// 用户连接信息
#[derive(Debug, Clone)]
pub struct UserConnection {
    pub user_id: Uuid,
    pub username: String,
    pub sender: mpsc::Sender<String>,
}

/// 房间成员信息
#[derive(Debug, Clone)]
pub struct RoomMemberInfo {
    pub user_id: Uuid,
    pub username: String,
    pub role: MemberRole,
}

/// WebSocket连接管理器
/// 管理所有活跃的WebSocket连接和房间订阅
/// 支持分布式部署，通过 Redis Pub/Sub 实现跨节点消息广播
#[derive(Debug)]
pub struct WebSocketManager {
    /// 用户ID到连接信息的映射
    connections: DashMap<Uuid, UserConnection>,
    /// 房间ID到用户ID集合的映射
    room_subscribers: DashMap<Uuid, Vec<Uuid>>,
    /// 用户当前加入的房间
    user_rooms: DashMap<Uuid, Vec<Uuid>>,
    /// 消息缓冲区大小（可动态更新）
    message_buffer_size: RwLock<usize>,
    /// 心跳间隔（秒，可动态更新）
    heartbeat_interval_secs: RwLock<u64>,
    /// 心跳超时（秒，可动态更新）
    heartbeat_timeout_secs: RwLock<u64>,
    /// Redis Pub/Sub 客户端（可选，用于分布式部署）
    redis_pubsub: RwLock<Option<RedisPubSub>>,
    /// 当前节点 ID（用于分布式部署）
    node_id: String,
}

impl WebSocketManager {
    /// 创建新的WebSocket管理器
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: DashMap::new(),
            room_subscribers: DashMap::new(),
            user_rooms: DashMap::new(),
            message_buffer_size: RwLock::new(DEFAULT_WS_MESSAGE_BUFFER_SIZE),
            heartbeat_interval_secs: RwLock::new(30),
            heartbeat_timeout_secs: RwLock::new(90),
            redis_pubsub: RwLock::new(None),
            node_id: format!("node-{}", Uuid::new_v4()),
        })
    }

    /// 从配置创建WebSocket管理器
    pub fn from_config(
        buffer_size: usize,
        heartbeat_interval: u64,
        heartbeat_timeout: u64,
    ) -> Arc<Self> {
        Arc::new(Self {
            connections: DashMap::new(),
            room_subscribers: DashMap::new(),
            user_rooms: DashMap::new(),
            message_buffer_size: RwLock::new(buffer_size),
            heartbeat_interval_secs: RwLock::new(heartbeat_interval),
            heartbeat_timeout_secs: RwLock::new(heartbeat_timeout),
            redis_pubsub: RwLock::new(None),
            node_id: format!("node-{}", Uuid::new_v4()),
        })
    }

    /// 获取当前节点 ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// 设置 Redis Pub/Sub 客户端
    ///
    /// # 参数
    /// - `redis_pubsub`: Redis Pub/Sub 客户端
    ///
    /// # 说明
    /// 启用 Redis 后，房间广播消息将通过 Redis 发布到其他节点
    pub async fn set_redis_pubsub(&self, redis_pubsub: RedisPubSub) {
        let mut pubsub = self.redis_pubsub.write().await;
        *pubsub = Some(redis_pubsub);
        debug!("Redis Pub/Sub client set for node {}", self.node_id);
    }

    /// 检查是否启用了 Redis
    pub async fn is_redis_enabled(&self) -> bool {
        let pubsub = self.redis_pubsub.read().await;
        pubsub.is_some()
    }

    /// 获取当前消息缓冲区大小
    pub async fn get_message_buffer_size(&self) -> usize {
        *self.message_buffer_size.read().await
    }

    /// 设置消息缓冲区大小（对新连接生效）
    pub async fn set_message_buffer_size(&self, size: usize) {
        let mut buffer_size = self.message_buffer_size.write().await;
        *buffer_size = size;
        debug!("WebSocket message buffer size updated to {}", size);
    }

    /// 获取当前心跳间隔（秒）
    pub async fn get_heartbeat_interval(&self) -> u64 {
        *self.heartbeat_interval_secs.read().await
    }

    /// 设置心跳间隔（秒，对新连接生效）
    pub async fn set_heartbeat_interval(&self, secs: u64) {
        let mut interval = self.heartbeat_interval_secs.write().await;
        *interval = secs;
        debug!("WebSocket heartbeat interval updated to {}s", secs);
    }

    /// 获取当前心跳超时（秒）
    pub async fn get_heartbeat_timeout(&self) -> u64 {
        *self.heartbeat_timeout_secs.read().await
    }

    /// 设置心跳超时（秒，对新连接生效）
    pub async fn set_heartbeat_timeout(&self, secs: u64) {
        let mut timeout = self.heartbeat_timeout_secs.write().await;
        *timeout = secs;
        debug!("WebSocket heartbeat timeout updated to {}s", secs);
    }

    /// 注册新连接
    pub fn connect(&self, user_id: Uuid, username: String, sender: mpsc::Sender<String>) {
        debug!("User {} ({}) connected to WebSocket", username, user_id);
        let connection = UserConnection {
            user_id,
            username: username.clone(),
            sender,
        };
        self.connections.insert(user_id, connection);
        self.user_rooms.insert(user_id, Vec::new());

        // 更新连接指标
        self.update_connection_metrics();
    }

    /// 断开连接
    pub fn disconnect(&self, user_id: Uuid) {
        debug!("User {} disconnected from WebSocket", user_id);

        // 获取用户加入的所有房间
        let rooms: Vec<Uuid> = self
            .user_rooms
            .get(&user_id)
            .map(|r| r.clone())
            .unwrap_or_default();

        // 从所有房间中移除该用户
        for room_id in rooms {
            self.leave_room(room_id, user_id);
        }

        // 移除连接
        self.connections.remove(&user_id);
        self.user_rooms.remove(&user_id);

        // 更新连接指标
        self.update_connection_metrics();
    }

    /// 更新连接指标
    fn update_connection_metrics(&self) {
        let total_connections = self.connections.len();
        debug!("Total active WebSocket connections: {}", total_connections);
        // 这里可以集成具体的指标收集器，如 Prometheus、OpenTelemetry 等
        // metrics::gauge!("websocket.connections", total_connections as f64);
    }

    /// 获取通道缓冲区使用率（用于监控背压）
    pub fn get_channel_buffer_usage(&self, _user_id: Uuid) -> Option<f64> {
        // 注意：tokio mpsc::Sender 没有直接的 capacity() 方法
        // 实际应用中可以添加专门的指标收集
        None
    }

    /// 加入房间
    pub fn join_room(&self, room_id: Uuid, user_id: Uuid) {
        debug!("User {} joined room {}", user_id, room_id);

        // 添加到房间订阅列表
        self.room_subscribers
            .entry(room_id)
            .or_default()
            .push(user_id);

        // 添加到用户的房间列表
        self.user_rooms.entry(user_id).or_default().push(room_id);
    }

    /// 离开房间
    pub fn leave_room(&self, room_id: Uuid, user_id: Uuid) {
        debug!("User {} left room {}", user_id, room_id);

        // 从房间订阅列表中移除
        if let Some(mut subscribers) = self.room_subscribers.get_mut(&room_id) {
            subscribers.retain(|&id| id != user_id);
        }

        // 从用户的房间列表中移除
        if let Some(mut rooms) = self.user_rooms.get_mut(&user_id) {
            rooms.retain(|&id| id != room_id);
        }
    }

    /// 广播消息到房间（排除指定用户）
    ///
    /// # 说明
    /// 如果启用了 Redis，消息将同时发布到 Redis，以便其他节点接收
    pub async fn broadcast_to_room(
        &self,
        room_id: Uuid,
        message: String,
        exclude_user: Option<Uuid>,
    ) {
        // 1. 本地广播
        self.broadcast_local(room_id, message.clone(), exclude_user)
            .await;

        // 2. 如果启用了 Redis，发布到 Redis
        if let Some(ref mut redis_pubsub) = *self.redis_pubsub.write().await {
            if let Err(e) = redis_pubsub
                .publish_room_message(room_id, message, exclude_user)
                .await
            {
                warn!("Failed to publish room message to Redis: {}", e);
            }
        }
    }

    /// 本地广播消息到房间（仅当前节点）
    ///
    /// # 参数
    /// - `room_id`: 房间 ID
    /// - `message`: 消息内容
    /// - `exclude_user`: 排除的用户 ID
    ///
    /// # 说明
    /// 该方法仅向当前节点的客户端发送消息，不通过 Redis 发布
    /// 用于处理从 Redis 接收到的消息
    pub async fn broadcast_local(
        &self,
        room_id: Uuid,
        message: String,
        exclude_user: Option<Uuid>,
    ) {
        if let Some(subscribers) = self.room_subscribers.get(&room_id) {
            for user_id in subscribers.iter() {
                // 跳过被排除的用户
                if let Some(exclude) = exclude_user {
                    if *user_id == exclude {
                        continue;
                    }
                }

                if let Err(e) = self.send_to_user(*user_id, message.clone()).await {
                    warn!("Failed to send message to user {}: {}", user_id, e);
                }
            }
        }
    }

    /// 广播消息到房间所有用户
    pub async fn broadcast_to_room_all(&self, room_id: Uuid, message: String) {
        self.broadcast_to_room(room_id, message, None).await;
    }

    /// 发送消息给指定用户
    /// 使用带超时的发送实现背压机制
    /// 如果通道已满（缓冲区 100 条消息），等待最多 1 秒
    /// 如果超时，返回错误并记录警告
    pub async fn send_to_user(&self, user_id: Uuid, message: String) -> anyhow::Result<()> {
        if let Some(connection) = self.connections.get(&user_id) {
            // 使用 tokio::time::timeout 实现发送超时
            // 防止因客户端处理慢导致服务端阻塞
            match tokio::time::timeout(
                std::time::Duration::from_secs(1),
                connection.sender.send(message),
            )
            .await
            {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => {
                    // 通道关闭，用户已断开
                    Err(anyhow::anyhow!("User {} connection closed: {}", user_id, e))
                }
                Err(_) => {
                    // 发送超时，触发背压
                    warn!("Backpressure: Failed to send message to user {} within timeout (channel full)", user_id);
                    Err(anyhow::anyhow!("Message send timeout (backpressure)"))
                }
            }
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

    /// 获取房间在线用户列表
    pub fn get_room_users(&self, room_id: Uuid) -> Vec<(Uuid, String)> {
        self.room_subscribers
            .get(&room_id)
            .map(|subscribers| {
                subscribers
                    .iter()
                    .filter_map(|user_id| {
                        self.connections
                            .get(user_id)
                            .map(|conn| (conn.user_id, conn.username.clone()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取总连接数
    pub fn get_total_connections(&self) -> usize {
        self.connections.len()
    }

    /// 获取在线用户数（与总连接数相同，因为一个用户一个连接）
    pub fn get_online_user_count(&self) -> usize {
        self.connections.len()
    }

    /// 获取活跃连接数（与总连接数相同）
    pub fn get_connection_count(&self) -> usize {
        self.connections.len()
    }

    /// 检查用户是否在线
    pub fn is_user_online(&self, user_id: Uuid) -> bool {
        self.connections.contains_key(&user_id)
    }

    /// 检查用户是否已连接（与 is_user_online 相同，用于重连检查）
    pub fn is_user_connected(&self, user_id: Uuid) -> bool {
        self.connections.contains_key(&user_id)
    }

    /// 获取用户连接信息
    pub fn get_user_connection(&self, user_id: Uuid) -> Option<UserConnection> {
        self.connections.get(&user_id).map(|c| c.clone())
    }

    /// 获取用户当前加入的房间
    pub fn get_user_rooms(&self, user_id: Uuid) -> Vec<Uuid> {
        self.user_rooms
            .get(&user_id)
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// 检查用户是否在房间中
    pub fn is_user_in_room(&self, room_id: Uuid, user_id: Uuid) -> bool {
        self.room_subscribers
            .get(&room_id)
            .map(|subscribers| subscribers.contains(&user_id))
            .unwrap_or(false)
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self {
            connections: DashMap::new(),
            room_subscribers: DashMap::new(),
            user_rooms: DashMap::new(),
            message_buffer_size: RwLock::new(DEFAULT_WS_MESSAGE_BUFFER_SIZE),
            heartbeat_interval_secs: RwLock::new(30),
            heartbeat_timeout_secs: RwLock::new(90),
            redis_pubsub: RwLock::new(None),
            node_id: format!("node-{}", Uuid::new_v4()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_manager_new() {
        let manager = WebSocketManager::new();
        assert_eq!(manager.get_total_connections(), 0);
    }

    #[test]
    fn test_connect_disconnect() {
        let manager = WebSocketManager::new();
        let user_id = Uuid::new_v4();
        let (tx, _rx) = mpsc::channel::<String>(100);

        manager.connect(user_id, "test_user".to_string(), tx);
        assert_eq!(manager.get_total_connections(), 1);
        assert!(manager.is_user_online(user_id));

        manager.disconnect(user_id);
        assert_eq!(manager.get_total_connections(), 0);
        assert!(!manager.is_user_online(user_id));
    }

    #[test]
    fn test_join_leave_room() {
        let manager = WebSocketManager::new();
        let user_id = Uuid::new_v4();
        let room_id = Uuid::new_v4();
        let (tx, _rx) = mpsc::channel::<String>(100);

        manager.connect(user_id, "test_user".to_string(), tx);
        manager.join_room(room_id, user_id);

        assert_eq!(manager.get_room_user_count(room_id), 1);
        assert!(manager.is_user_in_room(room_id, user_id));

        manager.leave_room(room_id, user_id);
        assert_eq!(manager.get_room_user_count(room_id), 0);
        assert!(!manager.is_user_in_room(room_id, user_id));
    }
}
