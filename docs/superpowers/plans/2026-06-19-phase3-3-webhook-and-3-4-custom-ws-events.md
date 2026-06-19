# Phase 3.3 + 3.4: Outbound Webhooks & Custom WS Events

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement outbound webhook delivery (3.3) and custom WebSocket event relay (3.4) for CapellaRoom v2.

**Architecture:** Two independent subsystems sharing OAuth app identity:
- **Webhook (3.3):** `WebhookService` manages subscriptions + delivery with HMAC-SHA256 signing, retry with exponential backoff, and a background retry task. Events dispatched from existing WS handlers via `tokio::spawn`.
- **Custom WS Events (3.4):** Extend `WebSocketMessage` enum with `CustomEvent`/`CustomEventForward` variants. External services send via authenticated WS or HTTP endpoint; CapellaRoom validates namespace + room membership, then broadcasts.

**Tech Stack:** Rust, Axum 0.7, sqlx 0.8, reqwest 0.11, hmac 0.12, sha2 0.10, hex 0.4, chrono, uuid, serde_json

**Design Spec:** `docs/superpowers/integration-design.md` sections 一 (Custom WS Events) + 三 (Outbound Webhooks)

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `migrations/021_webhook_tables.sql` | Create | webhook_subscriptions + webhook_deliveries tables |
| `migrations/022_custom_events.sql` | Create | custom_events table (persistent custom events) |
| `src/models/webhook.rs` | Create | WebhookSubscription, WebhookDelivery, DTOs |
| `src/models/custom_event.rs` | Create | CustomEventRecord, DTOs |
| `src/services/webhook_service.rs` | Create | Subscription CRUD, dispatch, signing, retry |
| `src/services/custom_event_service.rs` | Create | Store + retrieve persistent custom events |
| `src/handlers/webhook.rs` | Create | Webhook subscription + delivery HTTP handlers |
| `src/handlers/custom_event.rs` | Create | HTTP API fallback for custom events |
| `src/websocket/protocol.rs` | Modify | Add CustomEvent, CustomEventForward, GetMissedCustomEvents, MissedCustomEvents variants |
| `src/websocket/handler.rs` | Modify | Add handle_custom_event dispatch in handle_message |
| `src/config/mod.rs` | Modify | Add WebhookConfig (retry settings, timeout) |
| `src/state/mod.rs` | Modify | Add webhook_service + custom_event_service to AppState |
| `src/handlers/mod.rs` | Modify | Add pub mod webhook, custom_event |
| `src/lib.rs` | Modify | Add pub mod models::webhook, models::custom_event |
| `src/routes/mod.rs` | Modify | Register webhook + custom event routes |
| `src/main.rs` | Modify | Start background retry task |

---

## Task Group A: Outbound Webhooks (3.3)

### Task A1: Database Migration — Webhook Tables

**Files:**
- Create: `migrations/021_webhook_tables.sql`

- [x] **Step 1: Create migration**

```sql
-- 021_webhook_tables.sql
CREATE TABLE webhook_subscriptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    url             TEXT NOT NULL,
    secret          VARCHAR(128) NOT NULL,
    events          TEXT[] NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    max_retries     INT NOT NULL DEFAULT 3,
    retry_interval_secs INT NOT NULL DEFAULT 10,
    timeout_ms      INT NOT NULL DEFAULT 5000,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE webhook_deliveries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES webhook_subscriptions(id) ON DELETE CASCADE,
    event_type      VARCHAR(64) NOT NULL,
    event_id        UUID NOT NULL,
    payload         JSONB NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'success', 'failed', 'cancelled')),
    http_status     INT,
    response_body   TEXT,
    attempt_count   INT NOT NULL DEFAULT 0,
    next_retry_at   TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deliveries_pending ON webhook_deliveries(status, next_retry_at)
    WHERE status = 'pending' OR status = 'failed';
CREATE INDEX idx_deliveries_sub ON webhook_deliveries(subscription_id, created_at DESC);
```

