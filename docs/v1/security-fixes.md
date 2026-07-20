# 安全修复清单

> 本文档记录阶段 9 安全审计中发现的后端安全隐患及修复方案。
> 审计日期: 2026-07-03

---

## 0002-WebSocket 握手缺少 JWT 认证 [P1-高优先级] - 已修复

**位置**: `src/routes/mod.rs` — 公开路由 `/ws` 直接注册，无认证层

**问题描述**:
WebSocket 升级端点 `/ws` 注册在公开路由中，不需要任何认证即可建立 WebSocket 连接。攻击者可以直接连接 WS 并发送恶意消息。

**当前代码** (`src/routes/mod.rs:29-40`):
```rust
let public_routes = Router::new()
    .route("/health", get(health_check))
    .route("/health/detail", get(health_check_detailed))
    .route("/health/ready", get(readiness_check))
    .route("/health/live", get(liveness_check))
    .route("/api/version", get(api_version))
    .route("/api/config/client", get(config::get_client_config))
    .route("/ws", get(ws_handler));   // ← 无需认证
```

**验证结果**: `GET /ws` (Upgrade: websocket) → HTTP 101 升级成功，无需任何 Token。

**修复方案**:
将 `/ws` 移入 `protected_routes` 中，复用 `auth_middleware`。

```rust
// 方案 A：移入受保护路由
let protected_routes = Router::new()
    .route("/ws", get(ws_handler))  // 移到这里
    // ... 其他路由
    .layer(middleware::from_fn_with_state(
        Arc::clone(&state),
        auth_middleware,  // 自动套用 JWT 验证
    ));
```

**注意**: Rust Axum 的 WebSocket 升级在中间件之后执行。认证中间件会在 Upgrade 之前先验证 JWT，失败返回 401，不会进入 WS handler。

**影响文件**:
- `src/routes/mod.rs` — 将 `/ws` 从 public_routes 移到 protected_routes

**修复状态**: 已修复（2026-07-21）

**修复内容**:
将 `/ws` 路由从 `public_routes` 迁移至 `protected_routes`，使其经过 `auth_middleware` JWT 认证。Axum 的 WebSocket 升级在认证中间件之后执行，因此未携带有效 Token 的请求会在 Upgrade 之前返回 401，无法建立 WebSocket 连接。

---

## 0003-登录接口缺少速率限制 [P1-高优先级] - 已修复

**位置**: `src/routes/mod.rs` — `/api/auth/login` 无限流中间件

**问题描述**:
登录接口没有速率限制（Rate Limiting），攻击者可批量爆破用户名/密码。测试中 10 次连续请求全部正常响应（HTTP 401），无任何阻断。

**当前代码** (`src/routes/mod.rs:141-145`):
```rust
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh_token))
}
```

无 `rate_limit` 中间件包裹。

**修复方案**:

推荐使用 `tower_http::limit::RateLimitLayer` 或自定义中间件。

```rust
// 方案 A：tower_http 内置限流（推荐）
use tower_http::limit::{RateLimitLayer, Rate};
use std::time::Duration;

fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(auth::login))
        .layer(RateLimitLayer::new(
            5,  // 每个窗口期允许 5 次请求
            Duration::from_secs(60),  // 60 秒窗口
        ))
        .route("/refresh", post(auth::refresh_token))
}
```

```rust
// 方案 B：自定义中间件（更灵活，支持 IP 级别限流）
// src/middleware/rate_limit.rs
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let ip = req.extensions()
        .get::<SocketAddr>()
        .map(|addr| addr.ip())
        .unwrap_or_else(|| Ipv4Addr::new(0, 0, 0, 0).into());

    let key = format!("rate_limit:login:{}", ip);
    // 使用 Redis 或内存计数器检查频率
    // 如果超过阈值，返回 429 Too Many Requests
}
```

**建议参数**:
- 窗口: 60 秒
- 阈值: 5 次/窗口（普通用户）或 20 次/窗口（宽松）
- 响应: HTTP 429 + `Retry-After` Header
- 注意: 服务目前有 Redis 连接（`RedisManager`），可直接复用做分布式限流

**影响文件**:
- `src/routes/mod.rs` — 添加 RateLimitLayer
- `src/middleware/mod.rs` — 新增 `rate_limit.rs`（如使用方案 B）
- `src/state/mod.rs` — 可能需要暴露 Redis manager

**修复状态**: 已修复（2026-07-21）

