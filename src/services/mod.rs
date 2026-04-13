pub mod alert_engine;
pub mod alert_handler;
pub mod audit_service;
pub mod auth_service;
pub mod file_service;
pub mod ip_security_service;
pub mod message_service;
pub mod notification_service;
pub mod room_service;
pub mod user_service;

// 业务逻辑服务层
// - auth_service: 认证相关业务逻辑（注册、登录、Token管理）
// - user_service: 用户相关业务逻辑（用户信息、状态管理）
// - room_service: 聊天室相关业务逻辑（创建、加入、成员管理）
// - message_service: 消息相关业务逻辑（发送、查询、搜索）
// - file_service: 文件上传和资源管理业务逻辑
// - notification_service: 消息通知系统（私信、@提及、房间邀请、系统通知、文件上传通知）
// - audit_service: 安全审计系统（审计日志、告警管理、合规审计）
// - ip_security_service: IP 白名单/黑名单安全系统
//
// 服务层负责：
// - 处理复杂的业务逻辑
// - 协调多个数据访问操作
// - 事务管理
