# Phase 0：Perseus 认证集成设计

> **更新日期**: 2026-06-16
> **状态**: 设计定稿
> **关联**: CapellaRoom v2 Phase 3.1/3.2, Perseus Phase 0 (P-001~P-006)

---

## 一、概述

### 目标

CapellaRoom 作为 Perseus 的**统一认证中心**，提供 OAuth 2.0 Authorization Code Grant 流程。Perseus 将注册/登录委托给 CapellaRoom，用户通过 CapellaRoom 登录页面完成身份认证。

### 范围

| 模块 | 对应路线图 | 说明 |
|------|-----------|------|
| 外部应用注册 | CapellaRoom 3.1 | `oauth_apps` 表 + CRUD API |
| OAuth 2.0 授权码流程 | CapellaRoom 3.2 | `authorize` + `token` + 内嵌登录页 |
| 用户信息 API | Perseus P-002 | `userinfo` 端点 |
| 身份映射 | Perseus P-002 | `user_identity_mappings` 表 + API |
| 统一 JWT | Perseus P-004 | OAuth JWT 独立签发 + 密钥共享 |

### 非目标

- SSO 社交登录（GitHub/Google，规划 CapellaRoom 3.6）
- 自定义 WS 事件推送（规划 Phase 2 F-203）
- 房间-资源绑定（规划 Phase 2 F-201）

---

## 二、架构

### 认证架构

```
┌──────────────────────────────────────────────────────────────┐
│                         浏览器                                 │
│                                                               │
│   ┌─────────────────────┐     ┌───────────────────────────┐   │
│   │   Perseus 页面       │     │   CapellaRoom OAuth 页     │   │
│   │   (Vue 3)           │ ──→ │   (登录 + 授权确认)         │   │
│   │                     │ ←── │   302 redirect with code   │   │
│   └─────────┬───────────┘     └───────────────────────────┘   │
│             │                                                   │
│             │ POST /oauth/token                                │
│             │ GET  /api/v2/oauth/userinfo                      │
│             ▼                                                   │
│   ┌─────────────────────────────────────────────────────────┐ │
│   │              Perseus 后端 (FastAPI)                       │ │
│   │              验证 JWT, 创建/映射会话                      │ │
│   └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### 服务分层

```
CapellaRoom 端:

