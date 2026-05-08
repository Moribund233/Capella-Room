use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use tracing::debug;

use crate::models::audit::{AuditEventType, AuditMetadata, AuditSeverity, CreateAuditLogRequest};
use crate::models::user::UserRole;
use crate::services::audit_service::AuditService;
use crate::services::auth_service::Claims;
use crate::state::AppState;

/// 路由模式定义
/// 用于匹配请求路径和确定事件类型
#[derive(Debug, Clone, Copy)]
enum RoutePattern {
    /// 认证相关
    AuthLogin,
    AuthRegister,
    /// 用户相关
    UserProfileUpdate,
    UserMe,
    /// 房间相关
    RoomCreate,
    RoomDelete,
    RoomMemberAdd,
    RoomMemberRemove,
    RoomMemberRoleChange,
    /// 消息相关
    MessageSend,
    MessageEdit,
    MessageDelete,
    /// 通知相关
    NotificationQuery,
    /// 管理员 - 用户管理
    AdminUserDelete,
    AdminUserRoleChange,
    AdminUserDisable,
    AdminUserQuery,
    /// 管理员 - 房间管理
    AdminRoomDelete,
    AdminRoomQuery,
    /// 管理员 - 消息管理
    AdminMessageDelete,
    AdminMessageQuery,
    /// 管理员 - 配置管理
    AdminConfigUpdate,
    AdminConfigQuery,
    /// 管理员 - 审计系统
    AdminAuditQuery,
    AdminAuditExport,
    AdminAuditStats,
    AdminAlertQuery,
    AdminAlertRuleUpdate,
    AdminAuditCleanup,
    /// 管理员 - 统计
    AdminStatsQuery,
    /// 管理员 - IP安全
    AdminIpSecurity,
    /// 管理员 - 其他
    AdminOtherQuery,
    /// 未匹配
    Unknown,
}

impl RoutePattern {
    /// 根据路径和方法匹配路由模式
    fn match_route(path: &str, method: &Method) -> Self {
        use RoutePattern::*;

        // 使用元组数组定义路由规则，按优先级排序
        // (路径包含, 方法匹配, 路由模式)
        let rules: &[(&str, Option<Method>, RoutePattern)] = &[
            // 认证
            ("/auth/login", None, AuthLogin),
            ("/auth/register", None, AuthRegister),
            // 用户
            ("/users/me", Some(Method::GET), UserMe),
            ("/users", Some(Method::PUT), UserProfileUpdate),
            ("/users", Some(Method::PATCH), UserProfileUpdate),
            // 房间
            ("/rooms", Some(Method::POST), RoomCreate),
            ("/rooms", Some(Method::DELETE), RoomDelete),
            ("/rooms/join", Some(Method::POST), RoomMemberAdd),
            ("/rooms/leave", Some(Method::DELETE), RoomMemberRemove),
            ("/rooms/role", Some(Method::PUT), RoomMemberRoleChange),
            // 消息
            ("/messages", Some(Method::POST), MessageSend),
            ("/messages", Some(Method::PUT), MessageEdit),
            ("/messages", Some(Method::PATCH), MessageEdit),
            ("/messages", Some(Method::DELETE), MessageDelete),
            // 通知
            ("/notifications", None, NotificationQuery),
            // 管理员 - 审计系统（优先匹配）
            ("/admin/audit/export", None, AdminAuditExport),
            (
                "/admin/audit/cleanup",
                Some(Method::POST),
                AdminAuditCleanup,
            ),
            ("/admin/audit/stats", None, AdminAuditStats),
            (
                "/admin/audit/alerts",
                Some(Method::PUT),
                AdminAlertRuleUpdate,
            ),
            (
                "/admin/audit/alerts",
                Some(Method::POST),
                AdminAlertRuleUpdate,
            ),
            (
                "/admin/audit/rules",
                Some(Method::PUT),
                AdminAlertRuleUpdate,
            ),
            (
                "/admin/audit/rules",
                Some(Method::POST),
                AdminAlertRuleUpdate,
            ),
            ("/admin/audit/alerts", None, AdminAlertQuery),
            ("/admin/audit", None, AdminAuditQuery),
            // 管理员 - 统计
            ("/admin/stats", None, AdminStatsQuery),
            // 管理员 - IP安全
            ("/admin/ip", None, AdminIpSecurity),
            ("/admin/security/ip", None, AdminIpSecurity),
            // 管理员 - 配置
            ("/admin/configs", Some(Method::PUT), AdminConfigUpdate),
            ("/admin/configs", Some(Method::PATCH), AdminConfigUpdate),
            ("/admin/configs", Some(Method::POST), AdminConfigUpdate),
            ("/admin/configs", None, AdminConfigQuery),
            // 管理员 - 用户
            ("/admin/users", Some(Method::DELETE), AdminUserDelete),
            ("/admin/users/role", Some(Method::PUT), AdminUserRoleChange),
            (
                "/admin/users/role",
                Some(Method::PATCH),
                AdminUserRoleChange,
            ),
            ("/admin/users/status", Some(Method::PUT), AdminUserDisable),
            ("/admin/users/status", Some(Method::PATCH), AdminUserDisable),
            ("/admin/users", Some(Method::GET), AdminUserQuery),
            // 管理员 - 房间
            ("/admin/rooms", Some(Method::DELETE), AdminRoomDelete),
            ("/admin/rooms", Some(Method::GET), AdminRoomQuery),
            // 管理员 - 消息
            ("/admin/messages", Some(Method::DELETE), AdminMessageDelete),
            ("/admin/messages", Some(Method::GET), AdminMessageQuery),
            // 管理员 - 其他GET请求
            ("/admin", Some(Method::GET), AdminOtherQuery),
        ];

        for (pattern, method_check, route) in rules {
            if path.contains(pattern) {
                // 检查方法是否匹配（如果指定了方法）
                if let Some(ref expected_method) = *method_check {
                    if method != expected_method {
                        continue;
                    }
                }
                return *route;
            }
        }

        Unknown
    }

