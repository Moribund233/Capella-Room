use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{
    message::{CreateMessageRequest, Message},
    room::{CreateRoomRequest, Room, UpdateRoomRequest},
    user::{CreateUserRequest, UpdateUserRequest, User},
};
use crate::error::Result;

/// 用户服务 trait
#[async_trait]
pub trait UserService: Send + Sync {
    /// 创建用户
    async fn create_user(&self, req: CreateUserRequest) -> Result<User>;
    
    /// 根据ID获取用户
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>>;
    
    /// 根据邮箱获取用户
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    
    /// 更新用户信息
    async fn update_user(&self, user_id: Uuid, req: UpdateUserRequest) -> Result<User>;
    
    /// 更新用户状态
    async fn update_user_status(&self, user_id: Uuid, status: &str) -> Result<()>;
    
    /// 获取用户列表
    async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>>;
}

/// 聊天室服务 trait
#[async_trait]
pub trait RoomService: Send + Sync {
    /// 创建聊天室
    async fn create_room(&self, creator_id: Uuid, req: CreateRoomRequest) -> Result<Room>;
    
    /// 获取聊天室详情
    async fn get_room_by_id(&self, room_id: Uuid) -> Result<Option<Room>>;
    
    /// 获取聊天室列表
    async fn list_rooms(&self, limit: i64, offset: i64) -> Result<Vec<Room>>;
    
    /// 更新聊天室
    async fn update_room(&self, room_id: Uuid, req: UpdateRoomRequest) -> Result<Room>;
    
    /// 删除聊天室
    async fn delete_room(&self, room_id: Uuid) -> Result<()>;
    
    /// 用户加入聊天室
    async fn join_room(&self, room_id: Uuid, user_id: Uuid) -> Result<()>;
    
    /// 用户离开聊天室
    async fn leave_room(&self, room_id: Uuid, user_id: Uuid) -> Result<()>;
    
    /// 获取聊天室成员
    async fn get_room_members(&self, room_id: Uuid) -> Result<Vec<User>>;
}

/// 消息服务 trait
#[async_trait]
pub trait MessageService: Send + Sync {
    /// 创建消息
    async fn create_message(&self, sender_id: Uuid, req: CreateMessageRequest) -> Result<Message>;
    
    /// 根据ID获取消息
    async fn get_message_by_id(&self, message_id: Uuid) -> Result<Option<Message>>;
    
    /// 获取聊天室消息列表
    async fn get_room_messages(&self, room_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Message>>;
    
    /// 搜索消息
    async fn search_messages(&self, query: &str, room_id: Option<Uuid>, limit: i64) -> Result<Vec<Message>>;
    
    /// 删除消息
    async fn delete_message(&self, message_id: Uuid, user_id: Uuid) -> Result<()>;
}

/// 认证服务 trait
#[async_trait]
pub trait AuthService: Send + Sync {
    /// 用户注册
    async fn register(&self, username: &str, email: &str, password: &str) -> Result<User>;
    
    /// 用户登录
    async fn login(&self, email: &str, password: &str) -> Result<(User, String)>;
    
    /// 验证Token
    async fn verify_token(&self, token: &str) -> Result<Uuid>;
    
    /// 刷新Token
    async fn refresh_token(&self, token: &str) -> Result<String>;
}