**修复内容**:
新增自定义基于客户端 IP 的内存限流中间件 `src/middleware/rate_limit.rs`，对 `/api/auth/login` 实施限流。默认配置为每个客户端 IP 在每个 60 秒窗口内最多允许 5 次登录请求，超过阈值返回 HTTP 429 并携带 `Retry-After` 响应头。限流参数接入配置系统，支持通过 `config.toml` 的 `[server.login_rate_limit]`、环境变量 `SERVER_LOGIN_RATE_LIMIT_*` 以及数据库 `system_configs` 热重载动态调整。

---

## 0004-Webhook URL 未校验内网地址（SSRF） [P2-中优先级] - 已修复

**位置**: `src/services/webhook_service.rs` — `deliver_once()` 函数

**问题描述**:
Webhook 投递时直接将用户提供的 URL 传给 `reqwest::Client::post(url)`，未做任何 IP/域名限制。已认证的 OAuth 应用用户可以创建指向内网地址的 Webhook，后端会向其发送 POST 请求（SSRF）。

**当前代码** (`src/services/webhook_service.rs:367-406`):
```rust
async fn deliver_once(
    http_client: &reqwest::Client,
    url: &str,           // ← 用户提供，无校验
    secret: &str,
    event_type: &str,
    _event_id: Uuid,
    payload: &serde_json::Value,
    timeout_ms: i32,
    attempt: i32,
) -> Result<(i32, String)> {
    // ...
    let response = http_client
        .post(url)       // ← 直接发往用户指定的 URL
        .header("Content-Type", "application/json")
        // ...
        .send()
        .await?;
    // ...
}
```

**修复方案**:

在投递前添加 URL 校验，禁止内网/私有 IP 地址。

```rust
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// 检查 URL 是否指向内网地址
fn is_private_url(url: &str) -> bool {
    let parsed = url::Url::parse(url);
    if parsed.is_err() {
        return true; // 无效 URL 视为不安全
    }
    let parsed = parsed.unwrap();

    // 解析 host 为 IP
    if let Some(host) = parsed.host_str() {
        // 检查是否为 IP 地址
        if let Ok(ip) = host.parse::<IpAddr>() {
            return ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_unspecified();
        }

        // 域名检查：常见内网域名
        let private_domains = [
            "localhost", "127.0.0.1", "0.0.0.0",
            "[::1]", "[::]", "10.", "172.16.",
            "192.168.", "169.254.",
        ];
        if private_domains.iter().any(|d| host.starts_with(d)) {
            return true;
        }
    }

    false
}

// 在 deliver_once 开头校验
async fn deliver_once(..., url: &str, ...) -> Result<(i32, String)> {
    if is_private_url(url) {
        return Err(AppError::Validation(
            "Webhook URL 不能指向内网地址".to_string()
        ));
    }
    // ... 原有逻辑
}
```

```rust
// 方案B：创建时即校验（推荐）
pub async fn create_subscription(
    &self,
    app_id: Uuid,
    url: &str,
    secret: &str,
    events: &[String],
) -> Result<WebhookSubscription> {
    if is_private_url(url) {
        return Err(AppError::Validation(
            "Webhook URL 不能指向内网地址".to_string()
        ));
    }
    // ... 原有 CREATE 逻辑
}
```

**影响文件**:
- `src/services/webhook_service.rs` — 添加 `is_private_url()` 校验函数，在 `deliver_once()` 和 `create_subscription()` 中调用
- `src/utils/security.rs` — 可选，将 `is_private_url()` 提取到 security 工具模块

**修复状态**: 已修复（2026-07-21）

**修复内容**:
在 `src/services/webhook_service.rs` 中新增 `is_private_url()` 校验函数，使用 `url::Url` 解析 URL 并检查 host 是否指向回环地址、私有地址、链路本地地址或未指定地址，同时拦截 `localhost`、`127.0.0.1`、`0.0.0.0`、`[::1]`、`[::]` 以及以 `10.`、`172.16.`、`192.168.`、`169.254.` 开头的域名。该校验在 `create_subscription`、`update_subscription` 和 `deliver_once` 三个入口统一执行，防止 Webhook 触发 SSRF。

---

## 0005-audit_service.rs 使用 format!() 拼接 SQL [P2-中优先级] - 已修复

**位置**: `src/services/audit_service.rs:956-1010` — `query_alerts()` 函数

**问题描述**:
`query_alerts()` 函数使用 `format!()` 将用户输入的 `alert_type`、`affected_user_id`、时间范围等参数直接拼接到 SQL 查询字符串中，存在 SQL 注入风险。该函数仅 admin 可调用（需绕过 admin 认证中间件），但仍违背安全最佳实践。