    /// 将路由模式转换为审计事件类型
    /// 考虑用户角色来判断是否为未授权访问
    fn to_event_type(self, user_role: Option<&UserRole>) -> AuditEventType {
        // 检查是否为管理员路由
        let is_admin_route = matches!(
            self,
            RoutePattern::AdminUserDelete
                | RoutePattern::AdminUserRoleChange
                | RoutePattern::AdminUserDisable
                | RoutePattern::AdminUserQuery
                | RoutePattern::AdminRoomDelete
                | RoutePattern::AdminRoomQuery
                | RoutePattern::AdminMessageDelete
                | RoutePattern::AdminMessageQuery
                | RoutePattern::AdminConfigUpdate
                | RoutePattern::AdminConfigQuery
                | RoutePattern::AdminAuditQuery
                | RoutePattern::AdminAuditExport
                | RoutePattern::AdminAuditStats
                | RoutePattern::AdminAlertQuery
                | RoutePattern::AdminAlertRuleUpdate
                | RoutePattern::AdminAuditCleanup
                | RoutePattern::AdminStatsQuery
                | RoutePattern::AdminIpSecurity
                | RoutePattern::AdminOtherQuery
        );

        // 如果是管理员路由，检查用户权限
        if is_admin_route {
            match user_role {
                Some(UserRole::SuperAdmin) => {
                    // 超级管理员可以访问所有端点
                    self.admin_to_event_type()
                }
                Some(UserRole::Admin) => {
                    // 普通管理员不能访问某些敏感操作
                    match self {
                        RoutePattern::AdminUserDelete | RoutePattern::AdminUserRoleChange => {
                            // 普通管理员不能删除用户或修改角色，记录为未授权尝试
                            AuditEventType::SystemUnauthorizedAccess
                        }
                        _ => self.admin_to_event_type(),
                    }
                }
                _ => {
                    // 非管理员访问管理员端点，记录为未授权访问
                    AuditEventType::SystemUnauthorizedAccess
                }
            }
        } else {
            // 非管理员路由，直接转换
            self.to_event_type_direct()
        }
    }

