# Phase 3：OAuth 2.0 授权生态 + 房间资源绑定 设计文档

> **更新日期**: 2026-06-19
> **状态**: 设计定稿
> **关联**: CapellaRoom v2 Roadmap Phase 3.1/3.2/3.5
> **前置文档**: `docs/superpowers/specs/2026-06-16-phase0-auth-integration-design.md`

---

## 一、概述

### 目标

在 Phase 1（认证规范化）完成的基础上，实施 Phase 3 的三个核心模块：

| 模块 | 对应路线图 | 说明 |
|------|-----------|------|
| **3.1 外部应用注册** | CapellaRoom 3.1 | `oauth_apps` CRUD API handlers |
| **3.2 OAuth 2.0 授权码流程** | CapellaRoom 3.2 | `authorize` + `token` + `userinfo` + 内嵌登录页 |
| **3.5 房间-资源绑定** | CapellaRoom 3.5 | `room_resource_bindings` 表 + CRUD API |

### 非目标

- SSO 社交登录（Phase 3.6）
- 自定义 WS 事件（Phase 3.4）
- 出站 Webhook（Phase 3.3）

### 已有基础设施

| 组件 | 状态 | 位置 |
|------|------|------|
| OAuth 数据模型 | ✅ 已实现 | `src/models/oauth.rs` |
| 数据库迁移 | ✅ 已实现 | `migrations/019_oauth_infrastructure.sql` |
| OAuthService 基础 | ✅ 已实现 | `src/services/oauth_service.rs` |
| AppState 注册 | ✅ 已实现 | `src/state/mod.rs` |

### 需要新增/扩展的组件

| 组件 | 类型 | 说明 |
|------|------|------|
| `src/handlers/oauth.rs` | 新建 | OAuth HTTP handlers |
| `src/oauth/` | 新建 | 内嵌 HTML 模板 |
| `src/routes/mod.rs` | 修改 | 注册 OAuth 路由 |
| `src/services/oauth_service.rs` | 扩展 | 补全授权码/token/userinfo/映射方法 |
| `src/models/oauth.rs` | 扩展 | 补全 Request/Response DTOs |
| `migrations/020_room_resource_bindings.sql` | 新建 | 房间资源绑定表 |

---

## 二、模块 3.1：外部应用注册 API

### 路由

所有端点需用户认证（`auth_middleware`）：

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| POST | `/api/v2/oauth/apps` | 创建应用 | 已登录用户 |
| GET | `/api/v2/oauth/apps` | 我的应用列表 | 已登录用户 |
| GET | `/api/v2/oauth/apps/:id` | 应用详情 | owner |
| PUT | `/api/v2/oauth/apps/:id` | 更新应用 | owner |
| DELETE | `/api/v2/oauth/apps/:id` | 删除应用 | owner |
| PATCH | `/api/v2/oauth/apps/:id/secret` | 重新生成 secret | owner |

### 请求/响应 DTO

```rust
// 创建应用
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOAuthAppRequest {
    #[validate(length(min = 1, max = 128))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1))]
    pub redirect_uris: Vec<String>,
    pub scopes: Option<Vec<String>>,
}

// 创建响应（含明文 secret，仅此一次）
pub struct OAuthAppCreatedResponse {
    pub id: Uuid,
    pub name: String,
    pub client_id: Uuid,          // = id
    pub client_secret: String,    // 明文，仅返回一次
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// 列表/详情响应（不含 secret）
pub struct OAuthAppResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Handler 实现要点

- `register_app`: 调用 `oauth_service.register_app()`，返回 `OAuthAppCreatedResponse`
- `list_apps`: 调用 `oauth_service.list_apps(owner_id)`
- `get_app`: 验证 `owner_id == claims.sub`，否则 403
- `update_app`: 同上权限校验
- `delete_app`: 同上权限校验
- `rotate_secret`: 调用 `oauth_service.rotate_secret()`，返回新明文 secret

---

## 三、模块 3.2：OAuth 2.0 授权码流程

### 3.2.1 授权端点（浏览器端点）

#### `GET /oauth/authorize`

渲染内嵌登录页 + 授权确认。

**Query 参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `response_type` | 是 | 固定 `code` |
| `client_id` | 是 | OAuth 应用 ID (UUID) |
| `redirect_uri` | 是 | 回调地址（须精确匹配已注册列表） |
| `state` | 否 | CSRF 令牌（推荐传递） |
| `scope` | 否 | 请求的权限范围 |

**流程**：

```
GET /oauth/authorize?client_id=...&redirect_uri=...&response_type=code&state=...

  验证 client_id 存在且 is_active
  验证 redirect_uri 精确匹配已注册列表
    ↓
  渲染登录页（login.html）
    ↓
  用户提交 email + password
    ↓
  POST /oauth/authorize 验证凭据
    ↓
  成功 → 渲染授权确认页（consent.html）
  失败 → 重新渲染登录页 + 错误消息
    ↓
  用户点击"授权"
    ↓
  POST /oauth/authorize/consent
    ↓
  生成 authorization_code（5 分钟过期，单次使用）
  302 Redirect → redirect_uri?code=xxx&state=xxx