**当前代码** (`src/services/audit_service.rs:956-1010`):
```rust
pub async fn query_alerts(&self, query: AlertQuery) -> Result<(Vec<AuditAlertResponse>, i64)> {
    let mut sql = String::from(
        "SELECT ... FROM audit_alerts WHERE 1=1"
    );
    let mut count_sql = String::from("SELECT COUNT(*) FROM audit_alerts WHERE 1=1");

    if let Some(alert_type) = &query.alert_type {
        sql.push_str(&format!(" AND alert_type = '{}'", alert_type));       // ← SQLi
        count_sql.push_str(&format!(" AND alert_type = '{}'", alert_type)); // ← SQLi
    }
    if let Some(user_id) = query.affected_user_id {
        sql.push_str(&format!(" AND affected_user_id = '{}'", user_id));    // ← 安全（UUID）
        count_sql.push_str(&format!(" AND affected_user_id = '{}'", user_id));
    }
    if let Some(start_time) = query.start_time {
        sql.push_str(&format!(" AND created_at >= '{}'", start_time));      // ← 安全（DateTime）
        count_sql.push_str(&format!(" AND created_at >= '{}'", start_time));
    }
    // ...
    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));  // ← 安全（i64）
```

**风险分析**:
| 参数 | 类型 | 注入风险 |
|------|------|---------|
| `alert_type` | `Option<String>` | **高** — 任意字符串，直接拼接 |
| `status` | `Option<AlertStatus>` | 低 — serde 枚举，只接受预定义值 |
| `severity` | `Option<AuditSeverity>` | 低 — serde 枚举 |
| `affected_user_id` | `Option<Uuid>` | 低 — UUID 格式校验 |
| `start_time`/`end_time` | `Option<DateTime<Utc>>` | 低 — serde datetime 校验 |
| `limit`/`offset` | `Option<i64>` | 低 — 整数转字符串 |

**修复方案**:

使用 SQLx 的参数化查询 `$1`, `$2` 代替 `format!()`：

```rust
pub async fn query_alerts(&self, query: AlertQuery) -> Result<(Vec<AuditAlertResponse>, i64)> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send>> = Vec::new();
    let mut param_idx = 1u16;

    if let Some(status) = &query.status {
        let s = format!("{:?}", status).to_lowercase();
        conditions.push(format!("status = ${}", param_idx));
        // 使用 sqlx::query_builder 或条件参数
    }
    // ...
}
```

**最佳修复：使用 sqlx::QueryBuilder**:
```rust
use sqlx::QueryBuilder;

pub async fn query_alerts(&self, query: AlertQuery) -> Result<(Vec<AuditAlertResponse>, i64)> {
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);

    let mut builder = QueryBuilder::new(
        "SELECT id, rule_id, alert_type, severity, title, description, related_logs,
         source_ip, affected_user_id, status, acknowledged_by, acknowledged_at,
         resolved_by, resolved_at, created_at, updated_at
         FROM audit_alerts WHERE 1=1"
    );

    if let Some(alert_type) = &query.alert_type {
        builder.push(" AND alert_type = ");
        builder.push_bind(alert_type);  // ← 参数化绑定
    }
    if let Some(user_id) = query.affected_user_id {
        builder.push(" AND affected_user_id = ");
        builder.push_bind(user_id);
    }
    if let Some(start_time) = query.start_time {
        builder.push(" AND created_at >= ");
        builder.push_bind(start_time);
    }
    if let Some(end_time) = query.end_time {
        builder.push(" AND created_at <= ");
        builder.push_bind(end_time);
    }
    // status & severity 同理（枚举 → 字符串绑定）

    builder.push(" ORDER BY created_at DESC");
    builder.push(" LIMIT ");
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let alerts = builder
        .build_query_as::<AuditAlertResponse>()
        .fetch_all(self.db.pool())
        .await?;

    // 总数查询同理
    let mut count_builder = QueryBuilder::new(
        "SELECT COUNT(*) FROM audit_alerts WHERE 1=1"
    );
    // ... 相同条件（注意不要重复 ORDER BY / LIMIT）
    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(self.db.pool())
        .await?;

    Ok((alerts, total))
}
```

**影响文件**:
- `src/services/audit_service.rs` — `query_alerts()` 及所有涉及 `format!()` SQL 拼接的位置（约 20 行）
- 无需修改外部接口，`query_alerts` 的签名不变

**修复状态**: 已修复（2026-07-21）

**修复内容**:
将 `query_alerts()` 中的 SQL 构建方式从字符串拼接改为 `sqlx::QueryBuilder` + `push_bind()` 参数化绑定。所有查询条件（`alert_type`、`affected_user_id`、`start_time`、`end_time`、`status`、`severity`）以及分页参数均通过参数绑定传入，彻底消除 SQL 注入风险。

---

## 0006-file_service.rs 使用 format!() 拼接 SQL [P2-中优先级] - 已修复