    /// 管理员路由转换为事件类型
    fn admin_to_event_type(self) -> AuditEventType {
        match self {
            RoutePattern::AdminUserDelete => AuditEventType::AdminUserDelete,
            RoutePattern::AdminUserRoleChange => AuditEventType::AdminUserRoleChange,
            RoutePattern::AdminUserDisable => AuditEventType::AdminUserDisable,
            RoutePattern::AdminUserQuery => AuditEventType::AdminUserRoleChange,
            RoutePattern::AdminRoomDelete => AuditEventType::AdminRoomDelete,
            RoutePattern::AdminRoomQuery => AuditEventType::AdminRoomDelete,
            RoutePattern::AdminMessageDelete => AuditEventType::AdminMessageDelete,
            RoutePattern::AdminMessageQuery => AuditEventType::AdminMessageDelete,
            RoutePattern::AdminConfigUpdate => AuditEventType::AdminConfigUpdate,
            RoutePattern::AdminConfigQuery => AuditEventType::AdminConfigUpdate,
            RoutePattern::AdminAuditQuery => AuditEventType::AuditQuery,
            RoutePattern::AdminAuditExport => AuditEventType::AuditExport,
            RoutePattern::AdminAuditStats => AuditEventType::AuditStatsQuery,
            RoutePattern::AdminAlertQuery => AuditEventType::AlertQuery,
            RoutePattern::AdminAlertRuleUpdate => AuditEventType::AlertRuleUpdate,
            RoutePattern::AdminAuditCleanup => AuditEventType::AuditCleanup,
            RoutePattern::AdminStatsQuery => AuditEventType::AuditStatsQuery,
            RoutePattern::AdminIpSecurity => AuditEventType::AdminConfigUpdate,
            RoutePattern::AdminOtherQuery => AuditEventType::AuditQuery,
            _ => AuditEventType::SystemUnauthorizedAccess,
        }
    }

    /// 直接转换为事件类型（不考虑权限）
    fn to_event_type_direct(self) -> AuditEventType {
        match self {
            RoutePattern::AuthLogin => AuditEventType::UserLogin,
            RoutePattern::AuthRegister => AuditEventType::UserRegister,
            RoutePattern::UserProfileUpdate | RoutePattern::UserMe => {
                AuditEventType::UserProfileUpdate
            }
            RoutePattern::RoomCreate => AuditEventType::RoomCreate,
            RoutePattern::RoomDelete => AuditEventType::RoomDelete,
            RoutePattern::RoomMemberAdd => AuditEventType::RoomMemberAdd,
            RoutePattern::RoomMemberRemove => AuditEventType::RoomMemberRemove,
            RoutePattern::RoomMemberRoleChange => AuditEventType::RoomMemberRoleChange,
            RoutePattern::MessageSend => AuditEventType::MessageSend,
            RoutePattern::MessageEdit => AuditEventType::MessageEdit,
            RoutePattern::MessageDelete => AuditEventType::MessageDelete,
            RoutePattern::NotificationQuery => AuditEventType::UserProfileUpdate,
            RoutePattern::Unknown => AuditEventType::SystemUnauthorizedAccess,
            _ => AuditEventType::SystemUnauthorizedAccess,
        }
    }
}

/// 从路径中提取目标信息
/// 返回 (target_type, target_id)
fn extract_target_from_path(path: &str) -> (Option<String>, Option<uuid::Uuid>) {
    // 移除 API 前缀
    let path = path.strip_prefix("/api").unwrap_or(path);
    let path = path.strip_prefix("/v1").unwrap_or(path);

    // 分割路径
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if segments.is_empty() {
        return (None, None);
    }

    // 确定目标类型
    let target_type = match segments[0] {
        "rooms" => Some("room"),
        "users" => Some("user"),
        "messages" => Some("message"),
        "admin" => {
            // 处理 /admin/rooms/{id}, /admin/users/{id}, /admin/audit/* 等
            if segments.len() >= 2 {
                match segments[1] {
                    "rooms" => Some("room"),
                    "users" => Some("user"),
                    "messages" => Some("message"),
                    "audit" => Some("audit"),
                    "configs" => Some("config"),
                    _ => Some("admin"),
                }
            } else {
                Some("admin")
            }
        }
        "auth" => Some("auth"),
        "ws" => Some("websocket"),
        _ => None,
    };

    // 尝试提取 UUID
    let target_id = if segments[0] == "admin" && segments.len() >= 3 {
        // 对于 /admin/audit/* 路径，不提取 ID
        if segments[1] == "audit" {
            None
        } else {
            segments[2].parse::<uuid::Uuid>().ok()
        }
    } else if segments.len() >= 2 && segments[0] != "admin" {
        segments[1].parse::<uuid::Uuid>().ok()
    } else {
        None
    };

    (target_type.map(|s| s.to_string()), target_id)
}

