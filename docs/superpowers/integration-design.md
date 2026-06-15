# 集成层详细设计

> **关联**: Perseus Phase 0 (P-001~P-004), Phase 2 (F-201, F-203, F-205)
> **状态**: 设计阶段

---

## 目录

- [一、自定义 WS 事件类型](#一自定义-ws-事件类型)
- [二、房间-资源绑定](#二房间-资源绑定)
- [三、出站 Webhook](#三出站-webhook)
- [四、数据流总图](#四数据流总图)

---

## 一、自定义 WS 事件类型

### 目标

Perseus 等外部服务可以通过 CapellaRoom 的 WebSocket 通道，将 PR/Issue/Review 等业务事件**透传给前端用户**，无需自己维护 WebSocket 连接。

### 设计原则

- CapellaRoom 不做业务理解——只做**中继**（验证发送者权限 → 广播到房间）
- 事件名使用 `{app_name}:{event_type}` 命名空间防冲突
- 复用现有房间的成员关系作为权限模型

### 1.1 协议扩展

在 `WebSocketMessage` 枚举中新增两个变体：

```rust
// ===== 外部服务 → 客户端（通过 CapellaRoom 中继） =====

/// 外部服务发送自定义事件（需要 OAuth 身份）
#[serde(rename = "CustomEvent")]
CustomEvent {
    event_name: String,             // "perseus:pr_updated"
    room_id: Uuid,                  // 广播目标房间
    data: serde_json::Value,        // 透传的业务数据
    persistent: Option<bool>,       // 是否持久化（可选，默认 false）
}

/// 外部服务发送自定义通知给指定用户
#[serde(rename = "CustomNotification")]
CustomNotification {
    event_name: String,             // "perseus:review_requested"
    target_user_id: Uuid,           // 目标用户
    title: String,                  // 通知标题
    body: String,                   // 通知正文
    data: Option<serde_json::Value>,// 透传数据
}


// ===== 客户端接收（CapellaRoom → 客户端） =====

/// 转发给客户端的自定义事件
#[serde(rename = "CustomEventForward")]
CustomEventForward {
    event_name: String,
    room_id: Uuid,
    source_app: String,             // "perseus"
    data: serde_json::Value,
    timestamp: DateTime<Utc>,
}

/// 转发给客户端的自定义通知
#[serde(rename = "CustomNotificationForward")]
CustomNotificationForward {
    event_name: String,
    source_app: String,
    title: String,
    body: String,
    data: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
}
```

### 1.2 JSON 协议示例

```json
// Perseus 服务端 → CapellaRoom WS（服务端对服务端）
// （通过已认证的服务端 WebSocket 连接或 HTTP API 发送）
{
  "type": "CustomEvent",
  "payload": {
    "event_name": "perseus:pr_merged",
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "data": {
      "pr_id": 42,
      "title": "Fix login timeout",
      "merged_by": "alice",
      "target_branch": "main"
    },
    "persistent": true
  }
}

// CapellaRoom → 房间内所有客户端
{
  "type": "CustomEventForward",
  "payload": {
    "event_name": "perseus:pr_merged",
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "source_app": "perseus",
    "data": {
      "pr_id": 42,
      "title": "Fix login timeout",
      "merged_by": "alice",
      "target_branch": "main"
    },
    "timestamp": "2026-06-15T10:30:00Z"
  }
}
```

### 1.3 权限控制

| 发送者 | 可发事件 | 条件 |
|--------|---------|------|
| OAuth 认证的服务端 | `CustomEvent` | 应用已注册 + 事件名以 `{app_name}:` 开头 |
| 普通用户 | 不可发送 | 无权限 |
| Admin | 可强行广播 | 管理员端点，可发任意事件名 |

### 1.4 可选持久化

当 `persistent: true` 时，将事件写入 `custom_events` 表，客户端重连时可拉取遗漏事件：

```sql
CREATE TABLE custom_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_name  VARCHAR(128) NOT NULL,
    room_id     UUID NOT NULL REFERENCES rooms(id),
    source_app  VARCHAR(64) NOT NULL,
    data        JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 用于重连补拉
    INDEX idx_custom_events_room_created (room_id, created_at DESC)
);
```

### 1.5 重连补拉

在现有 `GetMissedMessages` 流程中扩展，新增 WS 消息：

```rust
#[serde(rename = "GetMissedCustomEvents")]
GetMissedCustomEvents {
    room_id: Uuid,
    since: DateTime<Utc>,
}

#[serde(rename = "MissedCustomEvents")]
MissedCustomEvents {
    room_id: Uuid,
    events: Vec<CustomEventRecord>,
    has_more: bool,
}
```

### 1.6 Handler 改动

在 `handle_message` 中新增一个匹配分支：

```rust
WebSocketMessage::CustomEvent { event_name, room_id, data, persistent } => {
    // 1. 验证发送者身份：必须是通过 OAuth 连接的服务
    let app_id = ensure_oauth_service(&state, &user_id)?;

    // 2. 验证事件名命名空间
    ensure_event_namespace(&event_name, &app_id)?;

    // 3. 验证发送者在房间内有权限
    let is_member = state.room_service().is_user_in_room(room_id, user_id).await?;
    ensure(is_member, "not a room member")?;

    // 4. 可选持久化
    if persistent.unwrap_or(false) {
        state.custom_event_service().store_event(&event_name, room_id, &app_id, &data).await?;
    }

    // 5. 构造转发消息广播到房间
    let forward = WebSocketMessage::CustomEventForward {
        event_name,
        room_id,
        source_app: app_id.to_string(),
        data,
        timestamp: Utc::now(),
    };

    let json = forward.to_json()?;
    state.ws_manager().broadcast_to_room(room_id, json, None).await;
}
```

---

## 二、房间-资源绑定

### 目标

将 CapellaRoom 的房间与 Perseus 的业务资源（仓库、PR、Issue）关联起来，使房间拥有"业务上下文"——前端可以在房间中直接跳转到关联的 PR/Issue，也能根据资源 ID 反查房间。

### 2.1 数据模型

```sql
CREATE TABLE room_resource_bindings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id         UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    resource_type   VARCHAR(64) NOT NULL,
    -- 如: "repository", "pull_request", "issue", "wiki"
    resource_id     VARCHAR(255) NOT NULL,
    -- Perseus 侧的业务资源 ID
    resource_url    TEXT,
    -- Perseus 侧的资源详情页 URL
    resource_name   VARCHAR(255),
    -- 资源名称缓存（方便显示，无需回查 Perseus）
    metadata        JSONB DEFAULT '{}',
    -- 扩展字段，如仓库语言、PR 状态等

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL REFERENCES users(id),

    UNIQUE (app_id, resource_type, resource_id),
    -- 一种资源只能绑定一个房间
    INDEX idx_bindings_room (room_id),
    INDEX idx_bindings_resource (app_id, resource_type, resource_id)
);
```

### 2.2 HTTP API

都需认证 + OAuth 应用身份：

```rust
// ===== 绑定管理 =====

/// 绑定：将仓库/PR 关联到聊天室
POST /api/v1/rooms/:room_id/resources
Request: {
    "app_id": "uuid",                    // OAuth 应用 ID
    "resource_type": "repository",        // 资源类型
    "resource_id": "42",                  // Perseus 中的资源 ID
    "resource_url": "https://perseus.local/alice/foo",  // 可选
    "resource_name": "alice/foo",         // 可选，显示用
    "metadata": { "language": "Rust" }   // 可选扩展
}
Response: { "id": "uuid", ... }

/// 解绑
DELETE /api/v1/rooms/:room_id/resources/:id

/// 查询房间的所有绑定
GET /api/v1/rooms/:room_id/resources

/// 反查：根据资源定位房间
GET /api/v1/resources/lookup?app_id=uuid&resource_type=repository&resource_id=42
Response: { "room_id": "uuid", "room_name": "...", ... }

/// 更新绑定元数据（如 PR 状态变化时同步）
PATCH /api/v1/rooms/:room_id/resources/:id
Request: { "resource_url": "...", "resource_name": "...", "metadata": {...} }
```

### 2.3 WS 事件扩展

当资源绑定创建/更新/删除时，向房间广播：

```rust
#[serde(rename = "RoomResourceBound")]
RoomResourceBound { room_id, resource_type, resource_id, resource_name, resource_url }

#[serde(rename = "RoomResourceUnbound")]
RoomResourceUnbound { room_id, resource_type, resource_id }

#[serde(rename = "RoomResourceUpdated")]
RoomResourceUpdated { room_id, resource_type, resource_id, resource_name, resource_url, metadata }
```

### 2.4 典型使用场景

```
Perseus 创建仓库 "alice/foo" 时：
  1. 自动调用 CapellaRoom POST /api/v1/rooms/:room_id/resources
     → 将仓库绑定到项目的聊天室
  2. 前端聊天组件通过 GET /api/v1/rooms/:room_id/resources
     → 显示"当前聊天关联：alice/foo"
     → 显示"仓库"图标 + 可点击链接

PR #42 创建时：
  1. Perseus 绑定 PR 到 PR 专用的临时聊天室
  2. 通过自定义 WS 事件推送 PR 变更
  3. 聊天室 UI 显示 "PR #42: Fix login bug" 标题栏
```

### 2.5 RoomService 扩展

```rust
impl RoomService {
    pub async fn bind_resource(
        &self, room_id: Uuid, app_id: Uuid,
        resource_type: &str, resource_id: &str,
        resource_url: Option<&str>, resource_name: Option<&str>,
        metadata: Option<serde_json::Value>, created_by: Uuid,
    ) -> Result<RoomResourceBinding>;

    pub async fn unbind_resource(&self, binding_id: Uuid, user_id: Uuid) -> Result<()>;

    pub async fn get_room_resources(&self, room_id: Uuid) -> Result<Vec<RoomResourceBinding>>;

    pub async fn lookup_room_by_resource(
        &self, app_id: Uuid, resource_type: &str, resource_id: &str,
    ) -> Result<Option<Room>>;

    pub async fn update_resource_binding(
        &self, binding_id: Uuid,
        resource_url: Option<&str>,
        resource_name: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()>;
}
```

---

## 三、出站 Webhook

### 目标

CapellaRoom 将事件（新消息、用户状态变更、@提及等）通过 HTTP POST 推送到 Perseus 等服务端，让 Perseus 可以在**不依赖 WebSocket** 的情况下接收实时事件并触发业务逻辑（如 CI/CD、邮件通知等）。

### 3.1 数据模型

```sql
-- Webhook 订阅
CREATE TABLE webhook_subscriptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    url             TEXT NOT NULL,
    -- 回调 URL，如 "https://perseus.local/api/webhooks/capella"
    secret          VARCHAR(128) NOT NULL,
    -- HMAC-SHA256 签名密钥（加密存储）
    events          TEXT[] NOT NULL,
    -- 订阅的事件类型，见下方列表
    is_active       BOOLEAN NOT NULL DEFAULT true,
    max_retries     INT NOT NULL DEFAULT 3,
    retry_interval_secs INT NOT NULL DEFAULT 10,
    timeout_ms      INT NOT NULL DEFAULT 5000,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Webhook 投递记录
CREATE TABLE webhook_deliveries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES webhook_subscriptions(id) ON DELETE CASCADE,
    event_type      VARCHAR(64) NOT NULL,
    event_id        UUID NOT NULL,
    -- 事件唯一 ID（用于幂等）
    payload         JSONB NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'success', 'failed', 'cancelled')),
    http_status     INT,
    response_body   TEXT,
    attempt_count   INT NOT NULL DEFAULT 0,
    next_retry_at   TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    INDEX idx_deliveries_status (status, next_retry_at)
    WHERE status = 'pending' OR status = 'failed',
    INDEX idx_deliveries_sub (subscription_id, created_at DESC)
);
```

### 3.2 Webhook 事件类型

| 事件名 | 触发时机 | 示例 Payload |
|--------|---------|-------------|
| `message.created` | 新消息发送 | `{ message_id, room_id, sender_id, content, message_type, created_at }` |
| `message.edited` | 消息编辑 | `{ message_id, room_id, new_content, edited_at }` |
| `message.deleted` | 消息删除 | `{ message_id, room_id }` |
| `room.user_joined` | 用户加入房间 | `{ room_id, user_id, username, joined_at }` |
| `room.user_left` | 用户离开房间 | `{ room_id, user_id, username }` |
| `user.status_changed` | 用户状态切换 | `{ user_id, username, old_status, new_status }` |
| `user.mentioned` | @提及 | `{ message_id, room_id, mentioned_user_id, mentioned_by, content_preview }` |
| `reaction.added` | 表情反应添加 | `{ message_id, room_id, user_id, emoji }` |
| `reaction.removed` | 表情反应移除 | `{ message_id, room_id, user_id, emoji }` |

### 3.3 WebhookService

新建 `WebhookService`，遵循现有服务模式：

```rust
pub struct WebhookService {
    db: Database,
    http_client: reqwest::Client,
}

impl WebhookService {
    pub fn new(db: Database) -> Self;

    // ===== 订阅管理 =====

    pub async fn create_subscription(
        &self, app_id: Uuid, url: &str, secret: &str,
        events: &[&str],
    ) -> Result<WebhookSubscription>;

    pub async fn update_subscription(
        &self, sub_id: Uuid, app_id: Uuid,
        url: Option<&str>, secret: Option<&str>,
        events: Option<&[&str]>, is_active: Option<bool>,
    ) -> Result<()>;

    pub async fn delete_subscription(&self, sub_id: Uuid, app_id: Uuid) -> Result<()>;

    pub async fn get_subscriptions(&self, app_id: Uuid) -> Result<Vec<WebhookSubscription>>;

    // ===== 事件投递 =====

    /// 核心方法：根据事件类型找到所有匹配的订阅并发起投递
    pub async fn dispatch_event(
        &self, event_type: &str, payload: serde_json::Value,
    ) -> Result<usize>;  // 返回匹配的订阅数

    // ===== 投递记录 =====

    pub async fn get_deliveries(
        &self, sub_id: Uuid, app_id: Uuid,
        limit: i64, offset: i64,
    ) -> Result<Vec<WebhookDelivery>>;

    pub async fn redeliver(
        &self, delivery_id: Uuid, sub_id: Uuid, app_id: Uuid,
    ) -> Result<()>;

    // ===== 内部 =====

    async fn send_webhook(
        &self, sub: &WebhookSubscription, payload: &serde_json::Value,
    ) -> Result<WebhookResult>;

    async fn retry_failed_deliveries(&self) -> Result<usize>;
}
```

### 3.4 签名机制

```rust
fn sign_payload(secret: &str, timestamp: i64, body: &str) -> String {
    let msg = format!("{}.{}", timestamp, body);
    let hmac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    let result = hmac.update(msg.as_bytes());
    hex::encode(result.finalize().into_bytes())
}
```

HTTP 请求头：

```
POST /api/webhooks/capella HTTP/1.1
Content-Type: application/json
X-Capella-Signature: sha256=<hex_hmac>
X-Capella-Timestamp: 1718461800
X-Capella-Event-Type: message.created
X-Capella-Delivery-Id: <uuid>        ← 用于幂等
X-Capella-Attempt: 1

{
  "event": "message.created",
  "event_id": "uuid",
  "app_id": "uuid",
  "timestamp": "2026-06-15T10:30:00Z",
  "payload": { ... }
}
```

### 3.5 重试策略

```
第 1 次失败 → 等待 retry_interval_secs (默认 10s) → 第 2 次
第 2 次失败 → 等待 *2 → 第 3 次
第 N 次失败 → 等待 *2 → 最多 max_retries (默认 3)
超过 max_retries → 标记投递状态为 failed
```

后台启动一个定时任务检查 `pending` 和 `failed` 的投递：

```rust
// 在 main.rs 的启动流程中
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(e) = webhook_service.retry_failed_deliveries().await {
            tracing::error!(error = %e, "webhook retry failed");
        }
    }
});
```

### 3.6 在现有流程中插入 Webhook

以发送消息为例，在 `handle_chat_message` 的末尾追加：

```rust
// 原有的：广播 + 持久化
// ...

// 新增的：触发 Webhook
let wh_payload = serde_json::json!({
    "message_id": message_id,
    "room_id": room_id,
    "sender_id": sender_id,
    "sender_name": username,
    "content": content,
    "message_type": "text",
    "created_at": Utc::now()
});

// 异步触发，不阻塞主流程
tokio::spawn(async move {
    if let Err(e) = state.webhook_service().dispatch_event("message.created", wh_payload).await {
        tracing::warn!(error = %e, "webhook dispatch failed");
    }
});
```

同理，在所有关键事件点插入 Webhook 调用：
- `handle_chat_message` → `message.created`
- `handle_edit_message` → `message.edited`
- `handle_delete_message` → `message.deleted`
- `handle_join_room` → `room.user_joined`
- `handle_update_status` → `user.status_changed`
- `send_mention` → `user.mentioned`

### 3.7 HTTP API

```rust
// ===== 订阅管理 (需要 OAuth 身份) =====
POST   /api/v1/webhooks                         // 创建订阅
GET    /api/v1/webhooks                         // 订阅列表
PUT    /api/v1/webhooks/:id                     // 更新订阅
DELETE /api/v1/webhooks/:id                     // 删除订阅

// ===== 投递记录 =====
GET    /api/v1/webhooks/:id/deliveries          // 投递历史
GET    /api/v1/webhooks/:id/deliveries/:did     // 投递详情
POST   /api/v1/webhooks/:id/deliveries/:did/redeliver  // 重试
```

---

## 四、数据流总图

```
┌───────────────────────────────────────────────────────────────────┐
│                         Perseus 后端                               │
│                                                                   │
│  ┌──────────┐  ┌──────────────┐  ┌───────────────────────────┐   │
│  │ PR Merge │  │ New Issue    │  │ Webhook Receiver           │   │
│  │ Event    │  │ Event        │  │ POST /api/webhooks/capella │   │
│  └────┬─────┘  └──────┬───────┘  └────────────┬──────────────┘   │
│       │               │                        ▲                  │
│       │         ┌─────▼────────────────────────┘                  │
│       │         │  自定义事件推送 (CustomEvent WS)                 │
│       │         │  或 资源绑定 API                                │
│       ▼         ▼                                                │
└───────┴─────────┴────────────────────────────────────────────────┘
        │         │
        │         │
        ▼         ▼
┌───────────────────────────────────────────────────────────────────┐
│                      CapellaRoom                                  │
│                                                                   │
│  ┌─────────┐   ┌──────────────┐   ┌──────────────┐               │
│  │ WS      │   │ Webhook      │   │ Resource     │               │
│  │ Handler │──▶│ Service      │──▶│ Binding API  │               │
│  │         │   │              │   │              │               │
│  │ match { │   │ dispatch()   │   │ bind()       │               │
│  │   Ping  │   │  ↓           │   │ lookup()     │               │
│  │   Msg   │   │ HTTP POST    │   │              │               │
│  │   Cust  │   │  → Perseus   │   └───────┬──────┘               │
│  │ }       │   └──────────────┘           │                      │
│  └────┬────┘                              │                      │
│       │                                   │                      │
│       ▼                                   ▼                      │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │              WebSocket 广播                              │     │
│  │  CustomEventForward → 房间内所有在线客户端               │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
        │
        │  WS: CustomEventForward / NewMessage / UserStatusChanged
        ▼
┌───────────────────────────────────────────────────────────────────┐
│                   浏览器 (Vue 3 前端)                              │
│                                                                   │
│  Perseus 页面中嵌入的 CapellaRoom 聊天组件：                       │
│  ┌─────────────────────────────────────┐                         │
│  │  #repo-alice-foo 聊天室             │                         │
│  │                                     │                         │
│  │  [PR #42 merged by alice]    ← CustomEventForward             │
│  │  > LGTM!                     ← NewMessage                     │
│  │                                     │                         │
│  │  绑定: alice/foo (仓库)      ← ResourceBound                  │
│  └─────────────────────────────────────┘                         │
└───────────────────────────────────────────────────────────────────┘
```

---

## 附：与现有模式的对比

| | NotificationService | WebhookService | CustomEvent |
|---|---|---|---|
| **方向** | CapellaRoom → 同一用户 | CapellaRoom → 外部服务 | 外部服务 → 房间用户 |
| **通道** | WebSocket | HTTP POST | WebSocket |
| **接收方** | 同一系统内的用户 | 注册了 Webhook 的外部服务 | 房间内的所有用户 |
| **典型场景** | @提及通知 | Perseus 收到消息后触发 CI | PR 变更推送到聊天室 |
| **持久化** | DB + WS 双写 | 无（投递记录仅用于追踪） | 可选持久化 |
