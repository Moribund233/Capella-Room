pub mod auth_service;
pub mod message_service;
pub mod room_service;
pub mod user_service;

// TODO: 业务逻辑服务层
// - auth_service: 认证相关业务逻辑
// - user_service: 用户相关业务逻辑
// - room_service: 聊天室相关业务逻辑
// - message_service: 消息相关业务逻辑
// 
// 服务层负责：
// - 处理复杂的业务逻辑
// - 协调多个数据访问操作
// - 事务管理
// - 缓存操作