- [x] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`
Expected: No errors (sqlx embeds migrations at compile time)

- [x] **Step 3: Commit**

```bash
git add migrations/021_webhook_tables.sql
git commit -m "feat: add webhook_subscriptions and webhook_deliveries migration"
```

---

### Task A2: Models — Webhook DTOs

**Files:**
- Create: `src/models/webhook.rs`
- Modify: `src/models/mod.rs` — add `pub mod webhook;`

- [x] **Step 1: Create webhook models**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ─── DB Models ───

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct WebhookSubscription {
    pub id: Uuid,
    pub app_id: Uuid,
    pub url: String,
    #[serde(skip_serializing)]
    pub secret: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub max_retries: i32,
    pub retry_interval_secs: i32,
    pub timeout_ms: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub event_id: Uuid,
    pub payload: serde_json::Value,
    pub status: String,
    pub http_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempt_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ─── Request DTOs ───

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateWebhookSubscriptionRequest {
    pub url: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebhookSubscriptionRequest {
    pub url: Option<String>,
    pub secret: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

// ─── Response DTOs ───

#[derive(Debug, Clone, Serialize)]
pub struct WebhookSubscriptionResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub max_retries: i32,
    pub retry_interval_secs: i32,
    pub timeout_ms: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WebhookSubscription> for WebhookSubscriptionResponse {
    fn from(s: WebhookSubscription) -> Self {
        Self {
            id: s.id,
            app_id: s.app_id,
            url: s.url,
            events: s.events,
            is_active: s.is_active,
            max_retries: s.max_retries,
            retry_interval_secs: s.retry_interval_secs,
            timeout_ms: s.timeout_ms,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookDeliveryResponse {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub event_id: Uuid,
    pub payload: serde_json::Value,
    pub status: String,
    pub http_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempt_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<WebhookDelivery> for WebhookDeliveryResponse {
    fn from(d: WebhookDelivery) -> Self {
        Self {
            id: d.id,
            subscription_id: d.subscription_id,
            event_type: d.event_type,
            event_id: d.event_id,
            payload: d.payload,
            status: d.status,
            http_status: d.http_status,
            response_body: d.response_body,
            attempt_count: d.attempt_count,
            next_retry_at: d.next_retry_at,
            completed_at: d.completed_at,
            created_at: d.created_at,
        }
    }
}
```

- [x] **Step 2: Add module declaration**

In `src/models/mod.rs`, add:
```rust
pub mod webhook;
```

- [x] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 4: Commit**

```bash
git add src/models/webhook.rs src/models/mod.rs
git commit -m "feat: add webhook subscription and delivery models"
```

---

### Task A3: Config — WebhookConfig

**Files:**
- Modify: `src/config/mod.rs`

- [x] **Step 1: Add WebhookConfig struct**

After `OAuthConfig`:

```rust
/// Webhook 配置
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    /// 后台重试扫描间隔（秒），默认 30
    #[serde(default = "default_retry_scan_interval")]
    pub retry_scan_interval_secs: u64,
    /// 默认 HTTP 超时（毫秒），默认 5000
    #[serde(default = "default_webhook_timeout_ms")]
    pub default_timeout_ms: u64,
    /// 默认最大重试次数，默认 3
    #[serde(default = "default_max_retries")]
    pub default_max_retries: i32,
}

fn default_retry_scan_interval() -> u64 { 30 }
fn default_webhook_timeout_ms() -> u64 { 5000 }
fn default_max_retries() -> i32 { 3 }

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            retry_scan_interval_secs: 30,
            default_timeout_ms: 5000,
            default_max_retries: 3,
        }
    }
}
```

- [x] **Step 2: Add `pub webhook: WebhookConfig` to AppConfig**

After the `oauth` field in `AppConfig`:
```rust
    #[serde(default)]
    pub webhook: WebhookConfig,
```

- [x] **Step 3: Add `[webhook]` to config.toml**

```toml
# -----------------------------------------------------------------------------
# Webhook 配置
# -----------------------------------------------------------------------------
[webhook]
# 后台重试扫描间隔（秒）
retry_scan_interval_secs = 30
# 默认 HTTP 超时（毫秒）
default_timeout_ms = 5000
# 默认最大重试次数
default_max_retries = 3
```

- [x] **Step 4: Fix test AppConfig constructors**

Add `webhook: Default::default(),` to all test files that construct `AppConfig`:
- `tests/phase1_infrastructure_test.rs` (4 occurrences)
- `tests/phase1_4_v1_register_admin_test.rs`
- `tests/phase1_5_user_email_verified_test.rs`
- `tests/phase4_websocket_test.rs`
- `tests/phase8_admin_system_test.rs`
- `tests/phase8_4_audit_system_test.rs`
- `tests/phaseC1_delete_account_test.rs`
- `tests/phaseC2_pinned_messages_test.rs`
- `tests/user_settings_test.rs`
- `tests/websocket_test.rs`
- `tests/pending_action_notification_test.rs`

