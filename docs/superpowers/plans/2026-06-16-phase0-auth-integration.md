# Phase 0: Perseus 认证集成 — 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** CapellaRoom 实现 OAuth 2.0 Authorization Code Grant，提供应用注册、用户认证、身份映射能力，使 Perseus 可委托认证。

**Architecture:** 新增 `oauth_service.rs` 处理 OAuth 核心逻辑 + 嵌入式 HTML 登录页，复用现有 JWT 体系，独立签名密钥。

**Tech Stack:** Rust/Axum, SQLx, bcrypt, include_str! HTML

---

## 文件结构

### 新建文件

| 文件 | 职责 |
|------|------|
| `migrations/019_oauth_infrastructure.sql` | oauth_apps, authorization_codes, oauth_tokens, user_identity_mappings 表 |
| `src/models/oauth.rs` | OAuthApp, AuthorizationCode, OAuthToken, UserIdentityMapping 结构体 |
| `src/services/oauth_service.rs` | OAuth 全部业务逻辑 |
| `src/routes/oauth_routes.rs` | `/oauth/authorize`, `/oauth/token` 端点 |
| `src/routes/oauth_api_routes.rs` | `/api/v2/oauth/*` REST 端点 |
| `src/oauth/templates/login.html` | 登录 HTML 表单 |
| `src/oauth/templates/consent.html` | 授权确认 HTML 页面 |
| `tests/phase3_1_oauth_apps_test.rs` | 应用 CRUD 测试 (8 用例) |
| `tests/phase3_2_oauth_flow_test.rs` | OAuth 流程测试 (8 用例) |
| `tests/phaseP002_user_mapping_test.rs` | 身份映射测试 (4 用例) |

### 修改文件

| 文件 | 改动 |
|------|------|
| `src/models/mod.rs` | 加 `pub mod oauth;` |
| `src/services/mod.rs` | 加 `pub mod oauth_service;` |
| `src/routes/mod.rs` | 加 `oauth_routes()`, `oauth_api_routes()` |
| `src/config/mod.rs` | 加 `oauth_jwt_secret: String` Config 项 |
| `src/main.rs` | 注册 OAuthService + 路由 |

---

### Task 1: 迁移 SQL

- [ ] **Write `migrations/019_oauth_infrastructure.sql`**

```sql
-- oauth_apps
CREATE TABLE IF NOT EXISTS oauth_apps (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL,
    description     TEXT,
    client_secret   VARCHAR(256) NOT NULL,
    redirect_uris   TEXT[] NOT NULL DEFAULT '{}',
    scopes          TEXT[] NOT NULL DEFAULT '{}',
    owner_id        UUID NOT NULL REFERENCES users(id),
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- oauth_authorization_codes
CREATE TABLE IF NOT EXISTS oauth_authorization_codes (
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

-- oauth_tokens
CREATE TABLE IF NOT EXISTS oauth_tokens (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id              UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES users(id),
    access_token        VARCHAR(512) NOT NULL UNIQUE,
    refresh_token_hash  VARCHAR(256),
    scopes              TEXT[],
    expires_at          TIMESTAMPTZ NOT NULL,
    refresh_expires_at  TIMESTAMPTZ,
    revoked_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- user_identity_mappings
CREATE TABLE IF NOT EXISTS user_identity_mappings (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users(id),
    app_id            UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    external_user_id  VARCHAR(255) NOT NULL,
    external_username VARCHAR(255),
    mapped_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, external_user_id)
);
```

- [ ] **Register migration** — 将 019 编号加入迁移追踪（参考已有迁移文件的模式）

- [ ] **Commit**

```bash
git add migrations/019_oauth_infrastructure.sql
git commit -m "feat: add OAuth infrastructure migration (019)"
```

---

### Task 2: Rust 模型

- [ ] **Create `src/models/oauth.rs`**

```rust
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct OAuthApp {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub client_secret: String,
    pub redirect_uris: sqlx::types::Vec<String>,
    pub scopes: sqlx::types::Vec<String>,
    pub owner_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuthorizationCode {
    pub id: Uuid,
    pub app_id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub redirect_uri: Option<String>,
    pub scopes: Option<sqlx::types::Vec<String>>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct OAuthToken {
    pub id: Uuid,
    pub app_id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token_hash: Option<String>,
    pub scopes: Option<sqlx::types::Vec<String>>,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserIdentityMapping {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub external_user_id: String,
    pub external_username: Option<String>,
    pub mapped_at: DateTime<Utc>,
}
```