**位置**: `src/services/file_service.rs:282-310` — `get_files_by_uploader()` 函数

**问题描述**:
`get_files_by_uploader()` 使用字符串拼接构建 SQL 查询，将 `uploader_id`（UUID）、`category`（枚举 Debug 输出）、`usage_type`（枚举 Debug 输出）直接拼入 SQL。虽然 `uploader_id` 来自 JWT Claims（仅当前用户可查自己的文件）、`category`/`usage_type` 经 serde 枚举校验（只接受预定义值），**实际注入风险极低**，但仍应使用参数化查询。

**当前代码** (`src/services/file_service.rs:282-310`):
```rust
pub async fn get_files_by_uploader(
    &self,
    uploader_id: Uuid,
    params: FileQueryParams,
) -> Result<FileListResponse> {
    let mut query = String::from(
        "SELECT * FROM file_resources WHERE is_deleted = false AND uploader_id = '"
    );
    query.push_str(&uploader_id.to_string());  // String 拼接
    query.push('\'');

    let mut count_query = String::from(
        "SELECT COUNT(*) FROM file_resources WHERE is_deleted = false AND uploader_id = '"
    );
    count_query.push_str(&uploader_id.to_string());
    count_query.push('\'');

    if let Some(ref category) = params.category {
        query.push_str(&format!(" AND category = '{:?}'", category));       // format!()
        count_query.push_str(&format!(" AND category = '{:?}'", category)); // format!()
    }
    if let Some(ref usage) = params.usage_type {
        query.push_str(&format!(" AND usage_type = '{:?}'", usage));        // format!()
        count_query.push_str(&format!(" AND usage_type = '{:?}'", usage));  // format!()
    }

    query.push_str(" ORDER BY created_at DESC");
    query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));  // i64，安全
}
```

**修复方案**:

使用 SQLx 的 `QueryBuilder` + 参数化绑定：

```rust
pub async fn get_files_by_uploader(
    &self,
    uploader_id: Uuid,
    params: FileQueryParams,
) -> Result<FileListResponse> {
    use sqlx::QueryBuilder;

    let mut builder = QueryBuilder::new(
        "SELECT * FROM file_resources WHERE is_deleted = false AND uploader_id = "
    );
    builder.push_bind(uploader_id);

    if let Some(ref category) = params.category {
        builder.push(" AND category = ");
        builder.push_bind(format!("{:?}", category).to_lowercase());
    }
    if let Some(ref usage) = params.usage_type {
        builder.push(" AND usage_type = ");
        builder.push_bind(format!("{:?}", usage).to_lowercase());
    }

    builder.push(" ORDER BY created_at DESC");

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    builder.push(" LIMIT ");
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let files = builder
        .build_query_as::<FileResource>()
        .fetch_all(self.db.pool())
        .await?;

    // 总数查询
    let mut count_builder = QueryBuilder::new(
        "SELECT COUNT(*) FROM file_resources WHERE is_deleted = false AND uploader_id = "
    );
    count_builder.push_bind(uploader_id);
    // ... 相同条件（不含 ORDER BY / LIMIT / OFFSET）
    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(self.db.pool())
        .await?;

    Ok(FileListResponse { files, total })
}
```

**影响文件**:
- `src/services/file_service.rs` — `get_files_by_uploader()` 函数（约 30 行）
- `src/models/file.rs` — 如果 `category`/`usage_type` 枚举未实现 `Display`，可能需要添加（或用 `serde_json::Value` 序列化）

**修复状态**: 已修复（2026-07-21）

**修复内容**:
将 `get_files_by_uploader()` 中的 SQL 构建方式从字符串拼接改为 `sqlx::QueryBuilder` + `push_bind()` 参数化绑定。`uploader_id`、`category`、`usage_type`、分页参数等均通过参数绑定传入，避免 SQL 注入风险。

---

## 汇总

| # | 问题 | 严重性 | 影响路径 | 是否需要新依赖 | 状态 |
|---|------|--------|---------|--------------|------|
| 0002 | WebSocket 无认证 | P1-高 | `routes/mod.rs` | 无 | 已修复 |
| 0003 | 登录无限流 | P1-高 | `routes/mod.rs` | 无（自定义中间件）| 已修复 |
| 0004 | Webhook SSRF | P2-中 | `webhook_service.rs` | `url` crate | 已修复 |
| 0005 | audit_service 格式 SQL | P2-中 | `audit_service.rs` | 无（sqlx::QueryBuilder 已内置）| 已修复 |
| 0006 | file_service 格式 SQL | P2-中 | `file_service.rs` | 无（sqlx::QueryBuilder 已内置）| 已修复 |
