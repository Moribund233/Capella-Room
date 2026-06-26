# Workspace 拆分方案

## 目标

将单 crate 拆分为多 crate workspace，实现编译隔离：修改任意 domain 只重编对应 crate（5-15s），无需全量重编（当前 2-3min）。

## 最终目标结构（方案 C）

```
capella-room/              ← workspace root, Cargo.toml ([workspace])
├── capella-types/         ← models + error（纯数据结构，零依赖）
├── capella-infra/         ← config + db + redis + utils
├── capella-core/          ← services（业务逻辑）
├── capella-websocket/     ← websocket 协议 + 连接管理
├── capella-api/           ← handlers + middleware + routes（HTTP 层）
└── capella-room-bin/      ← main.rs + AppState 组装
```

## 当前依赖图

```
models (leaf)    error (leaf)    config (leaf)
    ↓                              ↓
    ├── services ←────── websocket   db/redis (→ config)
    ├── handlers                      ↓
    └── state (hub, 引用所有模块)      routes + middleware
```

## 所有需要解耦的跨域依赖

### services → websocket

| 源 | 依赖目标 | 原因 |
|---|---|---|
| `NotificationService` | `WebSocketManager` | 推送实时通知 |
| `AlertHandler` | `WebSocketMessage`, `NotificationType` | 发送告警消息 |
| `AccountSecurityService` | `NotificationService` + `NotificationType` | 安全事件通知 |

### redis → websocket

| 源 | 依赖目标 | 原因 |
|---|---|---|
| `redis::pubsub` | `WebSocketManager` | Redis 广播 → WebSocket 转发 |

### config → services + websocket

| 源 | 依赖目标 | 原因 |
|---|---|---|
| `config::listener` | `WebSocketManager` | 配置热重载 → 通知 WS 客户端 |
| `config::listener` | `BatchMessageService` | 配置热重载 → 调整批量写入参数 |

### services → services

| 源 | 依赖目标 | 原因 |
|---|---|---|
| `NotificationService` | `UserSettingsService` | 检查用户通知偏好 |
| `AuditService` | `NotificationService` + `AlertEngine` + `AlertHandler` | 审计事件 → 通知 + 告警 |
| `IpSecurityService` | `AuditService` | IP 安全事件 → 审计日志 |
| `VerificationCodeService` | `MailService` | 发送验证码邮件 |
| `AccountSecurityService` | `NotificationService` | 安全事件通知 |

## Trait 抽象方案

所有跨域依赖通过 trait 解耦，trait 定义在 `capella-types` 中，实现在各自 domain crate。

### 1. `NotificationSender` — 核心解耦

```rust
// capella-types/src/notification.rs
#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_notification(&self, user_id: Uuid, notification: NotificationPayload) -> Result<()>;
    async fn broadcast_to_room(&self, room_id: Uuid, message: WsMessage) -> Result<()>;
}

// capella-websocket — 实现
impl NotificationSender for WebSocketManager { ... }

// capella-core — 使用
pub struct NotificationService {
    notifier: Box<dyn NotificationSender>,
    preference: Box<dyn NotificationPreference>,
    ...
}
```

涉及服务：`NotificationService`, `AlertHandler`, `AccountSecurityService`, `AuditService`

### 2. `MailSender` — 最简单，适合练手

```rust
// capella-types/src/mail.rs
#[async_trait]
pub trait MailSender: Send + Sync {
    async fn send_verification_code(&self, email: &str, code: &str) -> Result<()>;
    async fn send_password_reset(&self, email: &str, token: &str) -> Result<()>;
    async fn send_welcome(&self, username: &str, email: &str) -> Result<()>;
}

// capella-core — 实现（当前 MailService 逻辑）
impl MailSender for ConsoleMailSender { ... }
```

涉及服务：`VerificationCodeService`

### 3. `NotificationPreference`

```rust
// capella-types/src/notification.rs
#[async_trait]
pub trait NotificationPreference: Send + Sync {
    async fn should_notify(&self, user_id: Uuid, notif_type: NotificationType) -> Result<bool>;
}
```

涉及服务：`NotificationService` → `UserSettingsService`

### 4. `AuditLogger`

```rust
// capella-types/src/audit.rs
#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log_security_event(&self, event: SecurityEvent) -> Result<()>;
}
```

涉及服务：`IpSecurityService` → `AuditService`

### 5. `WsBroadcaster`

```rust
// capella-types/src/websocket.rs
#[async_trait]
pub trait WsBroadcaster: Send + Sync {
    async fn broadcast(&self, message: &[u8]) -> Result<()>;
}
```

涉及：`redis::pubsub` → `WebSocketManager`

### 6. `ConfigReloadHandler`

```rust
// capella-types/src/config.rs
pub trait ConfigReloadHandler: Send + Sync {
    fn on_config_reload(&self, old: &AppConfig, new: &AppConfig);
}
```

涉及：`config::listener` → `WebSocketManager` + `BatchMessageService`

## 实施步骤

### 阶段一：准备（预计 2-3 天）

1. 新建 workspace root `Cargo.toml`
2. 创建 `capella-types/`，移入 `models/` 和 `error/`
   - 所有 `use crate::models::xxx` → `use capella_types::xxx`
   - 全局替换约 200+ 处
3. 创建 `capella-infra/`，移入 `config/`, `db/`, `redis/`, `utils/`
   - 处理 `db` → `config`, `redis` → `config` 的依赖
4. 创建 `capella-core/`，移入 `services/`
5. 创建 `capella-websocket/`，移入 `websocket/`
6. 创建 `capella-api/`，移入 `handlers/` + `middleware/` + `routes/`
7. `capella-room-bin/` 保留 `main.rs` + `state/`

### 阶段二：定义 trait（预计 1-2 天）

1. 在 `capella-types` 中定义所有 trait
2. 在各个 domain crate 中实现 trait
3. 将 `impl` 改为通过 `Box<dyn Trait>` 注入

### 阶段三：拆分 crate（预计 1 天）

1. 逐个创建子 crate 并移入代码
2. 处理 `pub(crate)` → `pub` 等可见性问题
3. 修正所有 `use` 路径

### 阶段四：组装

1. `capella-room-bin` 中通过依赖注入组装所有服务
2. 移除 `state/mod.rs` 中的中心化 AppState，改为每个 crate 持有自己的状态
3. 编译验证 + 测试

## 风险

- `state/mod.rs` 当前引用所有模块，拆分后需要重构为依赖注入模式，改动最大
- `websocket::handler` 直接通过 `AppState` 获取服务引用，需要改为只注入所需依赖
- 部分 `pub(crate)` 类型需要提升为 `pub`，可能暴露不应公开的内部类型
- 测试中的 `use crate::xxx` 需要全部修正

## 时机建议

当前**不建议立即实施**。理由：
1. target cache + mold 已将增量编译降到 ~0.17s
2. 代码量还未到拆分的临界点（~80 个源文件）
3. 后续业务扩张、团队扩大时再拆分更划算

建议先完成阶段二的 trait 定义（解耦跨域依赖），为拆分做好准备，但不急于物理拆分 crate 目录。