Use sed:
```bash
rg -l 'mail: Default::default' tests/ | while read f; do
  sed -i 's/mail: Default::default(),/mail: Default::default(),\n        webhook: Default::default(),/' "$f"
done
```

Also fix `config_system_test.rs` — add `[webhook]` section to both TOML strings:
```toml
[webhook]
retry_scan_interval_secs = 30
default_timeout_ms = 5000
default_max_retries = 3
```

- [x] **Step 5: Verify compilation + tests**

Run: `cargo check 2>&1 | head -20`
Run: `cargo test --lib 2>&1 | tail -5`

- [x] **Step 6: Commit**

```bash
git add src/config/mod.rs config.toml tests/
git commit -m "feat: add WebhookConfig with defaults and config.toml section"
```

---

### Task A4: Service — WebhookService

**Files:**
- Create: `src/services/webhook_service.rs`
- Modify: `src/services/mod.rs` — add `pub mod webhook_service;`

- [x] **Step 1: Create WebhookService**

Key methods: `create_subscription`, `list_subscriptions`, `update_subscription`, `delete_subscription`, `dispatch_event` (find matching subscriptions, spawn delivery), `deliver_once` (HMAC sign + reqwest POST), `retry_failed_deliveries` (scan + retry), `get_deliveries`, `redeliver`.

Signature:
```rust
pub struct WebhookService {
    db: Database,
    http_client: reqwest::Client,
}
```

The `dispatch_event` method finds all active subscriptions matching the event type, creates a `webhook_deliveries` record with status `pending`, and spawns an async task to deliver.

The `deliver_once` method computes HMAC-SHA256 signature, sets headers (`X-Capella-Signature`, `X-Capella-Timestamp`, `X-Capella-Event-Type`, `X-Capella-Delivery-Id`, `X-Capella-Attempt`), and POSTs to the subscription URL.

The `retry_failed_deliveries` method queries pending/failed deliveries where `next_retry_at <= now()`, attempts delivery, and updates status/expiry on failure with exponential backoff.

HMAC signing:
```rust
fn sign_payload(secret: &str, timestamp: i64, body: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let msg = format!("{}.{}", timestamp, body);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(msg.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

- [x] **Step 2: Add `reqwest` and `hmac` to Cargo.toml**

```toml
# HTTP client for webhooks
reqwest = { version = "0.11", features = ["json"] }

