pub mod auth_service;
pub mod message_service;
pub mod room_service;
pub mod user_service;

// 业务逻辑服务层
// - auth_service: 认证相关业务逻辑（注册、登录、Token管理）
// - user_service: 用户相关业务逻辑（用户信息、状态管理）
// - room_service: 聊天室相关业务逻辑（创建、加入、成员管理）
// - message_service: 消息相关业务逻辑（发送、查询、搜索）
//
// 服务层负责：
// - 处理复杂的业务逻辑
// - 协调多个数据访问操作
// - 事务管理
