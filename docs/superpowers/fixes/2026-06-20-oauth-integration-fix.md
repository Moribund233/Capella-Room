# OAuth 集成两侧修复方案

> **日期**: 2026-06-20
> **涉及项目**: CapellaRoom（授权方）, Perseus（客户端）
> **状态**: 待实施

---

## 根因分析

Perseus 作为 OAuth 客户端集成 CapellaRoom 时遇到的问题，根源在于 **CapellaRoom Phase3 实现存在架构性缺陷**：

1. **OAuth APIs 不接受 OAuth token** — `/oauth/mappings`, `/oauth/apps`, Webhook, 资源绑定等 API 挂载在 `auth_middleware` 之下，要求 CapellaRoom 自身的系统 JWT。外部服务通过 OAuth 流程拿到的 `access_token` 只能调用 `/oauth/userinfo`。
2. **Token endpoint 只接受 JSON** — 违反 RFC 6749 要求（必须支持 `application/x-www-form-urlencoded`）。
3. **CustomEvent WS 无 OAuth 身份验证** — 任何房间成员都可发送自定义事件，`source_app` 硬编码为 `"self"`。

Perseus 因此被迫用共享 `jwt_secret` 自签 admin 令牌调用 CapellaRoom API，产生了 1.1 安全缺陷。其他 Perseus 侧问题（事务顺序、未调 unbind API、未持久化 token 等）属于客户端实现缺陷。

---

## 修复策略

**先修 CapellaRoom（授权方），再修 Perseus（客户端）**。两侧独立实施。

---

## 一、CapellaRoom 侧修复

### F1. OAuth token 认证中间件（核心修复）

**问题**: 所有 OAuth 管理 API（mappings, webhooks, resources）都需要 CapellaRoom 的 `auth_middleware`，无法用 OAuth `access_token` 调用。

**方案**: 新增 `oauth_token_middleware`，与现有 `auth_middleware` 并列。中间件逻辑：

```
请求到达
  → 提取 Authorization: Bearer <token>
  → 尝试解析为 OAuth access_token（verify_access_token）
  → 成功 → 注入 OAuthClaims（含 app_id, user_id, scope）
  → 尝试解析为 CapellaRoom JWT（现有流程）
  → 成功 → 注入 Claims
  → 都失败 → 401
```

**新增文件**: `src/middleware/oauth_token.rs`

```rust
pub enum AuthInfo {
    CapellaRoom(Claims),
    OAuth {
        user_id: Uuid,
        app_id: Uuid,
        scopes: Vec<String>,
    },
}
```

**修改路由结构** (`src/routes/mod.rs`):

将 `oauth_protected_routes()`, `room_resource_routes()`, `webhook_routes()`, `custom_event_routes()` 从 `protected_routes` 中移出，使用独立的中间件层：

```rust
let oauth_api_routes = Router::new()
    .merge(oauth_protected_routes())
    .merge(room_resource_routes())
    .merge(webhook_routes())
    .merge(custom_event_routes())
    .layer(middleware::from_fn_with_state(
        Arc::clone(&state),
        oauth_or_app_auth_middleware,  // 接受 OAuth token 或 CapellaRoom JWT
    ));
```

**修改 handler 签名**: 所有 OAuth-related handler 从 `Extension(claims): Extension<Claims>` 改为接受 `Extension(auth): Extension<AuthInfo>`，根据类型分发。

### F2. Token endpoint 支持 form-urlencoded（RFC 合规）

**问题**: `/oauth/token` handler (`oauth.rs:331`) 使用 `Json<TokenRequest>`，只接受 `application/json`。RFC 6749 §4.1.3 要求必须接受 `application/x-www-form-urlencoded`。

**方案**: 同时支持两种 Content-Type：

```rust
// 新增 FormTokenRequest（字段与 TokenRequest 一致但从 form 解析）
pub async fn token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Either<Json<TokenRequest>, Form<TokenRequest>>,
) -> Result<Json<TokenResponse>> {
    let req = match request {
        Either::Left(json) => json.0,
        Either::Right(form) => form.0,
    };
    // 原有交换逻辑不变
}
```

### F3. CustomEvent WS 增加 OAuth 应用身份验证

**问题**: `handle_custom_event` (`handler.rs:2214`) 只检查房间成员身份，未验证 OAuth 应用身份。`source_app` 硬编码 `"self"`。

