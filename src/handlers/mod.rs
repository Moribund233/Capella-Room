pub mod admin;
pub mod auth;
pub mod file;
pub mod message;
pub mod room;
pub mod user;

// HTTP请求处理器模块
// - admin: 管理员处理器（用户管理、系统配置管理）
// - auth: 认证相关处理器（注册、登录、刷新Token）
// - user: 用户相关处理器（获取用户信息、更新资料、用户列表）
// - room: 聊天室处理器（创建、加入、离开、获取列表）
// - message: 消息处理器（获取历史消息、搜索消息、删除消息）
// - file: 文件上传处理器（上传、获取、删除文件）
