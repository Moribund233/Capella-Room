# Phase 1 收尾：v1 Register Admin Guard + User 表扩展

> **日期**: 2026-06-16
> **定位**: CapellaRoom v2 Phase 1.4 / 1.5
> **基于**: [v2 roadmap](../../v2/roadmap.md)

---

## 1.4 v1 Register → Admin Guard

### 目标

`POST /api/v1/auth/register` 从公开端点转为仅 Admin 角色可调用的内部测试端点。

### 方案

**路由层**：从 `auth_routes()` 中移除 `/register`，新建独立路由组叠加认证+管理员中间件。

**改动文件**: `src/routes/mod.rs`

```rust
// auth_routes() 移除 register
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh_token))
}

// create_router() 中新增 protected register 路由块
let register_admin_router = Router::new()
    .route("/api/v1/auth/register", post(auth::register))
    .route("/api/auth/register", post(auth::register))
    .layer(middleware::from_fn_with_state(Arc::clone(&state), auth_middleware))
    .layer(middleware::from_fn_with_state(Arc::clone(&state), admin_auth_middleware));
```

### 认证顺序

1. `auth_middleware` — 验证 JWT → 注入 Claims
2. `admin_auth_middleware` — 检查 role 是否为 Admin/SuperAdmin

### 错误响应

非 Admin 请求返回：

```json
HTTP 403
{
  "success": false,
  "code": "FORBIDDEN",
  "error": "权限不足",
  "message": "需要管理员权限"
}
```

### 影响范围

| 项目 | 影响 |
|------|------|
| v1 API 兼容性 | 破坏性变更 — 外部用户无法再通过 v1 注册，需走 v2 验证码流程 |
| 文档 | 标记为"内部测试端点"，仅 Admin 可用 |
| 现有客户端 | 不受影响（客户端应已迁移到 v2 注册） |

---

## 1.5 User 表扩展

### 目标

为 `users` 表添加邮箱验证状态列，支撑 v2 认证体系中的"未验证邮箱不能使用忘记密码"等安全策略。

### 数据库迁移

**新增文件**: `migrations/018_add_email_verified_to_users.sql`

```sql
ALTER TABLE users
  ADD COLUMN email_verified     BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN email_verified_at  TIMESTAMPTZ;
```

- 所有已有用户 `email_verified = false`，需通过 v2 验证流程设为 true
- `email_verified_at` 记录首次验证时间，初始为 NULL

### Rust 模型变更

**改动文件**: `src/models/user.rs`

```rust
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    // ... 现有字段
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
}
```

### Service 层变更

**改动文件**: `src/services/user_service.rs`

`create_user` 的 INSERT 语句追加两列：

```sql
INSERT INTO users (username, email, password_hash, status, is_active, role, email_verified)
VALUES ($1, $2, $3, 'offline', true, 'user', false)
RETURNING id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
```

所有其他查询 `users` 表的 SQL 也需同步追加 `email_verified, email_verified_at` 到 RETURNING/SELECT 子句（如果使用了 `query_as::<User>` 则 `FromRow` 会自动映射，但 SELECT 列必须包含）。

### API 响应

`UserResponse` 不暴露 `email_verified` 字段（v2 需要时再添加）。

---

## 测试

| 测试 | 说明 |
|------|------|
| v1 register 无 token 请求 → 401 | 验证 middleware 拦截 |
| v1 register User 角色请求 → 403 | 验证权限检查 |
| v1 register Admin 角色请求 → 200 | 验证正常注册流程 |
| `email_verified` 默认值 → false | 验证新用户默认值 |
| 已有用户 `email_verified` → false | 迁移后查询确认 |

---

## 任务清单

1. `src/routes/mod.rs` — 拆分 register 路由 + admin guard
2. `migrations/018_add_email_verified_to_users.sql` — 新增列
3. `src/models/user.rs` — User struct 追加字段
4. `src/services/user_service.rs` — 所有 `SELECT *` / INSERT 同步
5. 检查所有 `query_as::<User>` 调用，确保 SELECT 列包含新字段
6. 更新文档标记