**方案**: 

1. 在 WebSocket 连接握手时，允许传入 OAuth `access_token`（通过 query param 或 Sec-WebSocket-Protocol 头）
2. 握手阶段解析并存储关联的 `app_id`
3. `handle_custom_event` 验证：
   - 连接必须关联到一个 OAuth app
   - `event_name` 必须以 `{app_name}:` 开头（app_name 从 oauth_apps 表查出 / 使用 app_id 的代号）
   - `source_app` 使用实际的 app_id/app_name，而非 `"self"`

**后续扩展**: 若当前 WS 握手无 OAuth，可考虑为 Perseus 服务端引入独立 WS 连接（不走用户登录 WS），通过 Client Credentials Grant 获取 token 并建立服务端 WS 连接。

### F4. 补充缺失的 Webhook 事件

**问题**: 设计文档定义 9 种事件，实际只实现了 5 种。缺少 `user.status_changed`, `user.mentioned`, `reaction.added`, `reaction.removed`。

**方案**: 在对应的 handler 中追加 `dispatch_event` 调用：

| 事件 | 插入位置 | 状态 |
|------|---------|------|
| `user.status_changed` | `handle_update_status` | 新增 |
| `user.mentioned` | `send_mention` / `process_mentions` | 新增 |
| `reaction.added` | `handle_add_reaction` | 新增 |
| `reaction.removed` | `handle_remove_reaction` | 新增 |

### F5. Webhook 重试：指数退避 + 后台扫描

**问题**: `webhook_service.rs:178` 重试间隔写死 `INTERVAL '10 seconds'`，无后台扫描任务。

**方案**:

1. 修改 `webhook_deliveries` 更新语句，根据 `attempt_count` 计算退避：

```rust
let backoff_seconds = 10 * 2_i64.pow(attempt_count - 1); // 10s, 20s, 40s
// UPDATE ... SET next_retry_at = NOW() + ({backoff_seconds} || ' seconds')::interval
```

2. 实现 `retry_failed_deliveries` 后台任务（`webhook_service.rs` 已有方法签名，补全实现），每 30s 扫描 pending/failed 投递并重试。

### F6. 资源绑定 CRUD 增加 WS 广播

**问题**: `bind_resource` / `unbind_resource` / `update_binding` 没有向房间广播。

**方案**: 每个操作后调用 `ws_manager.broadcast_to_room` 发送对应事件：

```rust
// bind_resource 末尾
let event = WebSocketMessage::RoomResourceBound { ... };
state.ws_manager().broadcast_to_room(room_id, event.to_json()?, None).await;

// unbind_resource 末尾
let event = WebSocketMessage::RoomResourceUnbound { ... };
state.ws_manager().broadcast_to_room(room_id, event.to_json()?, None).await;

// update_binding 末尾
let event = WebSocketMessage::RoomResourceUpdated { ... };
state.ws_manager().broadcast_to_room(room_id, event.to_json()?, None).await;
```

### F7. CustomNotification 实现

**问题**: 设计文档定义了 `CustomNotification`/`CustomNotificationForward`，但代码中没有。

**方案**: 按 `integration-design.md` 中的定义补充协议变体和 handler。优先级低，可在 Perseus 确实需要通知功能时再实施。

### F8. 补充集成测试

为以下模块添加集成测试：

| 模块 | 最少测试数 | 覆盖场景 |
|------|-----------|---------|
| OAuth token 交换 | 3 | authorization_code, refresh_token, client_credentials |
| Identity mapping | 3 | create, lookup, delete (含 OAuth token 鉴权) |
| Webhook subscription | 3 | CRUD + dispatch + delivery |
| CustomEvent HTTP API | 2 | send + get_missed |
| Resource binding | 3 | bind + unbind + lookup + WS broadcast |

---

## 二、Perseus 侧修复

### P1. 移除自签 admin JWT，改用标准 OAuth token

**前提**: CapellaRoom F1 完成后。

**修改 `capella_service.py`**:

