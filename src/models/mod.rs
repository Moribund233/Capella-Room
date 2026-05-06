pub mod audit;
pub mod file;
pub mod message;
pub mod response;
pub mod room;
pub mod security;
pub mod ui_config;
pub mod user;
pub mod user_settings;

// 数据模型模块
// - response: 统一响应结构（ApiResponse、PaginatedData 等）
// - user: 用户模型（User、UserStatus、RegisterRequest、LoginRequest 等）
// - room: 聊天室模型（Room、RoomMember、MemberRole 等）
// - message: 消息模型（Message、MessageType、SendMessageRequest 等）
// - file: 文件资源模型（FileResource、FileCategory、FileUploadRequest 等）
// - audit: 审计日志模型（AuditLog、AuditAlert、AlertRule 等）
// - security: 安全模型（IpListEntry、IpListType、IpCheckResult 等）
// - ui_config: 用户 UI 配置模型（UserUIConfig、UIConfigResponse 等）
