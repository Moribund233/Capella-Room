# 后端细节修复清单

> 本文档记录后端 API 响应结构中存在的用户体验问题，供阶段9优化时参考。

---

##  0001-后端多处 API 响应只返回 `user_id`（UUID 格式），前端无法直接显示友好的用户名，影响用户体验。

---

## 问题列表

### 1.  ✅ RoomResponse 缺少房主名称 [P1-高优先级] - 已修复

**位置**: `src/models/room.rs` - `RoomResponse` 结构体

**修复方案**:
- 采用统一 UserInfo 结构体方案
- 将 `owner_id: Uuid` 改为 `owner: UserInfo`
- SQL 查询使用 JOIN 获取用户信息

**修复后代码**:
```rust
pub struct RoomResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner: UserInfo,  // ✅ 使用 UserInfo 替代 owner_id
    pub is_private: bool,
    pub max_members: i32,
    pub member_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**影响文件**:
- `src/models/room.rs` - RoomResponse 定义
- `src/models/user.rs` - UserInfo 结构体
- `src/services/room_service.rs` - SQL 查询修改（6处）

**前端同步修改**:
- `client/debug/src/types/api.ts` - Room 接口 owner_id → owner
- `client/debug/src/views/RoomManager.vue` - 显示用户名和头像

---

### 2.  ✅ AuditAlert 缺少用户名称信息 [P2-中优先级] - 已修复

**位置**: `src/models/audit.rs` - `AuditAlertResponse` 结构体

**修复方案**:
- 创建新的 `AuditAlertResponse` DTO 结构体
- 使用 `UserInfo` 替代裸 UUID
- 原 `AuditAlert` 保持数据库模型不变

**修复后代码**:
```rust
pub struct AuditAlertResponse {
    pub id: Uuid,
    pub rule_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: AuditSeverity,
    pub title: String,
    pub description: String,
    pub affected_user: Option<UserInfo>,  // ✅ 使用 UserInfo
    pub status: AlertStatus,
    pub acknowledged_by: Option<UserInfo>,  // ✅ 使用 UserInfo
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<UserInfo>,  // ✅ 使用 UserInfo
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**影响文件**:
- `src/models/audit.rs` - 新增 AuditAlertResponse
- `src/services/audit_service.rs` - SQL 查询修改

**前端同步修改**:
- `client/debug/src/types/api.ts` - 新增 AuditAlert 接口

---

### 3.  ✅ FileResponse 缺少上传者信息 [P2-中优先级] - 已修复

**位置**: `src/models/file.rs` - `FileResponse` 结构体

**修复方案**:
- 添加 `uploader: Option<UserInfo>` 字段
- SQL 查询 LEFT JOIN users 表

**修复后代码**:
```rust
pub struct FileResponse {
    pub id: Uuid,
    pub original_name: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: String,
    pub category: FileCategory,
    pub usage_type: FileUsageType,
    pub uploader: Option<UserInfo>,  // ✅ 新增上传者信息
    pub created_at: DateTime<Utc>,
}
```

**影响文件**:
- `src/models/file.rs` - FileResponse 定义
- `src/services/file_service.rs` - SQL 查询修改

**前端同步修改**:
- `client/debug/src/types/api.ts` - 新增 FileResource 接口

---

### 4.  ✅ 统一 UserInfo 结构体 [P3-低优先级/架构优化] - 已修复

**修复方案**:
创建统一的 `UserInfo` 结构体，并在多处复用：

```rust
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
}
```

**应用范围**:
- ✅ `RoomResponse.owner` - 房主信息
- ✅ `AuditAlertResponse.affected_user` - 受影响用户
- ✅ `AuditAlertResponse.acknowledged_by` - 确认者
- ✅ `AuditAlertResponse.resolved_by` - 解决者
- ✅ `FileResponse.uploader` - 上传者
- ✅ `MessageResponse.sender` - 使用独立的 `SenderInfo`（保持独立，不强制复用）

**设计决策**:
- `SenderInfo` 保持独立结构，不强制复用 `UserInfo`，避免增加代码理解难度
- 两者结构相同但语义不同，便于后续独立演进

**影响文件**:
- `src/models/user.rs` - UserInfo 定义
- `src/models/room.rs` - RoomResponse 使用 UserInfo
- `src/models/audit.rs` - AuditAlertResponse 使用 UserInfo
- `src/models/file.rs` - FileResponse 使用 UserInfo
- `src/models/message.rs` - SenderInfo（独立但结构相同）

---

## 修复工作量评估（实际）

| 问题 | 影响文件数 | SQL查询修改 | 实际工作量 | 状态 |
|------|-----------|------------|-----------|------|
| RoomResponse 添加 owner | 3 | 6处 | 中等 | ✅ 完成 |
| AuditAlert 添加用户名称 | 2 | 2处 | 小 | ✅ 完成 |
| FileResponse 添加上传者 | 2 | 2处 | 小 | ✅ 完成 |
| 统一 UserInfo 结构体 | 5+ | 多处 | 较大 | ✅ 完成 |
| 前端类型同步修改 | 5 | - | 中等 | ✅ 完成 |

---

## 临时解决方案

在修复完成前，前端可采用以下方式处理：
1. UUID 截断显示：只显示前8位，如 `55bea4ba...`
2. 悬停提示：鼠标悬停时显示完整UUID
3. 异步加载：点击后单独查询用户信息

**状态**: 已修复，前端现在可以直接显示友好的用户名和头像。

---

## 相关文件清单

### 后端
- `src/models/user.rs` - UserInfo 定义
- `src/models/room.rs` - RoomResponse 定义
- `src/models/audit.rs` - AuditAlertResponse 定义
- `src/models/file.rs` - FileResponse 定义
- `src/models/message.rs` - SenderInfo 定义
- `src/services/room_service.rs` - 房间查询逻辑
- `src/services/audit_service.rs` - 审计查询逻辑
- `src/services/file_service.rs` - 文件查询逻辑

### 前端
- `client/debug/src/types/api.ts` - API 类型定义
- `client/debug/src/api/message.ts` - Message 类型
- `client/debug/src/api/index.ts` - 类型导出
- `client/debug/src/views/RoomManager.vue` - 房间管理组件
- `client/debug/src/views/MessageTest.vue` - 消息测试组件

---

## 0002-WebSocket 心跳机制双向不匹配导致周期性断连 [P0-紧急] - ✅ 已修复并验证

**位置**: `src/websocket/handler.rs` - `handle_message` 函数

**问题描述**:
后端设计意图为"服务端主导心跳"（服务端每30秒发Ping，客户端回复Pong），但实际实现中：
1. 服务端发送 Ping 后，客户端未回复 Pong
2. 客户端发送 Ping 后，服务端未回复 Pong（仅记录警告）
3. 双方都在发送 Ping，但都不回复对方的 Ping
4. 前端心跳超时更短（60秒），导致每60秒周期性断连

**修复方案**（混合模式 - 双向心跳 + 配置同步）：

### 1. 后端修复
在 `handle_message` 中添加处理客户端 Ping 的逻辑：

```rust
// 心跳请求 - 回复 Pong（支持客户端主导的心跳）
WebSocketMessage::Ping => {
    let pong = WebSocketMessage::Pong;
    if let Ok(json) = pong.to_json() {
        let _ = tx.send(json).await;
    }
    debug!("Received ping from user: {}, sent pong", user_id);
}

// 心跳响应
WebSocketMessage::Pong => {
    if let Ok(mut last) = last_pong.lock() {
        *last = Instant::now();
    }
    debug!("Received pong from user: {}", user_id);
}
```

### 2. 配置同步方案
为避免前后端心跳配置不一致导致的问题，实现了公开配置端点：

**后端**: `GET /api/config/client` - 返回客户端需要的配置
```rust
pub struct ClientConfig {
    pub websocket: ClientWebSocketConfig,
    pub reconnect: ClientReconnectConfig,
    pub upload: ClientUploadConfig,
    pub system: ClientSystemConfig,
}
```

**前端**: 页面加载时自动获取服务端配置
```javascript
const response = await fetch('http://localhost:8080/api/config/client');
const serverConfig = response.data;
// 使用服务端配置的心跳间隔
const interval = serverConfig.websocket.heartbeat_interval_secs * 1000;
```

### 3. 验证结果
终端日志分析（连接时长 > 7分钟，无断连记录）：
```
08:02:14 - WebSocket 连接建立，用户认证成功
08:02:14 - 收到客户端 Pong
08:02:44 - 收到客户端 Ping，回复 Pong
08:02:44 - 收到客户端 Pong
08:03:14 - 收到客户端 Ping，回复 Pong
08:03:14 - 收到客户端 Pong
...（持续 7+ 分钟，每 30 秒双向心跳正常）
08:09:14 - 仍在正常心跳交互，无断连
```

**状态**: ✅ 已修复并验证 - 混合心跳模式工作正常，连接稳定

**验证方式**: 
- HTML+JS 测试页面 (`temp/test/websocket-heartbeat-test.html`)
- Chrome DevTools 自动化测试
- 终端日志确认无断连

**影响文件**:
- `src/websocket/handler.rs` - 添加 Ping 处理分支
- `src/config/mod.rs` - ClientConfig 定义
- `src/handlers/config.rs` - 公开配置端点
- `src/routes/mod.rs` - 路由配置
- `temp/test/websocket-heartbeat-test.html` - 测试页面

**关联配置**:
- `websocket.heartbeat_interval_secs` - 30秒（服务端发Ping间隔）
- `websocket.heartbeat_timeout_secs` - 90秒（服务端超时）
- 前端通过 `/api/config/client` 动态获取配置

---

## 0003-WebSocket 未认证消息绕过风险 [P0-紧急] - ✅ 已修复

**位置**: `src/websocket/handler.rs` - `handle_message` 函数

**问题描述**:
后端仅在连接建立后的首条消息进行认证检查，后续消息默认已通过认证。但前端在异常情况下（如断线重连）可能：
1. 连接断开后消息队列仍保留待发送的业务消息（如 `ChatMessage`）
2. 重连后自动刷新消息队列，在认证完成前发送业务消息
3. 后端收到未认证的业务消息，虽拒绝但已造成安全风险

**日志证据**:
```
2026-04-13T14:07:39.163688Z  WARN seredeli_room::websocket::handler: 
  First message must be authentication or reconnect, got: ChatMessage { 
    room_id: 4bfa11aa-b30d-46ad-8b62-9c17b8dd82ae, 
    content: "这个功能好用", 
    reply_to: None 
  }
```

**安全风险评估**:

| 风险项 | 严重程度 | 说明 |
|--------|---------|------|
| **消息伪造** | 🔴 高 | 未认证用户可能通过重连机制发送消息 |
| **身份冒充** | 🔴 高 | 如果队列中有旧消息，可能以旧身份发送 |
| **信息泄露** | 🟡 中 | 消息内容在日志中可见 |
| **拒绝服务** | 🟡 中 | 大量重连+消息发送可能耗尽资源 |

**根本原因**:
1. **后端**: 仅验证首条消息，未维护"已认证"状态，后续消息无状态检查
2. **前端**: 断线重连时未清空消息队列，可能在认证前发送业务消息
3. **协议**: 缺乏"连接状态机"定义，前后端对连接生命周期理解不一致

**修复方案**:

### 1. 后端强化（必须）

**A. 连接状态机模式**
```rust
enum ConnectionState {
    Unauthenticated,  // 只允许 Auth/Reconnect
    Authenticated,    // 允许所有消息
}

// 每条消息都检查状态，不只是首条
async fn handle_message(
    state: &mut ConnectionState,
    msg: WebSocketMessage,
    // ...
) -> Result<()> {
    match state {
        ConnectionState::Unauthenticated => {
            if !is_auth_message(&msg) {
                return Err("Unauthorized: authentication required");
            }
            // 认证成功后切换状态
            *state = ConnectionState::Authenticated;
        }
        ConnectionState::Authenticated => {
            // 正常处理业务消息
        }
    }
}
```

**B. 连接断开时清理资源**
- 断开连接时清理该连接的所有待处理消息
- 记录安全审计日志

### 2. 前端修复（必须）

**A. 断开连接时清空消息队列**
```typescript
disconnect(): void {
    this.messageQueue = []  // 清空队列，防止重连后发送旧消息
    // ... 其他清理
}
```

**B. 认证后才允许入队**
```typescript
send(message: WebSocketMessage): boolean {
    // 未认证时不允许发送业务消息
    if (!this.isAuthenticated() && !isAuthMessage(message)) {
        console.error('Cannot send message before authentication');
        return false;
    }
    // ...
}
```

**C. 认证完成后才 resolve connect() Promise**
```typescript
// 当前：onopen 后立即 resolve
// 修复：收到 AuthResult { success: true } 后才 resolve
```

### 3. 审计日志（建议）

记录所有安全相关事件：
- 认证失败尝试（WARN 级别）
- 未认证消息尝试（WARN 级别）
- 异常连接模式（如短时间内大量连接）

**状态**: ✅ 已修复

**修复内容**:
1. **连接状态机** - 实现 `ConnectionState` 枚举（`Unauthenticated` / `Authenticated`）
2. **首条消息强制认证** - `wait_for_auth` 函数强制要求首条消息为 `Auth` 或 `Reconnect`
3. **状态检查** - 每条消息都通过 `is_message_allowed` 检查当前状态
4. **安全日志** - 未认证消息尝试记录 WARN 级别日志

**验证代码位置**:
- `src/websocket/handler.rs:L44-65` - `ConnectionState` 定义和 `is_message_allowed` 方法
- `src/websocket/handler.rs:L470-569` - `wait_for_auth` 强制认证
- `src/websocket/handler.rs:L350-367` - 消息处理时的状态检查

**影响文件**:
- `src/websocket/handler.rs` - 连接状态机和认证检查

**关联任务**: ✅ 全部完成
- ✅ 后端: 实现连接状态机，每条消息验证认证状态
- ✅ 测试: WebSocket 测试覆盖认证流程

---

## 0004-房间列表消息预览无法实时同步 [P1-高优先级] - ✅ 已修复

**位置**: WebSocket 消息订阅机制

**问题描述**:
当前后端架构中，用户必须显式发送 `JoinRoom` 消息订阅特定房间后，才能接收该房间的消息。这导致以下用户体验问题：

1. **房间列表消息预览滞后**: 用户已加入的房间（如房间A、B、C），如果当前只进入了房间A，则无法实时收到房间B、C的消息更新
2. **需要频繁刷新**: 用户必须手动刷新房间列表才能看到最新消息预览
3. **不符合即时通讯预期**: 用户加入群聊后即使不在该群聊天界面，也能收到消息通知

**修复方案**:

### 1. 新增用户级消息订阅机制

**A. 新增 WebSocket 消息类型** (`src/websocket/protocol.rs`)
```rust
/// 房间消息摘要（用于房间列表实时更新）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMessageSummary {
    pub room_id: Uuid,
    pub last_message: MessagePreview,
    pub unread_count: u32,
}

pub enum ServerMessage {
    // ... 现有消息类型
    RoomMessageSummary(RoomMessageSummary),
}
```

**B. 后端实现** (`src/websocket/handler.rs` & `src/websocket/manager.rs`)
- 用户认证成功后，自动查询该用户所有已加入的房间
- 将用户添加到各房间的消息摘要订阅者列表中
- 当任意已加入房间有新消息时，广播 `RoomMessageSummary` 给所有订阅用户

**C. 消息格式**
```json
{
  "type": "RoomMessageSummary",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "last_message": {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "content": "最新消息内容",
      "sender_name": "user123",
      "created_at": "2026-05-04T10:30:00.000Z"
    },
    "unread_count": 5
  }
}
```

### 2. 用户端适配 (client/user)

**A. 新增类型定义** (`client/user/src/types/websocket.ts`)
```typescript
export interface MessagePreview {
  id: string
  content: string
  sender_name: string
  created_at: string
}

export interface RoomMessageSummaryPayload {
  room_id: string
  last_message: MessagePreview
  unread_count: number
}

export enum WSMessageType {
  // ... 现有类型
  ROOM_MESSAGE_SUMMARY = 'RoomMessageSummary',
}
```

**B. Room Store 更新** (`client/user/src/stores/room.ts`)
```typescript
function updateRoomLastMessage(
  roomId: string,
  message: { id: string; content: string; sender_name: string; created_at: string },
  incrementUnread: boolean = true,
) {
  const room = rooms.value.find((r) => r.id === roomId)
  if (room) {
    room.last_message = message
    if (incrementUnread && currentRoom.value?.id !== roomId) {
      room.unread_count = (room.unread_count || 0) + 1
    }
  }
}
```

**C. WebSocket 消息处理** (`client/user/src/views/ChatRoomView.vue`)
```typescript
function handleRoomMessageSummary(payload: RoomMessageSummaryPayload) {
  const isCurrentRoom = payload.room_id === roomId.value
  roomStore.updateRoomLastMessage(
    payload.room_id,
    payload.last_message,
    !isCurrentRoom, // 只有非当前房间才增加未读数
  )
}

function subscribeMessages() {
  wsStore.onMessage<RoomMessageSummaryPayload>(
    WSMessageType.ROOM_MESSAGE_SUMMARY, 
    handleRoomMessageSummary
  )
}
```

### 3. 修改的文件清单

**后端**:
- ✅ `src/websocket/protocol.rs` - 新增 RoomMessageSummary 消息类型
- ✅ `src/websocket/handler.rs` - 认证成功后自动订阅用户房间
- ✅ `src/websocket/manager.rs` - 新增用户级房间订阅管理
- ✅ `src/models/room.rs` - MessagePreview 添加 Deserialize trait

**用户端 (client/user)**:
- ✅ `client/user/src/types/websocket.ts` - 新增 RoomMessageSummaryPayload 类型
- ✅ `client/user/src/stores/room.ts` - updateRoomLastMessage 支持未读计数
- ✅ `client/user/src/views/ChatRoomView.vue` - 订阅 RoomMessageSummary 消息

**调试端 (client/debug)**:
- ✅ `client/debug/src/types/websocket.ts` - 新增 RoomMessageSummaryPayload 类型
- ✅ `client/debug/src/store/websocket.ts` - 添加消息处理
- ✅ `client/debug/src/pages/room/RoomListPage.vue` - 显示实时消息预览

### 4. 工作流程

```
用户连接 WebSocket (用户端)
    ├── 认证成功 
    │   └── 后端自动查询用户所有已加入房间
    │   └── 将用户添加到各房间的摘要订阅列表
    │
    ├── 任意已加入房间有新消息
    │   ├── 如果用户在房间内 → 收到 NewMessage
    │   └── 无论用户在何处 → 收到 RoomMessageSummary
    │       └── 更新房间列表的消息预览和未读数
    │
    └── 断开连接 
        └── 后端自动从所有订阅列表中移除用户
```

### 5. 临时方案处理

原临时方案（在 ChatRoomView.vue 中通过 NewMessage 更新房间列表）已优化：
- 保留 `updateRoomLastMessage` 函数用于处理 RoomMessageSummary
- 新增 `incrementUnread` 参数控制是否增加未读计数
- 当前房间的消息不增加未读数

**状态**: ✅ 已修复 - 用户现在可以实时收到所有已加入房间的消息摘要更新

**验证**:
- ✅ `cargo check` 编译通过
- ✅ `cargo clippy` 检查通过
- ✅ `npm run type-check` (client/user) 类型检查通过

---

## 0005-通知系统在线/离线判断导致消息丢失 [P1-高优先级] - ✅ 已修复

**位置**: `src/services/notification_service.rs` - 各类通知发送方法

**问题描述**:
当前通知系统的逻辑是：
1. 如果用户**在线** → 通过 WebSocket 实时推送，**不存储到数据库**
2. 如果用户**离线** → 存储到数据库作为离线通知

这导致以下问题：
1. **消息丢失风险**: 用户在线时推送失败（网络抖动、连接未就绪），通知丢失
2. **多设备不同步**: 手机在线收到通知，电脑端无记录
3. **页面刷新丢失**: 在线时收到通知，刷新页面后通知消失
4. **无历史记录**: 无法查询历史通知，只能获取当前未读

**修复方案**: **方案A（后端双写）**

### 实现细节

无论用户是否在线，都先将通知存储到数据库，如果用户在线则额外推送 WebSocket。

**核心逻辑** (`src/services/notification_service.rs`):
```rust
pub async fn send_mention(&self, mentioned_user_id: Uuid, mention_info: MentionInfo) -> Result<()> {
    // 1. 先存储到数据库（无论在线与否，确保持久化）
    let notification_id = self
        .store_notification(
            mentioned_user_id,
            NotificationDbType::Mentioned,
            None,
            &format!("{} 提到了你", mention_info.mentioned_by_name),
            &serde_json::to_value(&mention_info).unwrap_or_default(),
        )
        .await?;

    debug!("Mention notification stored to database, id: {}", notification_id);

    // 2. 如果用户在线，额外推送WebSocket（异步，失败不影响已存储的通知）
    if self.ws_manager.is_user_online(mentioned_user_id) {
        let ws_message = WebSocketMessage::Mentioned { ... };
        if let Ok(json) = ws_message.to_json() {
            if let Err(e) = self.ws_manager.send_to_user(mentioned_user_id, json).await {
                warn!("Failed to send WebSocket notification: {}", e);
            }
        }
    }
    
    Ok(())
}
```

### 覆盖范围

| 通知方法 | 目标用户 | 写表 | WebSocket推送 | 状态 |
|---------|---------|------|--------------|------|
| `send_private_message` | 普通用户 | ✅ | ✅ | 已修复 |
| `send_mention` | 普通用户 | ✅ | ✅ | 已修复 |
| `send_room_invitation` | 普通用户 | ✅ | ✅ | 已修复 |
| `send_system_notification` | 指定用户（含admin） | ✅ | ✅ | 已修复 |
| `send_file_upload_complete` | 普通用户 | ✅ | ✅ | 已修复 |
| `send_pending_action` | Admin | ✅ | ✅ | 已修复 |
| `send_config_reload_notification` | Admin | ✅ | ✅ | 已修复 |

### HTTP API 端点

新增 RESTful API 用于前端获取通知和标记已读：

| 方法 | 端点 | 说明 |
|------|------|------|
| GET | `/notifications` | 获取通知列表（支持分页） |
| GET | `/notifications/unread-count` | 获取未读通知数量 |
| POST | `/notifications/:id/read` | 标记单条通知已读 |
| POST | `/notifications/read-all` | 标记所有通知已读 |

**架构简化**:
- **WebSocket**: 仅用于实时推送新通知
- **HTTP API**: 用于获取通知列表、标记已读/未读

### WebSocket 清理

移除了 WebSocket 中标记通知已读的处理，避免与 HTTP API 重复：
- 移除 `MarkNotificationRead` 消息处理
- 移除 `MarkAllNotificationsRead` 消息处理
- 移除 `handle_mark_notification_read` 函数
- 移除 `handle_mark_all_notifications_read` 函数

### 前端适配

**1. 新增 API 模块** (`client/user/src/api/notification.ts`):
```typescript
export const notificationApi = {
  getNotifications(params: NotificationQueryParams): Promise<ApiResponse<NotificationListResponse>>
  getUnreadCount(): Promise<ApiResponse<{ count: number }>>
  markAsRead(notificationId: string): Promise<ApiResponse<void>>
  markAllAsRead(): Promise<ApiResponse<void>>
}
```

**2. Store 更新** (`client/user/src/stores/notification.ts`):
- 使用服务端未读计数替代本地计数
- 集成 HTTP API 获取通知列表
- 标记已读时同步调用 API

**3. Badge 修复** (`client/user/src/styles/variables.css`):
- 添加 `--color-danger` CSS 变量，修复 badge 背景透明问题

### 前端缓存弃用说明

之前实现的前端缓存（`localStorage` 存储通知）已弃用，原因：
1. **数据一致性**: 后端双写确保数据持久化，前端无需缓存
2. **多端同步**: 后端成为唯一数据源，多设备自动同步
3. **简化逻辑**: 移除缓存逻辑，降低代码复杂度

**移除的缓存逻辑**:
- `localStorage` 中的通知存储
- 缓存过期检查
- 缓存与服务器数据合并逻辑

### 修复文件清单

**后端**:
- ✅ `src/services/notification_service.rs` - 所有通知方法实现双写
- ✅ `src/handlers/notification.rs` - 新增 HTTP API 处理器
- ✅ `src/routes/mod.rs` - 注册通知路由
- ✅ `src/websocket/handler.rs` - 移除 WS 标记已读处理

**前端**:
- ✅ `client/user/src/api/notification.ts` - 新增通知 API 模块
- ✅ `client/user/src/stores/notification.ts` - 集成 HTTP API，使用服务端计数
- ✅ `client/user/src/composables/quick/useQuickNotification.ts` - Badge 响应式更新
- ✅ `client/user/src/styles/variables.css` - 添加 `--color-danger` 变量

### 验证结果

- ✅ 在线用户收到通知后刷新页面，通知仍然显示
- ✅ Badge 正确显示未读数量（基于服务端计数）
- ✅ 标记已读后 Badge 实时更新
- ✅ `cargo clippy` 检查通过
- ✅ `npm run lint` 检查通过

**状态**: ✅ 已修复 - 所有通知类型（普通用户和 admin）都实现双写，确保不丢失

---

### 附录: 方案B（Redis 混合架构）- 后续优化参考

对于高并发场景，可考虑引入 Redis 优化性能：

```
┌─────────────────────────────────────────────────────────────┐
│  后端通知服务                                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  1. 写入DB   │  │ 2. 写入Redis │  │  3. WebSocket推送    │ │
│  │  (持久化)    │  │ (实时计数)   │  │  (在线用户)          │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Redis 数据结构**:
```redis
# 用户未读通知计数
notification:unread_count:{user_id} = 5

# 用户未读通知列表 (Sorted Set)
notification:unread:{user_id} = [notification_id_1, notification_id_2, ...]
```

**适用场景**: 日活用户 > 10万，通知发送频率 > 1000/秒

**当前状态**: 方案A已满足需求，方案B作为未来高并发优化参考

---

---

## 0006-用户设置体系 [P1-高优先级] - ✅ 后端已完成 / 🎨 前端待开发

**位置**: 后端用户相关模块 + 前端 SettingsView.vue

**状态**: 
- ✅ **后端**: 已完成所有基础功能实现
- 🎨 **前端**: 待开发 SettingsView.vue 界面

---

### 后端实现状态

#### 1. 数据库迁移 ✅
- **005_user_settings_and_security.sql**: 基础用户设置表 + 账号安全表
- **006_user_settings_expansion.sql**: 扩展设置（语言、无障碍、媒体）+ 房间级设置

#### 2. 数据模型 ✅
| 设置分组 | 结构体 | 状态 |
|---------|--------|------|
| 通知设置 | `NotificationSettings` | ✅ 完成 |
| 隐私设置 | `PrivacySettings` | ✅ 完成 |
| 消息设置 | `MessageSettings` | ✅ 完成 |
| 语言设置 | `LanguageSettings` | ✅ 完成 |
| 无障碍设置 | `AccessibilitySettings` | ✅ 完成 |
| 媒体设置 | `MediaSettings` | ✅ 完成 |
| 房间级设置 | `UserRoomSettings` | ✅ 完成 |

#### 3. API 端点 ✅
```
GET    /api/v1/users/me/settings              # 获取用户完整设置
PATCH  /api/v1/users/me/settings              # 部分更新用户设置
GET    /api/v1/users/me/rooms/settings        # 获取所有房间设置列表
GET    /api/v1/users/me/rooms/:room_id/settings  # 获取指定房间设置
PATCH  /api/v1/users/me/rooms/:room_id/settings  # 更新房间设置
DELETE /api/v1/users/me/rooms/:room_id/settings  # 重置房间设置
```

#### 4. 功能特性 ✅
- 支持部分更新（只传需要修改的分组）
- 使用 UPSERT 优化数据库操作
- 输入验证（颜色格式、字符串长度等）
- 默认值自动填充
- 房间级设置支持：静音、置顶、自定义名称/颜色、通知偏好

---

### 前端待开发内容 🎨

#### SettingsView.vue 界面模块

**1. 账号安全模块**
- [ ] 登录设备管理界面
  - 显示当前登录设备列表（包含 is_blocked 禁用状态）
  - 支持远程登出其他设备
  - 支持禁用/启用设备（被禁用设备无法使用旧 Token 登录）
  - 设备信息展示（设备名称、类型、位置、最后活跃时间、禁用状态）
  - 设备类型图标（mobile/tablet/desktop/unknown）
  - 操作按钮：登出、禁用、启用
- [ ] 登录历史查询界面
  - 分页显示登录记录
  - 显示登录时间、IP、设备、结果、风险等级
  - 标记可疑登录
  - 时间线形式展示
- [ ] 异地登录提醒设置
  - 开关：是否启用异地登录提醒
- [ ] 单设备登录设置
  - 开关：仅允许单设备登录（开启后新登录会自动终止其他会话）

**2. 通知设置模块**
- [ ] 通知开关组
  - 私信通知开关
  - @提及通知开关
  - 房间邀请通知开关
  - 系统通知开关
  - 文件上传完成通知
- [ ] 提醒方式设置
  - 声音提醒开关
  - 桌面通知开关
- [ ] 免打扰模式
  - 开关：启用免打扰
  - 时间范围选择（可选）

**3. 隐私设置模块**
- [ ] 在线状态可见性
  - 单选：所有人 / 仅好友 / 不可见
- [ ] 个人资料可见性
  - 单选：所有人 / 仅好友 / 不可见
- [ ] 互动权限
  - 开关：允许陌生人私信
  - 开关：允许房间邀请

**4. 消息设置模块**
- [ ] 消息显示设置
  - 开关：消息预览
  - 开关：已读回执
  - 开关：输入状态显示

**5. 语言与地区模块**
- [ ] 界面语言选择
  - 下拉选择：zh-CN / en-US 等
- [ ] 时区设置
  - 下拉选择时区
- [ ] 时间格式
  - 单选：12小时制 / 24小时制
- [ ] 日期格式
  - 单选：YYYY-MM-DD / DD/MM/YYYY / MM/DD/YYYY
- [ ] 星期起始日
  - 单选：周一 / 周日

**6. 无障碍设置模块**
- [ ] 字体大小
  - 单选：小 / 中 / 大
- [ ] 辅助功能开关
  - 开关：减少动效
  - 开关：高对比度模式
  - 开关：紧凑模式

**7. 媒体与存储模块**
- [ ] 媒体下载设置
  - 开关：自动下载媒体文件
  - 开关：保存到相册
- [ ] 图片质量
  - 单选：原图 / 高 / 中 / 低
- [ ] 自动播放设置
  - 开关：自动播放视频
  - 开关：自动播放音频

**8. 房间级设置模块**
- [ ] 房间列表管理
  - 显示用户参与的所有房间
  - 支持搜索和筛选
- [ ] 单个房间设置
  - 开关：静音该房间
  - 通知偏好：全部 / 仅@提及 / 静音
  - 开关：置顶该房间
  - 输入框：自定义房间名称（可选）
  - 颜色选择器：自定义房间颜色（可选）

---

### 前后端协作说明

**后端职责**（已完成）:
1. 存储用户设置偏好
2. 提供 CRUD API
3. 在通知服务中读取设置并应用（如：检查是否静音、是否启用桌面通知等）

**前端职责**（待开发）:
1. 提供友好的设置界面
2. 调用后端 API 保存/读取设置
3. 在客户端应用设置（如：根据设置决定是否播放声音、是否显示桌面通知等）
4. 本地缓存 + 云端同步策略

**注意**: 某些功能需要前后端协同实现，例如：
- **房间静音**: 后端负责记录静音状态，前端负责在收到消息时根据设置决定是否显示通知
- **免打扰模式**: 后端负责存储设置，前端负责在界面上显示免打扰状态，后端在发送通知时检查该设置

**解决方案**:

### 方案A：独立用户设置表（推荐）

创建 `user_settings` 表存储用户个性化设置：

```sql
-- 用户设置表
CREATE TABLE user_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- 通知设置 (JSONB)
    notification_settings JSONB DEFAULT '{
        "private_message": true,
        "mentioned": true,
        "room_invitation": true,
        "system_notification": true,
        "file_upload_complete": true,
        "sound_enabled": true,
        "desktop_notification": true
    }'::jsonb,
    
    -- 隐私设置 (JSONB)
    privacy_settings JSONB DEFAULT '{
        "online_status_visibility": "everyone",
        "profile_visibility": "everyone",
        "allow_stranger_message": true,
        "allow_room_invitation": true
    }'::jsonb,
    
    -- 消息设置 (JSONB)
    message_settings JSONB DEFAULT '{
        "message_preview": true,
        "read_receipt": true,
        "typing_indicator": true,
        "do_not_disturb": false
    }'::jsonb,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT unique_user_settings UNIQUE (user_id)
);
```

### API 设计

```
GET    /api/v1/users/me/settings          # 获取用户设置
PUT    /api/v1/users/me/settings          # 更新用户设置（支持部分更新）
```

**请求/响应示例**:

```json
// GET /api/v1/users/me/settings
{
  "success": true,
  "data": {
    "notifications": {
      "private_message": true,
      "mentioned": true,
      "room_invitation": true,
      "system_notification": true,
      "sound_enabled": true,
      "desktop_notification": true
    },
    "privacy": {
      "online_status_visibility": "everyone",
      "profile_visibility": "everyone",
      "allow_stranger_message": true,
      "allow_room_invitation": true
    },
    "message": {
      "message_preview": true,
      "read_receipt": true,
      "typing_indicator": true,
      "do_not_disturb": false
    }
  }
}

// PUT /api/v1/users/me/settings（部分更新）
{
  "notifications": {
    "sound_enabled": false,
    "desktop_notification": false
  }
}
```

### 实现建议

**后端**:
- 新增 `UserSettingsService` 服务层
- 新增 `UserSettings` 模型和 DTO
- 修改 `user_settings` 时同步更新通知服务的行为

**前端**:
- `SettingsView.vue` 按模块组织：账号安全、通知设置、隐私设置、消息设置
- 使用本地缓存 + 云端同步策略
- 设置变更实时生效

**优先级**:
1. 账号安全（阶段8.7.2）- 高优先级
2. 通知设置 - 中优先级
3. 隐私设置 - 中优先级
4. 消息设置 - 低优先级

**状态**: 📋 规划中 - 待阶段9完成后实施

**关联任务**:
- 阶段8.7.2 用户账号安全系统
- 前端 SettingsView.vue 开发

---

*文档创建时间: 2026-04-10*
*关联任务: 阶段9 - 后端细节优化*
*更新时间: 2026-05-06 - 添加 0006 用户设置体系缺失*