```python
# 删除 create_mapping 中的 self-signed JWT
# 改用构造函数传入的 access_token（在回调流程中已获取）

class CapellaService:
    def __init__(self, config: CapellaSettings, access_token: str | None = None):
        self.config = config
        self._access_token = access_token
        self._client = httpx.AsyncClient(base_url=config.api_base_url)

    async def create_mapping(
        self, capella_user_id: str, external_user_id: int, external_username: str
    ) -> dict:
        resp = await self._client.post(
            "/oauth/mappings",
            headers={"Authorization": f"Bearer {self._access_token}"},
            json={
                "app_id": self.config.client_id,
                "user_id": capella_user_id,
                "external_user_id": str(external_user_id),
                "external_username": external_username,
            },
        )
        resp.raise_for_status()
        return resp.json()
```

**修改 `capella_controller.py`**:

回调流程中，构造 `CapellaService` 时传入 `access_token`：

```python
token_data = await capella_service.exchange_code(code)
userinfo = await capella_service.get_userinfo(token_data["access_token"])

# 使用 OAuth access_token 创建映射
mapping_svc = CapellaService(config.capella, access_token=token_data["access_token"])
await mapping_svc.create_mapping(...)
```

### P2. Unbind 时调用 CapellaRoom 清理映射

**修改 `capella_controller.py:unbind`**:

```python
@router.post("/api/v1/capella/unbind")
async def unbind(
    current_user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_async_db),
    capella_service: CapellaService = Depends(get_capella_service),
):
    if not current_user.capella_user_id:
        raise HTTPException(status_code=409, detail="Not bound")

    try:
        # 先调 CapellaRoom 删除映射（使用持久化的 access_token）
        mapping_id = current_user.capella_mapping_id
        if mapping_id:
            await capella_service.delete_mapping(mapping_id)
    except Exception as e:
        # 记录日志但继续本地解绑（最终一致性）
        logger.warning(f"Failed to delete Capella mapping: {e}")

    # 再清除本地状态
    current_user.capella_user_id = None
    current_user.capella_username = None
    current_user.capella_mapping_id = None
    current_user.capella_access_token = None
    current_user.capella_refresh_token = None
    current_user.capella_token_expires_at = None
    await db.commit()
    return {"status": "unbound"}
```

### P3. 修复事务顺序：DB 先写，API 后调

**问题**: 先调 `create_mapping` API，再 `db.commit()`。API 成功但 DB 失败时 CapellaRoom 侧有孤儿数据。

**方案**: 使用本地事务先记录绑定意图，API 成功后标记完成：

```python
# 伪代码：使用两阶段
# 1. 先写入 DB（标记 pending 状态）
current_user.capella_bind_state = "pending"
current_user.capella_pending_external_id = str(current_user.id)
await db.flush()

# 2. 调 CapellaRoom
mapping = await capella_service.create_mapping(...)

# 3. 更新为已完成
current_user.capella_user_id = userinfo["sub"]
current_user.capella_username = userinfo["username"]
current_user.capella_mapping_id = mapping["id"]
current_user.capella_bind_state = "completed"
await db.commit()
```

简化方案：先调 CapellaRoom API，若失败则本地不回滚（返回错误）；若成功再写 DB。但更可靠的方案是使用 Saga 模式——本地 DB 写入和 CapellaRoom API 调用之间，若 DB commit 失败，自动发起补偿（调用 delete_mapping）。

### P4. TOCTOU 竞态防护

**问题**: `if current_user.capella_user_id` 检查与后续操作之间存在竞态。

**方案**: 在回调末尾 commit 前二次验证：

```python
@router.get("/api/v1/auth/capella/callback")
async def bind_callback(...):
    # ... state 验证 ...

    # 二次验证（事务内）
    fresh_user = await db.get(User, current_user.id)
    if fresh_user.capella_user_id:
        raise HTTPException(status_code=409, detail="Already bound (detected before commit)")

    # ... 调 CapellaRoom API ...

    current_user.capella_user_id = userinfo["sub"]
    await db.commit()
```

### P5. 持久化 OAuth token

**问题**: 回调拿到 `access_token` 后丢弃，后续无法调用 CapellaRoom API。

**方案**: User 模型新增字段：

```python
# alembic migration
capella_access_token: str | None
capella_refresh_token: str | None
capella_token_expires_at: datetime | None
capella_mapping_id: str | None  # CapellaRoom 侧 mapping 的 UUID
```

回调时保存：

```python
current_user.capella_access_token = token_data["access_token"]
current_user.capella_refresh_token = token_data.get("refresh_token")
current_user.capella_token_expires_at = datetime.now(timezone.utc) + timedelta(seconds=token_data["expires_in"])
```

