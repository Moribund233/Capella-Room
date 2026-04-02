use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::models::room::MemberRole;

/// 用户连接信息
#[derive(Debug, Clone)]
pub struct UserConnection {
    pub user_id: Uuid,
    pub username: String,
    pub sender: mpsc::UnboundedSender<String>,
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
#[derive(Debug)]
pub struct WebSocketManager {
    /// 用户ID到连接信息的映射
    connections: DashMap<Uuid, UserConnection>,
    /// 房间ID到用户ID集合的映射
    room_subscribers: DashMap<Uuid, Vec<Uuid>>,
    /// 用户当前加入的房间
    user_rooms: DashMap<Uuid, Vec<Uuid>>,
}

impl WebSocketManager {
    /// 创建新的WebSocket管理器
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: DashMap::new(),
            room_subscribers: DashMap::new(),
            user_rooms: DashMap::new(),
        })
    }

    /// 注册新连接
    pub fn connect(&self, user_id: Uuid, username: String, sender: mpsc::UnboundedSender<String>) {
        debug!("User {} ({}) connected to WebSocket", username, user_id);
        let connection = UserConnection {
            user_id,
            username: username.clone(),
            sender,
        };
        self.connections.insert(user_id, connection);
        self.user_rooms.insert(user_id, Vec::new());
    }

    /// 断开连接
    pub fn disconnect(&self, user_id: Uuid) {
        debug!("User {} disconnected from WebSocket", user_id);
        
        // 获取用户加入的所有房间
        let rooms: Vec<Uuid> = self.user_rooms
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
        self.user_rooms
            .entry(user_id)
            .or_default()
            .push(room_id);
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
    pub async fn broadcast_to_room(&self, room_id: Uuid, message: String, exclude_user: Option<Uuid>) {
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
    pub async fn send_to_user(&self, user_id: Uuid, message: String) -> anyhow::Result<()> {
        if let Some(connection) = self.connections.get(&user_id) {
            connection
                .sender
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

    /// 获取房间在线用户列表
    pub fn get_room_users(&self, room_id: Uuid) -> Vec<(Uuid, String)> {
        self.room_subscribers
            .get(&room_id)
            .map(|subscribers| {
                subscribers
                    .iter()
                    .filter_map(|user_id| {
                        self.connections.get(user_id).map(|conn| {
                            (conn.user_id, conn.username.clone())
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取总连接数
    pub fn get_total_connections(&self) -> usize {
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
        let (tx, _rx) = mpsc::unbounded_channel();

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
        let (tx, _rx) = mpsc::unbounded_channel();

        manager.connect(user_id, "test_user".to_string(), tx);
        manager.join_room(room_id, user_id);

        assert_eq!(manager.get_room_user_count(room_id), 1);
        assert!(manager.is_user_in_room(room_id, user_id));

        manager.leave_room(room_id, user_id);
        assert_eq!(manager.get_room_user_count(room_id), 0);
        assert!(!manager.is_user_in_room(room_id, user_id));
    }
}