/// 审计中间件
/// 透明记录 HTTP 请求的关键信息
#[derive(Clone)]
pub struct AuditMiddleware {
    #[allow(dead_code)]
    audit_service: Arc<AuditService>,
    excluded_paths: Vec<String>,
    sensitive_paths: Vec<String>,
}

impl AuditMiddleware {
    /// 创建新的审计中间件
    pub fn new(audit_service: Arc<AuditService>) -> Self {
        Self {
            audit_service,
            excluded_paths: vec![
                "/health".to_string(),
                "/health/detail".to_string(),
                "/health/ready".to_string(),
                "/health/live".to_string(),
                "/ws".to_string(),
                "/api/version".to_string(),
            ],
            sensitive_paths: vec!["/api/admin".to_string(), "/api/v1/admin".to_string()],
        }
    }

    /// 检查路径是否应该被排除
    fn should_skip(&self, path: &str) -> bool {
        self.excluded_paths.iter().any(|excluded| path == excluded)
    }

    /// 检查是否为敏感操作
    fn is_sensitive(&self, path: &str, method: &Method) -> bool {
        // DELETE 操作都是敏感的
        if method == Method::DELETE {
            return true;
        }

        // 检查敏感路径
        self.sensitive_paths
            .iter()
            .any(|sensitive| path.starts_with(sensitive))
    }

    /// 处理请求
    pub async fn handle(
        &self,
        State(state): State<Arc<AppState>>,
        request: Request,
        next: Next,
    ) -> Response {
        let path = request.uri().path().to_string();
        let method = request.method().clone();

        // 检查是否需要跳过审计
        if self.should_skip(&path) {
            return next.run(request).await;
        }

        // 提取请求信息（可选获取 ConnectInfo）
        let ip = request
            .extensions()
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip())
            .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));

        let user_agent = request
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // 从请求扩展中提取用户信息
        let (user_id, user_role) = if let Some(claims) = request.extensions().get::<Claims>() {
            if let Ok(uid) = uuid::Uuid::parse_str(&claims.sub) {
                (Some(uid), Some(claims.role.clone()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // 记录请求开始时间
        let start_time = std::time::Instant::now();

        // 执行后续中间件
        let response = next.run(request).await;

        // 计算响应时间
        let _duration = start_time.elapsed();
        let status = response.status();

        // 确定事件类型（使用新的路由模式系统）
        let user_role_ref = user_role.as_ref();
        let event_type = RoutePattern::match_route(&path, &method).to_event_type(user_role_ref);

        // 构建审计日志
        let mut log = CreateAuditLogRequest::new(
            event_type.clone(),
            method.to_string().to_lowercase(),
            format!("{} {} - {}", method, path, status),
        );

        // 设置操作者信息
        if let (Some(uid), Some(role)) = (user_id, user_role) {
            log = log.with_actor(uid, role);
        }

        // 设置目标信息
        let (target_type, target_id) = extract_target_from_path(&path);
        if let Some(t_type) = target_type {
            if let Some(t_id) = target_id {
                log = log.with_target(t_type, t_id);
            } else {
                // 只有目标类型，没有具体目标ID
                log.target_type = Some(t_type);
            }
        }

        // 设置元数据
        let mut metadata =
            AuditMetadata::new()
                .with_ip(ip)
                .with_request(method.to_string(), path.clone(), None);

        if let Some(ua) = user_agent {
            metadata = metadata.with_user_agent(ua);
        }

        log = log.with_metadata(metadata);

        // 设置状态
        if status.is_success() {
            log.status = Some("success".to_string());
        } else {
            log.status = Some("failure".to_string());
            log.severity = Some(AuditSeverity::Warning);
        }

        // 敏感操作提升严重级别
        if self.is_sensitive(&path, &method) {
            log.severity = Some(AuditSeverity::Warning);
        }

        // 异步记录审计日志
        let audit_service = Arc::clone(&state.audit_service);
        tokio::spawn(async move {
            if let Err(e) = audit_service.log_event(log).await {
                debug!("Failed to log audit event: {}", e);
            }
        });

        response
    }
}

/// 审计中间件处理函数
/// 用于 Axum 的 from_fn_with_state
pub async fn audit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let middleware = AuditMiddleware::new(Arc::clone(&state.audit_service));
    middleware.handle(State(state), request, next).await
}

/// 提取客户端真实 IP
/// 优先从 X-Forwarded-For 头获取，否则使用连接地址
pub fn extract_client_ip(
    headers: &axum::http::HeaderMap,
    addr: std::net::IpAddr,
) -> std::net::IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|ip| ip.trim().parse().ok())
        .unwrap_or(addr)
}

