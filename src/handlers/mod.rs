pub mod account_security;
pub mod admin;
pub mod audit;
pub mod auth;
pub mod auth_v2;
pub mod config;
pub mod custom_event;
pub mod file;
pub mod message;
pub mod message_reaction;
pub mod notification;
pub mod oauth;
pub mod pin_message;
pub mod room;
pub mod security;
pub mod ui_config;
pub mod user;
pub mod user_settings;
pub mod webhook;

// HTTP请求处理器模块
// - admin: 管理员处理器（用户管理、系统配置管理）
// - audit: 审计系统处理器（审计日志查询、告警管理、日志导出）
// - auth: 认证相关处理器（注册、登录、刷新Token）
// - config: 配置处理器（客户端配置获取）
// - user: 用户相关处理器（获取用户信息、更新资料、用户列表）
// - room: 聊天室处理器（创建、加入、离开、获取列表）
// - message: 消息处理器（获取历史消息、搜索消息、删除消息）
// - notification: 通知处理器（获取通知列表、标记已读）
// - file: 文件上传处理器（上传、获取、删除文件）
// - security: IP 安全处理器（白名单/黑名单管理）
// - ui_config: 用户 UI 配置处理器（界面偏好设置云端同步）