```

#### `POST /oauth/authorize`

处理登录表单提交。

**Content-Type**: `application/x-www-form-urlencoded`

**Body**: `email`, `password`, `client_id`, `redirect_uri`, `response_type`, `state`, `scope`

**行为**：
1. 验证凭据（复用 `auth_service`）
2. 验证 app 存在且 active
3. 验证 redirect_uri 匹配
4. 生成一次性 `auth_session_token`（关联 user_id + client_id，5 分钟有效）
5. 渲染 consent.html（含 auth_session_token hidden field）

#### `POST /oauth/authorize/consent`

处理授权确认。

**Content-Type**: `application/x-www-form-urlencoded`

**Body**: `auth_session_token`, `client_id`, `redirect_uri`, `response_type`, `state`, `scope`, `approve` (或 `deny`)

**行为**：
1. 验证 auth_session_token 有效
2. `approve=true` → 生成 authorization_code → 302 redirect
3. `approve=false` 或缺失 → 302 redirect 到 `redirect_uri?error=access_denied`

### 3.2.2 Token 端点

#### `POST /oauth/token`

**Content-Type**: `application/json`

支持三种 grant_type：

**Authorization Code Grant**：
```json
{
    "grant_type": "authorization_code",
    "code": "the_authorization_code",
    "redirect_uri": "https://perseus.local/auth/callback",
    "client_id": "uuid",
    "client_secret": "perseus_sk_xxx"
}
```

**Refresh Token Grant**：
```json
{
    "grant_type": "refresh_token",
    "refresh_token": "the_refresh_token",
    "client_id": "uuid",
    "client_secret": "perseus_sk_xxx"
}
```

**Client Credentials Grant**：
```json
{
    "grant_type": "client_credentials",
    "client_id": "uuid",
    "client_secret": "perseus_sk_xxx"
}
```

**成功响应**（RFC 6749 标准格式）：
```json
{
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "scope": "profile email"
}
```

**错误响应**（RFC 6749 Section 5.2）：
```json
{
    "error": "invalid_grant",
    "error_description": "Authorization code has expired"
}
```

### 3.2.3 UserInfo 端点

#### `GET /api/v2/oauth/userinfo`

**Authorization**: `Bearer <access_token>`

**响应**：
```json
{
    "sub": "capella_user_uuid",
    "username": "alice",
    "email": "alice@example.com",
    "email_verified": true,
    "avatar_url": null
}
```

### 3.2.4 身份映射 API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v2/oauth/mappings` | 创建映射 |
| GET | `/api/v2/oauth/mappings/lookup` | 反查 |
| DELETE | `/api/v2/oauth/mappings/:id` | 解除 |

### 3.2.5 JWT 策略

- 独立密钥：`OAUTH_JWT_SECRET`（与应用 JWT 分离）
- 签名算法：HS256
- 有效期：access_token 1 小时，refresh_token 30 天
- Payload：
  ```json
  {
    "sub": "user_uuid",
    "aud": "app_id",
    "iss": "capella-room",
    "scope": "openid profile email",
    "iat": 1718461800,
    "exp": 1718465400
  }
  ```

### 3.2.6 内嵌登录页