- [ ] **Add `pub mod oauth;` to `src/models/mod.rs`**

- [ ] **Add `bcrypt` dependency to `Cargo.toml`** — check if it's already there

- [ ] **Commit**

```bash
git add src/models/oauth.rs src/models/mod.rs
git commit -m "feat: add OAuth data models"
```

---

### Task 3: OAuthService — 应用管理

- [ ] **Write the failing test (`tests/phase3_1_oauth_apps_test.rs`)** — test_create_app, test_list_apps, test_update_app, test_delete_app, test_non_owner_forbidden, test_rotate_secret

```rust
use capella_room::test_helpers::*;
use capella_room::services::oauth_service::OAuthService;
use uuid::Uuid;

async fn setup_oauth_service() -> (OAuthService, sqlx::PgPool) {
    let app = create_test_app().await;
    let db = app.state().database.clone();
    let jwt_secret = "test_oauth_secret";
    let service = OAuthService::new(db.clone(), jwt_secret);
    (service, db)
}

#[tokio::test]
async fn test_register_app() {
    let (service, db) = setup_oauth_service().await;
    let owner_id = create_test_user_with_token(&db, "app_owner").await.0;

    let app = service.register_app(
        owner_id, "Test App", Some("description"),
        &["https://example.com/callback"],
        &["profile", "email"],
    ).await.expect("register app");

    assert_eq!(app.name, "Test App");
    assert_eq!(app.owner_id, owner_id);
    assert!(app.client_secret.len() > 20);
    assert!(app.is_active);
}
```

- [ ] **Run test to confirm failure** — `cargo test test_register_app --test phase3_1_oauth_apps_test` → fails (no module)

- [ ] **Implement `OAuthService::register_app`** + secret generation + bcrypt hash

```rust
// src/services/oauth_service.rs
pub struct OAuthService {
    db: Database,
    jwt_secret: String,
}

impl OAuthService {
    pub fn new(db: Database, jwt_secret: &str) -> Self {
        Self { db, jwt_secret: jwt_secret.to_string() }
    }

    pub async fn register_app(
        &self, owner_id: Uuid, name: &str, description: Option<&str>,
        redirect_uris: &[&str], scopes: &[&str],
    ) -> Result<OAuthApp> {
        let client_secret = Self::generate_client_secret();
        let secret_hash = Self::hash_secret(&client_secret);

        let app = sqlx::query_as::<_, OAuthApp>(
            r#"INSERT INTO oauth_apps (name, description, client_secret, redirect_uris, scopes, owner_id)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, name, description, client_secret, redirect_uris, scopes, owner_id, is_active, created_at, updated_at"#
        )
        .bind(name)
        .bind(description)
        .bind(&secret_hash)
        .bind(&redirect_uris.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .bind(&scopes.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .bind(owner_id)
        .fetch_one(&self.db)
        .await?;

        Ok(app)
    }

    fn generate_client_secret() -> String {
        use rand::Rng;
        let bytes: [u8; 32] = rand::thread_rng().gen();
        format!("capella_sk_{}", base64_url::encode(&bytes))
    }

    fn hash_secret(secret: &str) -> String {
        bcrypt::hash(secret, bcrypt::DEFAULT_COST).expect("bcrypt hash")
    }

    fn verify_secret(secret: &str, hash: &str) -> bool {
        bcrypt::verify(secret, hash).unwrap_or(false)
    }
}
```

- [ ] **Run test to confirm pass**

- [ ] **Write + implement remaining app management methods** — list, get, update, delete, rotate_secret (each with failing test first)

- [ ] **Commit**

```bash
git add src/services/oauth_service.rs src/services/mod.rs tests/phase3_1_oauth_apps_test.rs
git commit -m "feat: implement OAuth app registration service"
```

---

### Task 4: 应用管理 API 端点

- [ ] **Write failing test** — test POST /api/v2/oauth/apps returns 201, GET returns list, DELETE by non-owner returns 403

- [ ] **Create `src/routes/oauth_api_routes.rs`** — CRUD 端点