### P6. 错误处理

**问题**: `httpx.HTTPStatusError` 未被捕获，FastAPI 返回 500。

**方案**: 在 `CapellaService` 中统一封装异常，区分业务错误和系统错误：

```python
class CapellaError(Exception):
    """CapellaRoom API 调用基类异常"""
    def __init__(self, message: str, status_code: int | None = None, original: Exception | None = None):
        self.status_code = status_code
        self.original = original

class CapellaAuthError(CapellaError): pass
class CapellaConflictError(CapellaError): pass
class CapellaServerError(CapellaError): pass

class CapellaService:
    async def _request(self, method: str, path: str, **kwargs) -> dict:
        try:
            resp = await self._client.request(method, path, **kwargs)
            resp.raise_for_status()
            return resp.json()
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 401:
                raise CapellaAuthError("CapellaRoom 认证失败", 401, e)
            elif e.response.status_code == 409:
                raise CapellaConflictError("CapellaRoom 资源冲突", 409, e)
            else:
                raise CapellaServerError(f"CapellaRoom 返回 {e.response.status_code}", e.response.status_code, e)
        except httpx.RequestError as e:
            raise CapellaServerError("CapellaRoom 不可达", None, e)
```

Controller 中捕获并转为 HTTPException：

```python
try:
    await capella_service.create_mapping(...)
except CapellaAuthError:
    raise HTTPException(401, "CapellaRoom 认证失败，请重新绑定")
except CapellaConflictError as e:
    raise HTTPException(409, "CapellaRoom 映射已存在")
except CapellaServerError as e:
    raise HTTPException(502, f"CapellaRoom 服务异常: {e}")
```

### P7. State 存储：默认使用 Redis 回退走内存

**分析**: 当前 `create_bind_state_store()` 已支持 Redis 降级到内存，架构上没问题。问题在于 Perseus 生产部署 `--workers N` 时需配置 Redis。

**改进**: 在文档和配置示例中明确说明 Redis 的必要性，`MemoryBindStateStore` 仅在开发和测试中使用。

### P8. CapellaService 连接池复用

**问题**: 每个请求创建/销毁 `httpx.AsyncClient`（通过 FastAPI 依赖 `get_capella_service`）。

**方案**: 将 `httpx.AsyncClient` 提升为应用级别单例，`CapellaService` 只接收 `access_token` 作为请求级上下文：

```python
# app.py 或 lifespan 中
_client: httpx.AsyncClient | None = None

async def get_http_client() -> httpx.AsyncClient:
    global _client
    if _client is None:
        _client = httpx.AsyncClient(
            base_url=config.capella.api_base_url,
            limits=httpx.Limits(max_keepalive_connections=20, max_connections=100),
        )
    return _client

class CapellaService:
    def __init__(self, client: httpx.AsyncClient, access_token: str | None = None):
        self._client = client
        self._access_token = access_token
```

### P9. 前端绑定状态刷新

**问题**: `TeamChatView.vue` 仅 `onMounted` 检查状态，回调重定向回来时不刷新。

**方案**:

```vue
<script setup lang="ts">
import { ref, onMounted, onActivated } from 'vue'

// 改用 onActivated + onMounted 双保险
function checkBinding() {
  if (!auth.user) return
  fetch(`${import.meta.env.VITE_API_BASE_URL}/capella/status`, {
    headers: { Authorization: `Bearer ${auth.user.token}` },
  }).then(...)
}

onMounted(checkBinding)
// 组件被 keep-alive 缓存时再次检查
onActivated(checkBinding)
```

若未使用 keep-alive，可在 `router.beforeEach` 或 `router.push('/team-chat')` 前通过 route meta 强制刷新。

### P10. 前端统一 HTTP 客户端

**问题**: 使用原始 `fetch`，无拦截器。

**方案**: 迁移到 `axios`（如果项目中已有）或封装统一 `apiClient`：

```typescript
// src/api/client.ts
import axios from 'axios'
import { useAuthStore } from '@/stores/auth'

const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
})

apiClient.interceptors.request.use(config => {
  const auth = useAuthStore()
  if (auth.user?.token) {
    config.headers.Authorization = `Bearer ${auth.user.token}`
  }
  return config
})

apiClient.interceptors.response.use(
  response => response,
  error => {
    if (error.response?.status === 401) {
      // 处理 token 过期
    }
    return Promise.reject(error)
  }
)
```