`src/oauth/templates/` 目录：
- `login.html` — 登录表单（email + password）
- `consent.html` — 授权确认页（应用名 + 权限列表 + 授权/拒绝按钮）

通过 `include_str!` 编译期嵌入，使用 `format!()` 注入动态变量。

**会话跟踪**：不使用 cookie，改用一次性 `auth_session_token`（hidden field）。

---

## 四、模块 3.5：房间-资源绑定

### 4.1 数据库迁移

```sql
-- 020_room_resource_bindings.sql
CREATE TABLE room_resource_bindings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id         UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    resource_type   VARCHAR(64) NOT NULL,
    resource_id     VARCHAR(255) NOT NULL,
    resource_url    TEXT,
    resource_name   VARCHAR(255),
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, resource_type, resource_id)
);

CREATE INDEX idx_rrb_room_id ON room_resource_bindings(room_id);
CREATE INDEX idx_rrb_app_id ON room_resource_bindings(app_id);
CREATE INDEX idx_rrb_lookup ON room_resource_bindings(app_id, resource_type, resource_id);
```

### 4.2 Rust 模型

```rust
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RoomResourceBinding {
    pub id: Uuid,
    pub room_id: Uuid,
    pub app_id: Uuid,
    pub resource_type: String,
    pub resource_id: String,
    pub resource_url: Option<String>,
    pub resource_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateResourceBindingRequest {
    pub app_id: Uuid,
    #[validate(length(min = 1, max = 64))]
    pub resource_type: String,
    #[validate(length(min = 1, max = 255))]
    pub resource_id: String,
    pub resource_url: Option<String>,
    pub resource_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceLookupQuery {
    pub app_id: Uuid,
    pub resource_type: String,
    pub resource_id: String,
}
```

### 4.3 API 设计

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| POST | `/api/v2/rooms/:room_id/resources` | 绑定资源 | 房间管理员 |
| GET | `/api/v2/rooms/:room_id/resources` | 列表 | 房间成员 |
| GET | `/api/v2/resources/lookup` | 反查房间 | 已认证 |
| PATCH | `/api/v2/rooms/:room_id/resources/:id` | 更新元数据 | 房间管理员 |
| DELETE | `/api/v2/rooms/:room_id/resources/:id` | 解绑 | 房间管理员 |

### 4.4 Service 扩展

在 `OAuthService` 中新增方法（或新建 `RoomResourceService`）：

```rust
impl OAuthService {
    pub async fn create_resource_binding(
        &self, room_id: Uuid, request: CreateResourceBindingRequest
    ) -> Result<RoomResourceBinding>;

    pub async fn list_resource_bindings(
        &self, room_id: Uuid
    ) -> Result<Vec<RoomResourceBinding>>;

    pub async fn lookup_resource(
        &self, app_id: Uuid, resource_type: &str, resource_id: &str
    ) -> Result<Option<RoomResourceBinding>>;

    pub async fn update_resource_binding(
        &self, binding_id: Uuid, metadata: serde_json::Value
    ) -> Result<RoomResourceBinding>;

    pub async fn delete_resource_binding(
        &self, binding_id: Uuid
    ) -> Result<()>;
}
```

---

## 五、路由注册

### 新增路由组

```rust
// src/routes/mod.rs

// OAuth 浏览器端点（无 auth_middleware，内部处理认证）
let oauth_routes = Router::new()
    .route("/authorize", get(oauth_authorize_get).post(oauth_authorize_post))
    .route("/authorize/consent", post(oauth_authorize_consent))
    .route("/token", post(oauth_token));

// OAuth API 端点（需 auth_middleware）
let oauth_api_routes = Router::new()
    .route("/apps", post(create_app).get(list_apps))
    .route("/apps/:id", get(get_app).put(update_app).delete(delete_app))
    .route("/apps/:id/secret", patch(rotate_secret))
    .route("/userinfo", get(userinfo))
    .route("/mappings", post(create_mapping))
    .route("/mappings/lookup", get(lookup_mapping))
    .route("/mappings/:id", delete(delete_mapping));

// 房间资源绑定
let resource_routes = Router::new()
    .route("/rooms/:room_id/resources", post(bind_resource).get(list_bindings))
    .route("/rooms/:room_id/resources/:id", patch(update_binding).delete(unbind_resource))
    .route("/resources/lookup", get(lookup_resource));
```