```rust
pub fn oauth_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v2/oauth/apps", post(create_app).get(list_apps))
        .route("/api/v2/oauth/apps/:id", get(get_app).put(update_app).delete(delete_app))
        .route("/api/v2/oauth/apps/:id/secret", patch(rotate_secret))
}
```

- [ ] **Register router in `src/routes/mod.rs`**

- [ ] **Run test to confirm pass**

- [ ] **Commit**

---

### Task 5: OAuthService — 授权码 + Token 签发

- [ ] **Write failing test `tests/phase3_2_oauth_flow_test.rs`**

```rust
#[tokio::test]
async fn test_authorize_creates_code() { ... }

#[tokio::test]
async fn test_exchange_code_returns_token() { ... }

#[tokio::test]
async fn test_expired_code_returns_error() { ... }

#[tokio::test]
async fn test_reused_code_returns_error() { ... }

#[tokio::test]
async fn test_invalid_secret_returns_error() { ... }

#[tokio::test]
async fn test_refresh_token_flow() { ... }

#[tokio::test]
async fn test_client_credentials_grant() { ... }

#[tokio::test]
async fn test_revoked_token_rejected() { ... }
```

- [ ] **Implement authorize logic in OAuthService**

```rust
pub async fn authorize(
    &self, user_id: Uuid, app_id: Uuid, redirect_uri: &str,
) -> Result<AuthorizationCode> {
    // 1. 验证 app 存在且活跃
    let app = self.get_app(app_id).await?;
    ensure(app.is_active, "app is inactive");
    // 2. 验证 redirect_uri 精确匹配
    ensure(
        app.redirect_uris.contains(&redirect_uri.to_string()),
        "redirect_uri not registered",
    );
    // 3. 生成随机 code（32 字节 CSPRNG）
    let code = Self::generate_authorization_code();
    // 4. 写入 oauth_authorization_codes（5 分钟过期）
    let record = sqlx::query_as::<_, AuthorizationCode>(
        r#"INSERT INTO oauth_authorization_codes (app_id, user_id, code, redirect_uri, expires_at)
           VALUES ($1, $2, $3, $4, NOW() + INTERVAL '5 minutes')
           RETURNING id, app_id, user_id, code, redirect_uri, scopes, expires_at, used_at, created_at"#
    )
    .bind(app_id)
    .bind(user_id)
    .bind(&code)
    .bind(redirect_uri)
    .fetch_one(&self.db)
    .await?;
    Ok(record)
}
```

- [ ] **Implement exchange_code logic**

```rust
pub async fn exchange_code(
    &self, code: &str, client_id: &str, client_secret: &str,
) -> Result<TokenResponse> {
    // 1. 查找 code 记录
    // 2. 验证未使用（used_at IS NULL）且未过期
    // 3. 验证 app_id 匹配 client_id
    // 4. 验证 client_secret（bcrypt verify）
    // 5. 标记 used_at = NOW()
    // 6. 签发 access_token（JWT）+ refresh_token
    // 7. 写入 oauth_tokens
    // 8. 返回 TokenResponse
}
```

- [ ] **Implement JWT generation for OAuth tokens**

```rust
fn generate_jwt(&self, user_id: Uuid, app_id: Uuid, scopes: &[&str]) -> Result<String> {
    use jsonwebtoken::{encode, Header, EncodingKey};
    let claims = serde_json::json!({
        "sub": user_id.to_string(),
        "aud": app_id.to_string(),
        "iss": "capella-room",
        "scope": scopes.join(" "),
        "iat": chrono::Utc::now().timestamp(),
        "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    });
    encode(&Header::default(), &claims, &EncodingKey::from_secret(self.jwt_secret.as_bytes()))
        .map_err(|e| anyhow::anyhow!("JWT encode: {}", e))
}
```

- [ ] **Implement refresh_token flow + client_credentials + revoke**

- [ ] **Run all tests to confirm pass**

- [ ] **Commit**

---

### Task 6: /oauth/token API 端点

- [ ] **Write integration test** — POST /oauth/token with valid code returns 200 + access_token

- [ ] **Create `src/routes/oauth_routes.rs`**

```rust
pub fn oauth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/oauth/token", post(token_handler))
        .route("/oauth/authorize", get(authorize_page).post(authorize_login))
        .route("/oauth/authorize/consent", post(authorize_consent))
}
```