/// 判断是否为敏感操作
pub fn is_sensitive_operation(method: &Method, path: &str) -> bool {
    // DELETE 操作
    if method == Method::DELETE {
        return true;
    }

    // 管理员操作
    if path.starts_with("/api/admin") || path.starts_with("/api/v1/admin") {
        return true;
    }

    // 批量操作
    if path.contains("/batch") {
        return true;
    }

    // 用户敏感操作
    if path.contains("/password") || path.contains("/security") {
        return true;
    }

    false
}

/// 获取审计事件类型描述
pub fn get_event_description(event_type: &AuditEventType) -> &'static str {
    match event_type {
        AuditEventType::UserLogin => "用户登录",
        AuditEventType::UserLogout => "用户登出",
        AuditEventType::UserRegister => "用户注册",
        AuditEventType::UserPasswordChange => "密码修改",
        AuditEventType::UserProfileUpdate => "资料更新",
        AuditEventType::UserFriendRequestSend => "发送好友申请",
        AuditEventType::UserFriendRequestAccept => "接受好友申请",
        AuditEventType::UserFriendRequestReject => "拒绝好友申请",
        AuditEventType::UserFriendRequestCancel => "取消好友申请",
        AuditEventType::UserFriendRemove => "删除好友",
        AuditEventType::RoomCreate => "创建房间",
        AuditEventType::RoomDelete => "删除房间",
        AuditEventType::RoomMemberAdd => "添加成员",
        AuditEventType::RoomMemberRemove => "移除成员",
        AuditEventType::RoomMemberRoleChange => "变更角色",
        AuditEventType::MessageSend => "发送消息",
        AuditEventType::MessageEdit => "编辑消息",
        AuditEventType::MessageDelete => "删除消息",
        AuditEventType::MessageReport => "举报消息",
        AuditEventType::AdminUserDisable => "禁用用户",
        AuditEventType::AdminUserRoleChange => "变更用户角色",
        AuditEventType::AdminUserDelete => "删除用户",
        AuditEventType::AdminRoomDelete => "删除房间",
        AuditEventType::AdminMessageDelete => "删除消息",
        AuditEventType::AdminConfigUpdate => "更新配置",
        AuditEventType::SystemLoginFailure => "登录失败",
        AuditEventType::SystemUnauthorizedAccess => "未授权访问",
        AuditEventType::SystemRateLimitTriggered => "触发限流",
        AuditEventType::AuditQuery => "审计查询",
        AuditEventType::AuditExport => "审计导出",
        AuditEventType::AuditStatsQuery => "审计统计查询",
        AuditEventType::AlertQuery => "告警查询",
        AuditEventType::AlertRuleUpdate => "告警规则更新",
        AuditEventType::AuditCleanup => "审计清理",
        AuditEventType::IpBlocked => "IP被阻止",
        AuditEventType::IpWhitelistDenied => "IP不在白名单",
        AuditEventType::IpRateLimited => "IP频率限制",
        AuditEventType::IpListAdded => "添加IP列表",
        AuditEventType::IpListRemoved => "移除IP列表",
        AuditEventType::IpListUpdated => "更新IP列表",
    }
}
