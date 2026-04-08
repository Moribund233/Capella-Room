use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use tracing::debug;

use crate::models::audit::{AuditEventType, AuditMetadata, AuditSeverity, CreateAuditLogRequest};
use crate::services::audit_service::AuditService;
use crate::services::auth_service::Claims;
use crate::state::AppState;

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

        // 确定事件类型
        let event_type = self.determine_event_type(&path, &method);

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

    /// 根据请求路径和方法确定事件类型
    fn determine_event_type(&self, path: &str, method: &Method) -> AuditEventType {
        // 认证相关
        if path.contains("/auth/login") {
            return AuditEventType::UserLogin;
        }
        if path.contains("/auth/register") {
            return AuditEventType::UserRegister;
        }

        // 用户相关
        if path.contains("/users") && (method == Method::PUT || method == Method::PATCH) {
            return AuditEventType::UserProfileUpdate;
        }

        // 房间相关
        if path.contains("/rooms") {
            if method == Method::POST {
                return AuditEventType::RoomCreate;
            }
            if method == Method::DELETE {
                return AuditEventType::RoomDelete;
            }
            if path.contains("/join") && method == Method::POST {
                return AuditEventType::RoomMemberAdd;
            }
            if path.contains("/leave") && method == Method::DELETE {
                return AuditEventType::RoomMemberRemove;
            }
            if path.contains("/role") && method == Method::PUT {
                return AuditEventType::RoomMemberRoleChange;
            }
        }

        // 消息相关
        if path.contains("/messages") {
            if method == Method::POST {
                return AuditEventType::MessageSend;
            }
            if method == Method::PUT || method == Method::PATCH {
                return AuditEventType::MessageEdit;
            }
            if method == Method::DELETE {
                return AuditEventType::MessageDelete;
            }
        }

        // 管理员操作
        if path.contains("/admin") {
            if path.contains("/users") {
                if method == Method::DELETE {
                    return AuditEventType::AdminUserDelete;
                }
                if path.contains("/role") && method == Method::PUT {
                    return AuditEventType::AdminUserRoleChange;
                }
                if path.contains("/status") && method == Method::PUT {
                    return AuditEventType::AdminUserDisable;
                }
            }
            if path.contains("/rooms") && method == Method::DELETE {
                return AuditEventType::AdminRoomDelete;
            }
            if path.contains("/messages") && method == Method::DELETE {
                return AuditEventType::AdminMessageDelete;
            }
            if path.contains("/configs") && method == Method::PUT {
                return AuditEventType::AdminConfigUpdate;
            }
        }

        // 审计系统相关操作
        if path.contains("/admin/audit") {
            if path.contains("/export") {
                return AuditEventType::AuditExport;
            }
            if path.contains("/stats") {
                return AuditEventType::AuditStatsQuery;
            }
            if path.contains("/alerts") {
                if method == Method::PUT {
                    return AuditEventType::AlertRuleUpdate;
                }
                return AuditEventType::AlertQuery;
            }
            if path.contains("/rules") && method == Method::PUT {
                return AuditEventType::AlertRuleUpdate;
            }
            if path.contains("/cleanup") && method == Method::POST {
                return AuditEventType::AuditCleanup;
            }
            // 审计日志查询
            if path.contains("/logs") || path.contains("/admin/audit") {
                return AuditEventType::AuditQuery;
            }
        }

        // 默认返回系统事件
        AuditEventType::SystemUnauthorizedAccess
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
    }
}