- [ ] **Implement `token_handler`** — parse grant_type, dispatch to OAuthService, return JSON

- [ ] **Run test to confirm pass**

- [ ] **Commit**

---

### Task 7: 嵌入式登录页 + 授权确认页

- [ ] **Create `src/oauth/templates/login.html`**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="UTF-8"><title>登录 - CapellaRoom</title></head>
<body>
<div class="container">
  <h1>{app_name} 请求登录</h1>
  <form method="post" action="/oauth/authorize">
    <input type="hidden" name="client_id" value="{client_id}">
    <input type="hidden" name="redirect_uri" value="{redirect_uri}">
    <input type="hidden" name="response_type" value="code">
    <input type="hidden" name="state" value="{state}">
    <input type="email" name="email" placeholder="邮箱" required>
    <input type="password" name="password" placeholder="密码" required>
    <button type="submit">登录</button>
  </form>
  {error_html}
</div>
</body>
</html>
```

- [ ] **Create `src/oauth/templates/consent.html`**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="UTF-8"><title>授权确认 - CapellaRoom</title></head>
<body>
<div class="container">
  <h1>{app_name} 将获得以下权限</h1>
  <ul>{scopes_html}</ul>
  <form method="post" action="/oauth/authorize/consent">
    <input type="hidden" name="auth_session" value="{auth_session}">
    <input type="hidden" name="client_id" value="{client_id}">
    <input type="hidden" name="redirect_uri" value="{redirect_uri}">
    <input type="hidden" name="state" value="{state}">
    <button type="submit" name="approve" value="true">授权</button>
    <button type="submit" name="approve" value="false">拒绝</button>
  </form>
</div>
</body>
</html>
```

- [ ] **Add auth session tracking to OAuthService**

```rust
/// 登录→授权确认之间的临时会话（内存存储，不落 DB）
pub struct AuthSessionStore {
    sessions: Arc<DashMap<String, AuthSession>>,
}

struct AuthSession {
    user_id: Uuid,
    app_id: Uuid,
    redirect_uri: String,
    state: Option<String>,
    expires_at: Instant,
}
```

- [ ] **Implement `authorize_page`** (GET) — render login.html with app info

- [ ] **Implement `authorize_login`** (POST) — validate email+password, generate auth_session_token (CSPRNG 32 bytes, store in AuthSessionStore with 5min TTL), render consent.html

- [ ] **Implement `authorize_consent`** (POST) — validate auth_session_token from AuthSessionStore, call OAuthService::authorize, 302 redirect

- [ ] **Test manually** (no automated browser test for HTML rendering)

- [ ] **Commit**

---

### Task 8: Userinfo + 身份映射 API

- [ ] **Write failing test `tests/phaseP002_user_mapping_test.rs`** — create mapping, lookup, duplicate conflict, delete

- [ ] **Implement userinfo handler**

```rust
async fn userinfo_handler(
    state: ExtractorState,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let token = extract_bearer_token(&headers)?;
    let info = state.oauth_service.get_user_info(&token).await?;
    Ok(Json(serde_json::json!({
        "sub": info.user_id,
        "username": info.username,
        "email": info.email,
        "email_verified": info.email_verified,
        "avatar_url": info.avatar_url,
    })))
}
```

- [ ] **Implement identity mapping endpoints**

- [ ] **Register routes**

- [ ] **Run tests to confirm pass**

- [ ] **Commit**

---

### Task 9: 集成配置 + 路由注册

- [ ] **Add `oauth_jwt_secret` to `AppConfig`**

- [ ] **Initialize `OAuthService` in main.rs startup**

- [ ] **Register all routes**: `oauth_routes()`, `oauth_api_routes()`

- [ ] **Run existing test suite** — verify no regression

- [ ] **Commit**

```bash
git add src/main.rs src/config/mod.rs src/routes/mod.rs
git commit -m "feat: integrate OAuth routes and service into app"
```

---

### Task 10: 完整集成测试

- [ ] **Run full test suite** — `cargo test --test phase3_1_oauth_apps_test --test phase3_2_oauth_flow_test --test phaseP002_user_mapping_test`

- [ ] **Run all existing tests** to confirm no regression

- [ ] **Commit**