┌──────────────────────────────────────────┐
│              OAuth Router                  │
│  /oauth/authorize  GET|POST  → 登录页     │
│  /oauth/token      POST      → token 签发  │
│  /api/v2/oauth/*   CRUD      → REST API   │
└──────────────────┬───────────────────────┘
                   │
┌──────────────────▼───────────────────────┐
│           OAuthService                    │
│  应用管理 / code 签发 / token 验证 / 映射  │
└──────────────────┬───────────────────────┘
                   │
┌──────────────────▼───────────────────────┐
│   AuthService (已有)  /  UserService (已有)│
└──────────────────────────────────────────┘
```

---

## 三、数据模型

### 迁移 OAuth 基础设施

```sql
-- 3.1 外部应用注册
CREATE TABLE oauth_apps (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL,
    description     TEXT,
    client_secret   VARCHAR(128) NOT NULL,
    redirect_uris   TEXT[] NOT NULL DEFAULT '{}',
    scopes          TEXT[] NOT NULL DEFAULT '{}',
    owner_id        UUID NOT NULL REFERENCES users(id),
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3.2 授权码（一次性，有效期 5 分钟）
CREATE TABLE oauth_authorization_codes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id),
    code            VARCHAR(64) NOT NULL UNIQUE,
    redirect_uri    TEXT,
    scopes          TEXT[],
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3.2 OAuth 令牌
CREATE TABLE oauth_tokens (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id              UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES users(id),
    access_token        VARCHAR(512) NOT NULL UNIQUE,
    refresh_token       VARCHAR(512) UNIQUE,
    scopes              TEXT[],
    expires_at          TIMESTAMPTZ NOT NULL,
    refresh_expires_at  TIMESTAMPTZ,
    revoked_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- P-002 用户身份映射
CREATE TABLE user_identity_mappings (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users(id),
    app_id            UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    external_user_id  VARCHAR(255) NOT NULL,
    external_username VARCHAR(255),
    mapped_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, external_user_id)
);
```

### Rust 模型

新建 `src/models/oauth.rs`：

```rust
pub struct OAuthApp {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub client_secret: String,      // 加密存储
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub owner_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct AuthorizationCode {
    pub id: Uuid,
    pub app_id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub redirect_uri: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct OAuthToken {
    pub id: Uuid,
    pub app_id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct UserIdentityMapping {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub external_user_id: String,
    pub external_username: Option<String>,
    pub mapped_at: DateTime<Utc>,
}
```

---

## 四、API 设计

### 4.1 应用管理 API（3.1）

所有端点需用户认证（`Authorization: Bearer <user_jwt>`）：

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| POST | `/api/v2/oauth/apps` | 创建应用 | 已登录用户 |
| GET | `/api/v2/oauth/apps` | 我的应用列表 | 已登录用户 |
| GET | `/api/v2/oauth/apps/:id` | 应用详情 | owner |
| PUT | `/api/v2/oauth/apps/:id` | 更新应用 | owner |
| DELETE | `/api/v2/oauth/apps/:id` | 删除应用 | owner |
| PATCH | `/api/v2/oauth/apps/:id/secret` | 重新生成 secret | owner |

`POST /api/v2/oauth/apps` 请求体：

```json
{
    "name": "Perseus",
    "description": "Code collaboration platform",
    "redirect_uris": ["https://perseus.local/auth/callback"],
    "scopes": ["openid", "profile", "email"]
}
```

响应（含明文 `client_secret`，仅创建时返回）：

```json
{
    "id": "uuid",
    "name": "Perseus",
    "client_id": "uuid",
    "client_secret": "perseus_sk_xxxxxxxxxxxx",
    "redirect_uris": ["..."],
    "scopes": ["..."],
    "is_active": true,
    "created_at": "..."
}
```

### 4.2 OAuth 2.0 端点

#### GET /oauth/authorize（浏览器端点）

渲染 CapellaRoom 登录页 + 授权确认页。

**请求参数**（query string）：

| 参数 | 必填 | 说明 |
|------|------|------|
| `response_type` | 是 | 固定 `code` |
| `client_id` | 是 | OAuth 应用 ID |
| `redirect_uri` | 是 | 授权回调地址（须匹配已注册的 redirect_uris） |
| `state` | 否 | CSRF 令牌（推荐传递） |
| `scope` | 否 | 请求的权限范围 |

**流程**：

```
GET /oauth/authorize?client_id=...&redirect_uri=...&response_type=code&state=...

  ┌─ 用户已登录 CapellaRoom? ──┐
  │       │         │           │
  │     是 否       否          │
  │       │         │           │
  │       ▼         ▼           │
  │  展示授权   展示登录页       │
  │  确认页   ← 提交表单 →       │
  │       │   验证凭据           │
  │       ▼         │           │
  │  用户同意?       ▼          │
  │   │     │   登录成功         │
  │  是    否      │             │
  │   │     │      │             │
  │   ▼     ▼      │             │
  │ 生成code 拒绝    │             │
  └──────────────────────────────┘
        │
        ▼
  302 Redirect → redirect_uri?code=xxx&state=xxx
```

**`POST /oauth/authorize`**（表单提交）：

- `Content-Type: application/x-www-form-urlencoded`
- Body: `email=...&password=...&client_id=...&redirect_uri=...&response_type=code&state=...`
- 验证凭据成功 → 渲染授权确认页（显示应用名、请求权限）
- 验证失败 → 渲染登录页 + 错误消息

**`POST /oauth/authorize/consent`**（确认授权）：

- Body: `client_id=...&redirect_uri=...&response_type=code&state=...&approve=true`
- 验证用户会话有效
- 生成一次性 `authorization_code`（有效期 5 分钟）
- 302 Redirect → `redirect_uri?code=xxx&state=xxx`

#### POST /oauth/token（API 端点）

`Content-Type: application/json`，不支持浏览器表单。

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

**Client Credentials Grant**（服务端对服务端，无用户上下文）：

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

### 4.3 用户信息 API（P-002）

```
GET /api/v2/oauth/userinfo
Authorization: Bearer <access_token>
```

响应：

```json
{
    "sub": "capella_user_uuid",
    "username": "alice",
    "email": "alice@example.com",
    "email_verified": true,
    "avatar_url": null
}
```

### 4.4 身份映射 API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v2/oauth/mappings` | 创建映射 |
| GET | `/api/v2/oauth/mappings/lookup` | 反查房间 |
| DELETE | `/api/v2/oauth/mappings/:id` | 解除映射 |

`POST /api/v2/oauth/mappings`：

```json
{
    "app_id": "uuid",
    "user_id": "capella_user_uuid",
    "external_user_id": "perseus_user_id_123",
    "external_username": "alice"
}
```

---

## 五、认证流程

### 完整登录流程

```
[Perseus 用户点击"使用 CapellaRoom 登录"]

Step 1: 浏览器重定向
  Perseus → 用户浏览器
  302 Location: https://capella.local/oauth/authorize
    ?response_type=code
    &client_id=<perseus_app_id>
    &redirect_uri=https://perseus.local/auth/callback
    &state=<random_csrf_token>
    &scope=openid+profile+email

Step 2: CapellaRoom 登录
  用户看到 CapellaRoom 登录页（email + password 表单）
  用户填写凭据并提交 POST /oauth/authorize
  验证成功 → 渲染授权确认页（"Perseus 将访问你的用户名和邮箱"）

Step 3: 用户授权
  用户点击"授权" → POST /oauth/authorize/consent
  CapellaRoom 验证用户会话
  生成 authorization_code（5 分钟过期，单次使用）
  302 Location: https://perseus.local/auth/callback?code=<xxx>&state=<yyy>

Step 4: Perseus 换 token
  Perseus 后端 POST /oauth/token
  Body: { grant_type: "authorization_code", code, client_id, client_secret }
  CapellaRoom 验证 code + client_secret
  返回 { access_token, refresh_token, expires_in }

Step 5: Perseus 获取用户信息
  Perseus 后端 GET /api/v2/oauth/userinfo
  Header: Authorization: Bearer <access_token>
  返回 { sub, username, email, email_verified }

Step 6: Perseus 创建/映射用户
  Perseus 查找 sub 是否已有映射
  无 → 创建 Perseus 本地用户 + 创建 identity mapping
  有 → 更新会话
  创建 Perseus 会话，返回前端
```

### JWT 策略

- OAuth JWT 独立于 CapellaRoom 用户 JWT，使用**独立的签名密钥** (`OAUTH_JWT_SECRET`)
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
- Perseus 通过共享 `OAUTH_JWT_SECRET` 环境变量本地验证 token
- 未来可升级为 RS256 + JWKS 端点

---

## 六、服务层

新建 `src/services/oauth_service.rs`：

```rust
pub struct OAuthService {
    db: Database,
    jwt_secret: String,
}

impl OAuthService {
    // 应用管理
    pub async fn register_app(...) -> Result<OAuthApp>;
    pub async fn get_app(app_id: Uuid) -> Result<OAuthApp>;
    pub async fn list_apps(owner_id: Uuid) -> Result<Vec<OAuthApp>>;
    pub async fn update_app(...) -> Result<OAuthApp>;
    pub async fn delete_app(app_id: Uuid, owner_id: Uuid) -> Result<()>;
    pub async fn rotate_secret(app_id: Uuid, owner_id: Uuid) -> Result<String>;

    // OAuth Core
    pub async fn authorize(
        user_id: Uuid, app_id: Uuid, redirect_uri: &str
    ) -> Result<AuthorizationCode>;

    pub async fn exchange_code(
        code: &str, client_id: &str, client_secret: &str
    ) -> Result<TokenResponse>;

    pub async fn exchange_refresh_token(...) -> Result<TokenResponse>;

    pub async fn client_credentials_grant(
        client_id: &str, client_secret: &str
    ) -> Result<TokenResponse>;

    pub async fn get_user_info(access_token: &str) -> Result<UserInfo>;

    pub async fn revoke_token(access_token: &str) -> Result<()>;

    // 身份映射
    pub async fn create_mapping(...) -> Result<UserIdentityMapping>;
    pub async fn lookup_mapping(app_id: Uuid, external_user_id: &str) -> Result<Option<UserIdentityMapping>>;
    pub async fn delete_mapping(id: Uuid) -> Result<()>;

    // 内部
    fn generate_client_secret() -> String;
    fn generate_authorization_code() -> String;
    fn generate_jwt(user_id: Uuid, app_id: Uuid, scopes: &[&str]) -> Result<String>;
    fn verify_jwt(token: &str) -> Result<Claims>;
    fn hash_secret(secret: &str) -> String;
    fn verify_secret(secret: &str, hash: &str) -> bool;
}
```

### AppState 注册

```rust
// src/main.rs 或初始化
pub struct AppState {
    // ... 现有字段
    pub oauth_service: Arc<OAuthService>,
}
```

---

## 七、嵌入式登录页

`GET /oauth/authorize` 渲染的 HTML 页面内嵌在 Rust 二进制中：

```
src/
└── oauth/
    ├── mod.rs
    └── templates/
        ├── login.html          # 登录表单
        └── consent.html        # 授权确认页
```

通过 `include_str!` 编译期嵌入，无需模板引擎。

**login.html**：email + password 表单，提交到 `POST /oauth/authorize`

```html
<!DOCTYPE html>
<form method="post" action="/oauth/authorize">
    <input type="hidden" name="client_id" value="{{client_id}}">
    <input type="hidden" name="redirect_uri" value="{{redirect_uri}}">
    <input type="hidden" name="response_type" value="code">
    <input type="hidden" name="state" value="{{state}}">
    <input type="email" name="email" placeholder="Email" required>
    <input type="password" name="password" placeholder="Password" required>
    <button type="submit">登录</button>
</form>
```

**consent.html**：显示应用名 + 请求权限 + 确认/拒绝按钮

实际实现中，使用 `format!()` 或简单字符串替换注入动态变量。

**会话跟踪方案**：登录后到授权确认之间需跟踪用户身份。不使用 cookie 会话，改为两步：
1. `POST /oauth/authorize` 验证凭据后，生成一次性 `auth_session_token`（关联 user_id + client_id，有效期 5 分钟），渲染 consent.html 时以 hidden field 注入
2. `POST /oauth/authorize/consent` 验证 `auth_session_token` 有效性后签发 authorization_code

---

## 八、测试策略

TDD 驱动，分 3 个测试文件：

| 测试文件 | 用例数 | 覆盖内容 |
|---------|--------|---------|
| `tests/phase3_1_oauth_apps_test.rs` | 8 | 应用 CRUD、secret 轮换、权限校验（非 owner 403） |
| `tests/phase3_2_oauth_flow_test.rs` | 8 | authorize → code → token → userinfo 完整流程、code 过期、重复使用、client_credentials |
| `tests/phaseP002_user_mapping_test.rs` | 4 | 映射创建、反查、重复冲突、删除 |

关键测试场景：

```
Phase3.1:
  ✓ 注册新应用返回 name + client_secret
  ✓ 查看我的应用列表
  ✓ 非 owner 无法修改/删除应用
  ✓ 重新生成 secret 后旧 secret 失效

Phase3.2:
  ✓ authorize → code → token → userinfo 完整流程
  ✓ 使用过期 code 返回 invalid_grant
  ✓ 重复使用 code 返回 invalid_grant
  ✓ 无效 client_secret 返回 401
  ✓ refresh_token 换取新 access_token
  ✓ client_credentials 模式获取 token（无用户上下文）
  ✓ 已吊销的 token 返回 401
  ✓ 不匹配的 redirect_uri 拒绝
```

---

## 九、安全设计

### 9.1 凭据存储

| 凭据 | 存储方式 | 说明 |
|------|---------|------|
| `client_secret` | **bcrypt 哈希**（非加密） | 只在创建/轮换时返回明文一次，不可逆存储 |
| `authorization_code` | 明文 + `expires_at` + `used_at` | 随机 32 字节 CSPRNG → base64(43 chars)，5 分钟过期 |
| `refresh_token` | **bcrypt 哈希** | 随机 32 字节 CSPRNG → base64(43 chars)，30 天过期 |
| `access_token` | JWT（签名不加密） | Payload 不含敏感信息，Perseus 本地验证签名即可 |

`client_secret` 和 `refresh_token` 在 DB 中均为 bcrypt 哈希值，即使 DB 泄露也无法逆向。

### 9.2 JWT 签名

- 独立密钥：`OAUTH_JWT_SECRET`（与应用 JWT 分离），256 位以上
- 签名算法：HS256
- Payload 不含密码/密钥等敏感信息
- 共享给 Perseus 的方式：环境变量（`OAUTH_JWT_SECRET`）或后续升级为 RS256 + JWKS

### 9.3 输入校验

| 端点 | 校验规则 |
|------|---------|
| `POST /oauth/authorize` | `client_id` 存在且 `is_active`；`redirect_uri` 精确匹配已注册列表（非前缀匹配） |
| `POST /oauth/token` | `client_secret` bcrypt 比对；code 检查 `used_at IS NULL` 且未过期 |
| `GET /oauth/authorize` | `client_id` 存在；`redirect_uri` 必须已注册 |

### 9.4 防滥用

| 防护 | 措施 |
|------|------|
| CSRF | `state` 参数必传（authorize → callback 原样返回） |
| 重放攻击 | authorization_code 单次使用（`used_at` 标记）；refresh_token 单次使用（旧 token 立即作废） |
| 暴力破解 | 登录接口复用现有速率限制；`/oauth/token` 限制 100 req/min per IP |
| redirect_uri 校验 | 精确字符串匹配（禁止子路径/通配符绕过） |

### 9.5 数据最小化

- `userinfo` 端点只返回 `sub`、`username`、`email`、`email_verified`、`avatar_url`
- JWT payload 不包含用户密码、secret、内部 ID
- scope 机制控制第三方应用可访问的数据范围

---

## 十、错误处理

### OAuth 错误响应（RFC 6749 Section 5.2）

```json
{
    "error": "invalid_grant",
    "error_description": "Authorization code has expired"
}
```

| HTTP 状态 | `error` | 场景 |
|-----------|---------|------|
| 400 | `invalid_request` | 缺少必填参数、redirect_uri 不匹配 |
| 400 | `invalid_grant` | code 过期/已使用/无效、refresh_token 无效 |
| 400 | `unauthorized_client` | client_secret 不匹配、应用未激活 |
| 400 | `unsupported_grant_type` | grant_type 不支持 |
| 401 | `invalid_token` | access_token 无效/过期/已吊销 |
| 401 | `invalid_client` | client_id 不存在 |

### API 错误（REST 风格）

```json
{
    "error": "FORBIDDEN",
    "message": "Only the app owner can perform this action"
}
```

| HTTP | 场景 |
|------|------|
| 400 | 应用名称为空、redirect_uris 格式错误 |
| 401 | 未认证 |
| 403 | 非 owner 操作他人应用 |
| 404 | 应用/映射不存在 |
| 409 | 身份映射重复 |

---

## 十一、实施计划

### 实施顺序

| 步骤 | 模块 | 估算 |
|------|------|------|
| 1 | 迁移 SQL + `src/models/oauth.rs` | 0.5 天 |
| 2 | `OAuthService` 应用管理 + 秘密管理 | 1 天 |
| 3 | `OAuthService` 授权码 + token 签发 | 1.5 天 |
| 4 | `OAuthService` 身份映射 + userinfo | 0.5 天 |
| 5 | `POST /oauth/token` API 端点 + 路由 | 0.5 天 |
| 6 | `GET/POST /oauth/authorize` 浏览器端点 + HTML 模板 | 1 天 |
| 7 | 应用注册 CRUD API 端点 | 0.5 天 |
| 8 | 身份映射 + userinfo API 端点 | 0.5 天 |
| 9 | 路由注册 + AppState 集成 | 0.5 天 |
| 10 | Phase3.1 测试 | 0.5 天 |
| 11 | Phase3.2 + P002 测试 | 1 天 |

**合计**：约 8 天

---

## 十二、后续扩展

Phase 2 在此基础之上：

- **F-201 项目房间映射**：复用 `user_identity_mappings` 模式，扩展 `room_resource_bindings`
- **F-203 业务事件推送**：OAuth 客户端认证的 WS 连接发送 `CustomEvent`
- **F-205 实时通知**：通过 `OAuthService` 颁发的 token 关联用户身份