### 路由挂载位置

```rust
// 主路由
.nest("/oauth", oauth_routes)                    // 浏览器端点
.nest("/api/v2/oauth", oauth_api_routes)          // API 端点（auth_middleware）
.nest("/api/v2", resource_routes)                 // 资源绑定（auth_middleware）
```

---

## 六、安全设计

### 6.1 凭据存储

| 凭据 | 存储方式 | 说明 |
|------|---------|------|
| `client_secret` | bcrypt 哈希 | 只在创建/轮换时返回明文一次 |
| `authorization_code` | 明文 + 过期时间 + used_at | 32 字节 CSPRNG → base64，5 分钟过期 |
| `refresh_token` | bcrypt 哈希 | 32 字节 CSPRNG → base64，30 天过期 |
| `access_token` | JWT（HS256 签名） | 不含敏感信息，Perseus 本地验证 |

### 6.2 防滥用

| 防护 | 措施 |
|------|------|
| CSRF | `state` 参数原样返回 |
| 重放攻击 | authorization_code 单次使用；refresh_token 单次使用 |
| redirect_uri | 精确字符串匹配（禁止子路径/通配符） |
| 速率限制 | 复用现有速率限制中间件 |

---

## 七、测试策略

### 测试文件

| 测试文件 | 覆盖内容 |
|---------|---------|
| `tests/phase3_1_oauth_apps_test.rs` | 应用 CRUD、secret 轮换、权限校验 |
| `tests/phase3_2_oauth_flow_test.rs` | authorize → code → token → userinfo 完整流程 |
| `tests/phase3_5_resource_bindings_test.rs` | 资源绑定 CRUD、反查、权限校验 |

### 关键测试场景

```
Phase 3.1:
  ✓ 创建应用返回 name + client_secret
  ✓ 查看应用列表
  ✓ 非 owner 无法修改/删除应用
  ✓ 重新生成 secret 后旧 secret 失效

Phase 3.2:
  ✓ authorize → code → token → userinfo 完整流程
  ✓ 过期 code 返回 invalid_grant
  ✓ 重复使用 code 返回 invalid_grant
  ✓ 无效 client_secret 返回 401
  ✓ refresh_token 换取新 access_token
  ✓ client_credentials 模式获取 token
  ✓ 不匹配的 redirect_uri 拒绝

Phase 3.5:
  ✓ 绑定资源到房间
  ✓ 列表房间资源
  ✓ 反查资源所在房间
  ✓ 更新资源元数据
  ✓ 解绑资源
  ✓ 非房间管理员无法绑定
```

---

## 八、实施顺序

| 步骤 | 模块 | 说明 |
|------|------|------|
| 1 | 迁移 | 新建 `020_room_resource_bindings.sql` |
| 2 | Models | 扩展 `oauth.rs`：补全 DTOs + 新增 `RoomResourceBinding` |
| 3 | Service | 扩展 `oauth_service.rs`：授权码/token/userinfo/映射/资源绑定 |
| 4 | Handlers | 新建 `handlers/oauth.rs`：所有 OAuth + 资源绑定 handlers |
| 5 | Templates | 新建 `oauth/templates/`：login.html + consent.html |
| 6 | Routes | 修改 `routes/mod.rs`：注册所有新路由 |
| 7 | Config | 扩展 `config/mod.rs`：新增 `oauth` 配置段 |
| 8 | Tests | 编写 3 个测试文件 |

---

## 九、配置扩展

```toml
# config.toml 新增
[oauth]
jwt_secret = "${OAUTH_JWT_SECRET}"     # OAuth JWT 签名密钥
access_token_ttl = 3600                 # access_token 有效期（秒）
refresh_token_ttl = 2592000             # refresh_token 有效期（秒，默认 30 天）
authorization_code_ttl = 300            # 授权码有效期（秒，默认 5 分钟）
```

---

*文档版本: 1.0.0*
*最后更新: 2026-06-19*