# HMAC signing for webhook payloads
hmac = "0.12"
```

Remove `reqwest` from `[dev-dependencies]` since it's now a regular dependency.

- [x] **Step 3: Add module declaration**

In `src/services/mod.rs`:
```rust
pub mod webhook_service;
```

- [x] **Step 4: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 5: Commit**

```bash
git add src/services/webhook_service.rs src/services/mod.rs Cargo.toml
git commit -m "feat: add WebhookService with subscription CRUD, dispatch, HMAC signing, retry"
```

---

### Task A5: Handlers — Webhook HTTP Endpoints

**Files:**
- Create: `src/handlers/webhook.rs`
- Modify: `src/handlers/mod.rs` — add `pub mod webhook;`

- [x] **Step 1: Create webhook handlers**

Endpoints (all require auth + OAuth app identity):
- `POST /webhooks` — create subscription
- `GET /webhooks` — list subscriptions for app
- `PUT /webhooks/:id` — update subscription
- `DELETE /webhooks/:id` — delete subscription
- `GET /webhooks/:id/deliveries` — delivery history
- `POST /webhooks/:id/redeliver/:did` — retry delivery

Pattern: follow `src/handlers/oauth.rs` style (Extension<Claims>, State<Arc<AppState>>, Json/Form extractors).

- [x] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 3: Commit**

```bash
git add src/handlers/webhook.rs src/handlers/mod.rs
git commit -m "feat: add webhook subscription and delivery HTTP handlers"
```

---

### Task A6: Routes + State — Register Webhook Routes

**Files:**
- Modify: `src/routes/mod.rs`
- Modify: `src/state/mod.rs`

- [x] **Step 1: Add webhook_service to AppState**

In `src/state/mod.rs`:
- Add field: `pub webhook_service: WebhookService`
- Initialize in `AppState::new()` and `Clone` impl
- Add accessor: `pub fn webhook_service(&self) -> &WebhookService { &self.webhook_service }`

- [x] **Step 2: Register routes**

Add to `oauth_protected_routes()` in `src/routes/mod.rs`:
```rust
.route("/webhooks", post(webhook::create_subscription).get(webhook::list_subscriptions))
.route("/webhooks/:id", put(webhook::update_subscription).delete(webhook::delete_subscription))
.route("/webhooks/:id/deliveries", get(webhook::get_deliveries))
.route("/webhooks/:id/redeliver/:did", post(webhook::redeliver))
```

- [x] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 4: Commit**

```bash
git add src/routes/mod.rs src/state/mod.rs
git commit -m "feat: register webhook routes and add WebhookService to AppState"
```

---

### Task A7: Event Dispatch Hooks — Insert Webhook Triggers

**Files:**
- Modify: `src/websocket/handler.rs`

- [x] **Step 1: Add webhook dispatch calls in existing handlers**

At the end of each handler function, add a `tokio::spawn` that dispatches the event:

In `handle_chat_message` (after broadcast):
```rust
let wh_payload = serde_json::json!({
    "message_id": message_id,
    "room_id": room_id,
    "sender_id": sender_id,
    "sender_name": username,
    "content": content,
    "message_type": "text",
    "created_at": Utc::now()
});
let state_clone = Arc::clone(state);
tokio::spawn(async move {
    if let Err(e) = state_clone.webhook_service().dispatch_event("message.created", wh_payload).await {
        warn!("webhook dispatch failed: {}", e);
    }
});
```

Similarly for:
- `handle_edit_message` → `"message.edited"`
- `handle_delete_message` → `"message.deleted"`
- `handle_join_room` → `"room.user_joined"`
- `handle_leave_room` → `"room.user_left"`

- [x] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 3: Commit**

```bash
git add src/websocket/handler.rs
git commit -m "feat: insert webhook dispatch hooks in WS message handlers"
```

---

### Task A8: Background Retry Task

**Files:**
- Modify: `src/main.rs` (or wherever the startup spawns background tasks)

- [x] **Step 1: Spawn retry task at startup**

After AppState creation, before server start:
```rust
let webhook_service = Arc::new(state.webhook_service().clone()); // or Arc clone of state
let retry_interval = config.webhook.retry_scan_interval_secs;
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(retry_interval));
    loop {
        interval.tick().await;
        if let Err(e) = webhook_service.retry_failed_deliveries().await {
            tracing::error!(error = %e, "webhook retry task failed");
        }
    }
});
```

Note: WebhookService may need to be `Clone` (wrap http_client in Arc) to be spawned.

- [x] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: spawn webhook background retry task at startup"
```

---

## Task Group B: Custom WS Events (3.4)

### Task B1: Database Migration — Custom Events

**Files:**
- Create: `migrations/022_custom_events.sql`

- [x] **Step 1: Create migration**

```sql
-- 022_custom_events.sql
CREATE TABLE custom_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_name  VARCHAR(128) NOT NULL,
    room_id     UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    source_app  VARCHAR(64) NOT NULL,
    data        JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_custom_events_room_created ON custom_events(room_id, created_at DESC);
```

- [x] **Step 2: Commit**

```bash
git add migrations/022_custom_events.sql
git commit -m "feat: add custom_events migration for persistent custom events"
```

---

### Task B2: Models — Custom Event DTOs

**Files:**
- Create: `src/models/custom_event.rs`
- Modify: `src/models/mod.rs` — add `pub mod custom_event;`