### P11. 完善测试

| 测试 | 说明 |
|------|------|
| exchange_code 失败 → 401 | respx mock token endpoint 返回 400/401 |
| get_userinfo 失败 → 502 | respx mock userinfo endpoint 返回 500 |
| create_mapping 失败 → 502 | respx mock mapping endpoint 返回 500 |
| unbind 调用 CapellaRoom | 验证 unbind 时调用了 delete_mapping |
| 并发绑定防护 | 两个请求同时绑定，验证只有一个成功 |
| Token 持久化 | 验证回调后 access_token/refresh_token 正确保存 |

---

## 三、实施顺序

```
Phase 1: CapellaRoom 核心修复
  ├── F1: OAuth token 中间件 + 路由重构（关键路径）
  ├── F2: Token endpoint form-urlencoded 支持（RFC 合规）
  └── F3: CustomEvent WS OAuth 鉴权（安全）
  → 部署 CapellaRoom 到测试环境

Phase 2: Perseus 适配
  ├── P1: 移除自签 JWT，改用 OAuth token（依赖 F1）
  ├── P5: Token 持久化
  ├── P6: 错误处理封装
  ├── P8: 连接池复用
  ├── P9: 前端状态刷新
  └── P10: 统一 HTTP 客户端
  → 端到端验证 OAuth 绑定流程

Phase 3: 数据一致性
  ├── P2: Unbind 通知 CapellaRoom
  ├── P3: 事务顺序修复
  ├── P4: TOCTOU 防护
  └── P7: State store 配置
  → 测试并发和数据一致性场景

Phase 4: CapellaRoom 次要修复
  ├── F4: 缺失 Webhook 事件
  ├── F5: Webhook 重试优化
  ├── F6: 资源绑定 WS 广播
  ├── F7: CustomNotification（按需）
  └── F8: 集成测试

Phase 5: 双方测试完善
  ├── CapellaRoom: 集成测试套件
  └── Perseus: 异常链路测试
```

---

## 四、CapellaConfig 调整

Perseus 侧的 `CapellaSettings` 可以移除 `jwt_secret` 字段（不再需要自签 JWT）：

```python
class CapellaSettings(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="PERSEUS_CAPELLA_")

    client_id: str
    client_secret: str
    authorize_url: str = "http://localhost:3001/oauth/authorize"
    redirect_uri: str = "http://localhost:5173/auth/capella/callback"
    scopes: str = "openid profile email"
    api_base_url: str = "http://localhost:3001"
    # jwt_secret 已移除 — 使用标准 OAuth token 认证
```

---

## 五、Notable Risks

1. **向后兼容**: CapellaRoom F1 重构路由结构时，需确保现有 CapellaRoom 前端（调用 `/oauth/apps` 等 API 的页面）不受影响。建议在 `oauth_or_app_auth_middleware` 中保持对 CapellaRoom JWT 的支持。
2. **Token 刷新**: Perseus P5 中，若 `access_token` 过期，需要使用 `refresh_token`。当前 CapellaRoom `/oauth/token` 支持 `refresh_token` grant_type，Perseus 需要实现自动刷新逻辑。
3. **WS OAuth**: F3 是最复杂的修改。简化方案：初期可以让 Perseus 服务端通过 HTTP API（`POST /rooms/:room_id/custom-events`，已有实现）发送自定义事件，而非直接通过 WS。HTTP API 已经走 `auth_middleware`，直接集成 F1 后即可支持 OAuth token。

---

## 六、验收标准

| 场景 | 预期结果 |
|------|---------|
| Perseus 发起 OAuth 绑定流程 | 跳转到 CapellaRoom 授权页，用户确认后回跳 |
| Perseus 回调使用 form-urlencoded 交换 token | 200 + 正常返回 token 对 |
| Perseus 用 access_token 调用 `/oauth/mappings` | 201 + mapping 创建成功 |
| Perseus unbind 调用 CapellaRoom | CapellaRoom 侧 mapping 被删除 |
| 绑定流程中 Perseus DB 事务失败 | CapellaRoom 侧 mapping 被自动删除 |
| Webhook 重试达到最大次数 | 标记 failed，不再重试 |
| CustomEvent WS 来自未认证客户端 | 被拒绝 |
