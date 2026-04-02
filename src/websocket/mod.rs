pub mod handler;
pub mod manager;
pub mod protocol;

// WebSocket模块
// - manager: WebSocket连接管理器（管理在线用户、房间订阅）
// - handler: WebSocket连接处理器（消息收发、心跳检测）
// - protocol: WebSocket消息协议定义