- [x] **Step 1: Create custom event models**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct CustomEventRecord {
    pub id: Uuid,
    pub event_name: String,
    pub room_id: Uuid,
    pub source_app: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomEventHttpRequest {
    pub event_name: String,
    pub room_id: Uuid,
    pub data: serde_json::Value,
    pub persistent: Option<bool>,
}
```

- [x] **Step 2: Add module declaration**

In `src/models/mod.rs`:
```rust
pub mod custom_event;
```

- [x] **Step 3: Commit**

```bash
git add src/models/custom_event.rs src/models/mod.rs
git commit -m "feat: add custom event record model and HTTP request DTO"
```

---

### Task B3: Service — CustomEventService

**Files:**
- Create: `src/services/custom_event_service.rs`
- Modify: `src/services/mod.rs` — add `pub mod custom_event_service;`

- [x] **Step 1: Create CustomEventService**

```rust
pub struct CustomEventService {
    db: Database,
}

impl CustomEventService {
    pub fn new(db: Database) -> Self { Self { db } }

    pub async fn store_event(
        &self, event_name: &str, room_id: Uuid,
        source_app: &str, data: &serde_json::Value,
    ) -> Result<CustomEventRecord> {
        let record = sqlx::query_as::<_, CustomEventRecord>(
            r#"INSERT INTO custom_events (event_name, room_id, source_app, data)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#
        )
        .bind(event_name)
        .bind(room_id)
        .bind(source_app)
        .bind(data)
        .fetch_one(self.db.pool())
        .await?;
        Ok(record)
    }

    pub async fn get_missed_events(
        &self, room_id: Uuid, since: DateTime<Utc>, limit: i64,
    ) -> Result<Vec<CustomEventRecord>> {
        let events = sqlx::query_as::<_, CustomEventRecord>(
            r#"SELECT * FROM custom_events
               WHERE room_id = $1 AND created_at > $2
               ORDER BY created_at DESC LIMIT $3"#
        )
        .bind(room_id)
        .bind(since)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await?;
        Ok(events)
    }
}
```

- [x] **Step 2: Add module declaration**

- [x] **Step 3: Commit**

```bash
git add src/services/custom_event_service.rs src/services/mod.rs
git commit -m "feat: add CustomEventService for persistent custom events"
```

---

### Task B4: Protocol Extension — WebSocketMessage Variants

**Files:**
- Modify: `src/websocket/protocol.rs`

- [x] **Step 1: Add new variants to WebSocketMessage enum**

After the existing variants, add:

```rust
// ========== 外部服务自定义事件 ==========
/// 外部服务发送自定义事件（需要 OAuth 身份）
CustomEvent {
    event_name: String,
    room_id: Uuid,
    data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    persistent: Option<bool>,
},

/// 转发给客户端的自定义事件
CustomEventForward {
    event_name: String,
    room_id: Uuid,
    source_app: String,
    data: serde_json::Value,
    timestamp: DateTime<Utc>,
},

/// 客户端请求获取错过的自定义事件
GetMissedCustomEvents {
    room_id: Uuid,
    since: DateTime<Utc>,
},

/// 服务端返回错过的自定义事件
MissedCustomEvents {
    room_id: Uuid,
    events: Vec<CustomEventForwardPayload>,
    has_more: bool,
},
```

Add helper struct:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEventForwardPayload {
    pub id: Uuid,
    pub event_name: String,
    pub source_app: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
```

- [x] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 3: Commit**

```bash
git add src/websocket/protocol.rs
git commit -m "feat: add CustomEvent/CustomEventForward/GetMissedCustomEvents WS variants"
```

---

### Task B5: Handler — Custom Event Dispatch in WS

**Files:**
- Modify: `src/websocket/handler.rs`

- [x] **Step 1: Add CustomEvent handling in handle_message**

Add a new match arm after the existing handlers:

```rust
WebSocketMessage::CustomEvent { event_name, room_id, data, persistent } => {
    // 1. Verify event name namespace (must contain ":")
    if !event_name.contains(':') {
        let err = WebSocketMessage::error("INVALID_EVENT_NAME", "Event name must use 'namespace:event' format");
        if let Ok(json) = err.to_json() { let _ = tx.send(json).await; }
        return Ok(());
    }

    // 2. Verify sender is a room member
    let is_member = state.room_service().is_user_in_room(room_id, user_id).await.unwrap_or(false);
    if !is_member {
        let err = WebSocketMessage::error("NOT_IN_ROOM", "Must be a room member to send custom events");
        if let Ok(json) = err.to_json() { let _ = tx.send(json).await; }
        return Ok(());
    }

    // 3. Extract app name from event namespace
    let source_app = event_name.split(':').next().unwrap_or("unknown").to_string();

    // 4. Optionally persist
    if persistent.unwrap_or(false) {
        let state_clone = Arc::clone(state);
        let en = event_name.clone();
        let d = data.clone();
        let sa = source_app.clone();
        tokio::spawn(async move {
            if let Err(e) = state_clone.custom_event_service().store_event(&en, room_id, &sa, &d).await {
                warn!("failed to persist custom event: {}", e);
            }
        });
    }

    // 5. Broadcast CustomEventForward to room
    let forward = WebSocketMessage::CustomEventForward {
        event_name,
        room_id,
        source_app,
        data,
        timestamp: Utc::now(),
    };
    if let Ok(json) = forward.to_json() {
        state.ws_manager().broadcast_to_room(room_id, json, None).await;
    }
}
```

- [x] **Step 2: Add GetMissedCustomEvents handling**

```rust
WebSocketMessage::GetMissedCustomEvents { room_id, since } => {
    let state_clone = Arc::clone(state);
    let uid = user_id;
    tokio::spawn(async move {
        match state_clone.custom_event_service().get_missed_events(room_id, since, 50).await {
            Ok(events) => {
                let payloads: Vec<CustomEventForwardPayload> = events.into_iter().map(|e| {
                    CustomEventForwardPayload {
                        id: e.id,
                        event_name: e.event_name,
                        source_app: e.source_app,
                        data: e.data,
                        timestamp: e.created_at,
                    }
                }).collect();
                let has_more = payloads.len() >= 50;
                let msg = WebSocketMessage::MissedCustomEvents { room_id, events: payloads, has_more };
                if let Ok(json) = msg.to_json() { let _ = tx.send(json).await; }
            }
            Err(e) => {
                warn!("failed to get missed custom events: {}", e);
            }
        }
    });
}
```

- [x] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 4: Commit**

```bash
git add src/websocket/handler.rs
git commit -m "feat: handle CustomEvent dispatch and GetMissedCustomEvents in WS handler"
```

---

### Task B6: HTTP API Fallback — Custom Events Endpoint

**Files:**
- Create: `src/handlers/custom_event.rs`
- Modify: `src/handlers/mod.rs` — add `pub mod custom_event;`
- Modify: `src/routes/mod.rs` — register route

- [x] **Step 1: Create handler**

```rust
pub async fn send_custom_event(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CustomEventHttpRequest>,
) -> Result<Json<ApiResponse<()>>> {
    // Validate event name
    if !request.event_name.contains(':') {
        return Err(AppError::Validation("Event name must use 'namespace:event' format".to_string()));
    }

    // Verify room membership
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("invalid user id".to_string()))?;
    let is_member = state.room_service().is_user_in_room(request.room_id, user_id).await?;
    if !is_member {
        return Err(AppError::Forbidden);
    }

    let source_app = request.event_name.split(':').next().unwrap_or("unknown").to_string();

    // Persist if requested
    if request.persistent.unwrap_or(false) {
        state.custom_event_service().store_event(
            &request.event_name, request.room_id, &source_app, &request.data
        ).await?;
    }

    // Broadcast
    let forward = WebSocketMessage::CustomEventForward {
        event_name: request.event_name,
        room_id: request.room_id,
        source_app,
        data: request.data,
        timestamp: Utc::now(),
    };
    let json = forward.to_json().map_err(|e| AppError::Internal)?;
    state.ws_manager().broadcast_to_room(request.room_id, json, None).await;

    Ok(Json(ApiResponse::success_with_message("custom event dispatched")))
}
```

- [x] **Step 2: Register route**

Add to `oauth_protected_routes()`:
```rust
.route("/events/custom", post(custom_event::send_custom_event))
```

- [x] **Step 3: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 4: Commit**

```bash
git add src/handlers/custom_event.rs src/handlers/mod.rs src/routes/mod.rs
git commit -m "feat: add HTTP API fallback for custom events"
```

---

### Task B7: AppState — Add CustomEventService

**Files:**
- Modify: `src/state/mod.rs`

- [x] **Step 1: Add custom_event_service to AppState**

Same pattern as webhook_service:
- Add field `pub custom_event_service: CustomEventService`
- Initialize in `AppState::new()` and `Clone` impl
- Add accessor `pub fn custom_event_service(&self) -> &CustomEventService`

- [x] **Step 2: Verify compilation**

Run: `cargo check 2>&1 | head -20`

- [x] **Step 3: Commit**

```bash
git add src/state/mod.rs
git commit -m "feat: add CustomEventService to AppState"
```

---

## Final Verification

- [x] **Full build**: `cargo build 2>&1`
- [x] **Unit tests**: `cargo test --lib 2>&1 | tail -10`
- [x] **Integration tests**: `cargo test 2>&1 | grep -E '(FAILED|test result:)'`
- [x] **All commits clean**: `git log --oneline -15`

---

## Summary

| Task | Description | Est. Time |
|------|-------------|-----------|
| A1 | Webhook migration | 0.5h |
| A2 | Webhook models | 0.5h |
| A3 | WebhookConfig | 0.5h |
| A4 | WebhookService | 3h |
| A5 | Webhook handlers | 1.5h |
| A6 | Routes + State | 1h |
| A7 | Event dispatch hooks | 1.5h |
| A8 | Background retry | 0.5h |
| B1 | Custom events migration | 0.5h |
| B2 | Custom event models | 0.5h |
| B3 | CustomEventService | 1h |
| B4 | Protocol extension | 1h |
| B5 | WS handler dispatch | 2h |
| B6 | HTTP API fallback | 1h |
| B7 | AppState update | 0.5h |
| **Total** | | **~15h** |
