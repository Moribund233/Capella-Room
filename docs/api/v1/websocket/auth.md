# WebSocket 连接认证和心跳机制

> **文档版本**: v1  
> **最后更新**: 2026-04-26

## 目录

- [概述](#概述)
- [连接流程](#连接流程)
- [认证机制](#认证机制)
- [心跳机制](#心跳机制)
- [断线重连](#断线重连)
- [错误处理](#错误处理)
- [代码示例](#代码示例)

---

## 概述

WebSocket 连接认证是保障实时通信安全的第一道防线。本文档详细描述了：

- 如何建立 WebSocket 连接
- 如何进行身份认证
- 心跳保活机制
- 断线重连策略
- 错误处理方法

### 认证流程概览

```
┌─────────┐                    ┌─────────┐
│ Client  │ ── 1. WebSocket ──▶│ Server  │
│         │    连接建立         │         │
│         │◀─ 2. AuthResult ──│         │
│         │    认证结果         │         │
│         │ ── 3. Ping/Pong ─▶│         │
│         │    心跳保活         │         │
│         │◀─ 4. 业务消息 ────│         │
│         │    实时通信         │         │
└─────────┘                    └─────────┘
```

---

## 连接流程

### 1. 建立 WebSocket 连接

**连接端点**:
```
ws://localhost:8080/ws
```

**JavaScript 示例**:
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('WebSocket connected');
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = (event) => {
  console.log('WebSocket closed:', event.code, event.reason);
};
```

### 2. IP 安全检查

连接建立后，服务端会自动进行 IP 安全检查：

- 检查 IP 是否在白名单中（如果启用白名单模式）
- 检查 IP 是否在黑名单中
- 记录安全检查日志

如果 IP 被阻止，服务端会发送错误消息并关闭连接：

```json
{
  "type": "Error",
  "payload": {
    "code": "IP_BLOCKED",
    "message": "IP address is blocked"
  }
}
```

### 3. 认证超时

连接建立后，客户端必须在 **30 秒** 内完成认证（发送 `Auth` 或 `Reconnect` 消息），否则服务端会断开连接。

---

## 认证机制

### 首次连接认证

客户端需要在连接建立后发送认证消息：

**请求消息**:
```json
{
  "type": "Auth",
  "payload": {
    "token": "eyJhbGciOiJIUzI1NiIs..."
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 固定为 "Auth" |
| payload.token | string | 是 | JWT Access Token |

### 认证响应

服务端返回认证结果：

**认证成功**:
```json
{
  "type": "AuthResult",
  "payload": {
    "success": true,
    "message": "Authentication successful"
  }
}
```

**认证失败**:
```json
{
  "type": "AuthResult",
  "payload": {
    "success": false,
    "message": "Invalid token"
  }
}
```

**响应字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| type | string | 固定为 "AuthResult" |
| payload.success | boolean | 认证是否成功 |
| payload.message | string | 认证结果描述 |

### Token 获取

JWT Token 需要通过 HTTP API 获取：

```bash
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

响应：
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_in": 900,
    "user": { ... }
  }
}
```

### Token 刷新

WebSocket 连接期间，服务端会每 5 分钟验证一次 Token。如果 Token 过期，会发送错误消息并断开连接：

**Token 过期错误消息**:
```json
{
  "type": "Error",
  "payload": {
    "code": "TOKEN_EXPIRED",
    "message": "Token expired, please reconnect"
  }
}
```

**错误字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| type | string | 固定为 "Error" |
| payload.code | string | 错误代码: `TOKEN_EXPIRED` |
| payload.message | string | 错误描述 |

客户端需要：
1. 使用 Refresh Token 获取新的 Access Token
2. 重新建立 WebSocket 连接
3. 使用新 Token 进行认证

---

## 心跳机制

### 心跳流程

为了保持连接活跃，服务端实现了**服务端主导的心跳机制**：

1. **服务端发送 Ping**: 每 30 秒发送一次
2. **客户端回复 Pong**: 收到 Ping 后立即回复
3. **超时检测**: 如果 90 秒内未收到 Pong，服务端断开连接

**Ping 消息** (服务端 → 客户端):
```json
{
  "type": "Ping"
}
```

**Pong 消息** (客户端 → 服务端):
```json
{
  "type": "Pong"
}
```

> ⚠️ **重要**: Ping 和 Pong 是 unit variant，不需要 payload 字段。

### 双向心跳支持

除了服务端主导的心跳外，系统也支持**客户端主动发送心跳**：

- 客户端可以主动发送 `Ping` 消息，服务端会回复 `Pong`
- WebSocket 协议层的 Ping/Pong 帧也会被自动处理（由 axum 底层实现）
- 建议客户端采用**服务端主导模式**：等待服务端 Ping 并回复 Pong，同时监控是否超时

### 心跳配置

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| heartbeat_interval_secs | 30 | 心跳发送间隔（秒） |
| heartbeat_timeout_secs | 90 | 心跳超时时间（秒） |

这些配置可以通过管理接口动态调整。

#### 获取服务端配置

为了避免前后端配置不同步导致的连接问题，客户端应该在启动时从服务端获取配置：

```http
GET /api/config/client
```

**响应示例**：
```json
{
  "success": true,
  "data": {
    "websocket": {
      "heartbeat_interval_secs": 30,
      "heartbeat_timeout_secs": 90,
      "auth_timeout_secs": 30
    },
    "reconnect": {
      "base_delay_ms": 1000,
      "max_delay_ms": 30000,
      "max_attempts": 10,
      "multiplier": 2
    }
  }
}
```

**使用建议**：
1. 在应用启动时调用此接口获取服务端配置
2. 使用服务端返回的 `heartbeat_interval_secs` 和 `heartbeat_timeout_secs` 设置客户端心跳超时检测
3. 使用服务端返回的重连配置 (`reconnect`) 控制重连行为
4. 此端点为开放端点，无需认证即可访问

> 📖 **详细文档**: [system.md](../http/system.md) - 包含完整的客户端配置端点说明

### 心跳超时处理

如果客户端未能在 90 秒内回复 Pong，服务端会：

1. 发送超时错误消息：
```json
{
  "type": "Error",
  "payload": {
    "code": "HEARTBEAT_TIMEOUT",
    "message": "Connection closed due to heartbeat timeout"
  }
}
```

**错误字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| type | string | 固定为 "Error" |
| payload.code | string | 错误代码: `HEARTBEAT_TIMEOUT` |
| payload.message | string | 错误描述 |

2. 关闭 WebSocket 连接

客户端应该实现自动重连机制。当收到 `HEARTBEAT_TIMEOUT` 错误时，应立即尝试重新连接。

---

## 断线重连

### 重连流程

当连接意外断开时，客户端可以使用重连机制恢复会话：

```
┌─────────┐                    ┌─────────┐
│ Client  │ ── 1. Reconnect ─▶│ Server  │
│         │    携带 Token      │         │
│         │    和断线时间      │         │
│         │◀─ 2. ReconnectResult│        │
│         │    重连结果         │         │
│         │    含房间列表       │         │
│         │ ── 3. 自动加入房间 ▶│        │
│         │    (服务端处理)     │         │
│         │◀─ 4. SessionRestored│        │
│         │    恢复完成         │         │
└─────────┘                    └─────────┘
```

### 重连请求

**请求消息**:
```json
{
  "type": "Reconnect",
  "payload": {
    "token": "eyJhbGciOiJIUzI1NiIs...",
    "last_disconnect_at": "2024-01-15T10:30:00Z"
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 固定为 "Reconnect" |
| payload.token | string | 是 | JWT Access Token |
| payload.last_disconnect_at | string | 否 | 上次断开时间（ISO 8601 格式）|

### 重连响应

**重连成功**:
```json
{
  "type": "ReconnectResult",
  "payload": {
    "success": true,
    "message": "Reconnected successfully",
    "rooms_to_rejoin": [
      "550e8400-e29b-41d4-a716-446655440000",
      "550e8400-e29b-41d4-a716-446655440001"
    ]
  }
}
```

**重连失败**:

如果 Token 无效或认证失败，服务端会返回 `AuthResult`（与首次认证失败相同）并断开连接：

```json
{
  "type": "AuthResult",
  "payload": {
    "success": false,
    "message": "Invalid token: 认证失败: Token验证失败: InvalidToken"
  }
}
```

> **注意**: 无论是首次认证 (`Auth`) 还是重连 (`Reconnect`)，只要认证失败，服务端都会返回 `AuthResult` 并断开连接，不会返回 `ReconnectResult`。

**响应字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| type | string | 固定为 "ReconnectResult" |
| payload.success | boolean | 重连是否成功 |
| payload.message | string | 结果描述 |
| payload.rooms_to_rejoin | array | 需要重新加入的房间 ID 列表 |

### 会话恢复

重连成功后，服务端会自动：

1. 断开旧连接（如果存在）
2. 重新注册新连接
3. 自动重新加入之前的房间（如果用户仍在房间中）
4. 发送会话恢复完成消息：

```json
{
  "type": "SessionRestored",
  "payload": {
    "restored_at": "2024-01-15T10:30:05Z",
    "rooms_restored": 2,
    "total_unread": 15
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| restored_at | string | 恢复时间（ISO 8601）|
| rooms_restored | integer | 恢复的房间数量 |
| total_unread | integer | 未读消息总数 |

### 获取离线消息

会话恢复后，客户端可以请求离线期间的消息：

**请求**:
```json
{
  "type": "GetMissedMessages",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "last_message_id": "550e8400-e29b-41d4-a716-446655440010"
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| room_id | string (UUID) | 是 | 房间 ID |
| last_message_id | string (UUID) | 否 | 客户端已知的最后一条消息 ID，服务端返回此消息之后的新消息 |

**响应**:
```json
{
  "type": "MissedMessages",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "messages": [
      {
        "message_id": "...",
        "room_id": "...",
        "sender_id": "...",
        "sender_name": "user1",
        "content": "Hello!",
        "reply_to": null,
        "reply_to_message": null,
        "created_at": "2024-01-15T10:25:00Z"
      }
    ],
    "has_more": false
  }
}
```

### 重连策略与指数退避

客户端实现重连时应遵循以下策略：

| 重连次数 | 延迟时间 | 说明 |
|---------|---------|------|
| 第 1 次 | 1 秒 | 立即尝试 |
| 第 2 次 | 2 秒 | 线性退避 |
| 第 3 次 | 4 秒 | 指数退避 |
| 第 4 次 | 8 秒 | 指数退避 |
| 第 5 次 | 16 秒 | 最大延迟 |
| 超过 5 次 | 停止重连 | 提示用户手动刷新 |

**重连触发条件**:
- 心跳超时（90 秒未收到 Ping）
- 连接异常关闭（非主动调用 `close()`）
- Token 过期后刷新成功

**重连最佳实践**:
1. 记录断开时间 `last_disconnect_at`，用于 Reconnect 消息
2. 重连成功后优先调用 `GetMissedMessages` 同步离线消息
3. 如果重连失败（Token 无效），先刷新 Token 再重连
4. 网络恢复后应重置重连计数器
```

---

## 错误处理

### 错误消息格式

```json
{
  "type": "Error",
  "payload": {
    "code": "ERROR_CODE",
    "message": "错误描述"
  }
}
```

### 认证相关错误码

| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| `AUTH_REQUIRED` | 需要认证 | 发送 Auth 消息进行认证 |
| `AUTH_FAILED` | 认证失败 | 检查 Token 是否有效 |
| `TOKEN_EXPIRED` | Token 已过期 | 刷新 Token 后重新连接 |
| `INVALID_MESSAGE` | 消息格式错误 | 检查消息 JSON 格式 |
| `UNAUTHORIZED` | 未授权的消息类型 | 确保已认证后再发送业务消息 |
| `HEARTBEAT_TIMEOUT` | 心跳超时 | 检查网络连接，重新连接 |
| `IP_BLOCKED` | IP 被阻止 | 联系管理员 |
| `SECURITY_CHECK_FAILED` | 安全检查失败 | 检查网络环境 |

### 连接状态机

```
┌─────────────┐     Auth/Reconnect      ┌─────────────┐
│ Unauthenticated│ ───────────────────▶ │ Authenticated│
│   (未认证)    │                        │   (已认证)   │
└─────────────┘                        └─────────────┘
       │                                        │
       │ 只允许 Auth/Reconnect                   │ 允许所有消息
       │                                        │
       ▼                                        ▼
  ┌─────────┐                              ┌─────────┐
  │ 断开连接 │                              │ 业务处理 │
  └─────────┘                              └─────────┘
```

在未认证状态下，只允许发送：
- `Auth` - 认证
- `Reconnect` - 重连

发送其他消息会收到认证失败响应并断开连接：

```json
{
  "type": "AuthResult",
  "payload": {
    "success": false,
    "message": "First message must be authentication or reconnect"
  }
}
```

> **注意**: 服务端会在返回此响应后断开连接。

---

## 代码示例

### 完整连接示例 (JavaScript)

```javascript
class WebSocketClient {
  constructor(url) {
    this.url = url;
    this.ws = null;
    this.token = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.reconnectDelay = 1000;
    this.isAuthenticated = false;
    this.heartbeatInterval = null;
  }

  // 建立连接
  connect(token) {
    this.token = token;
    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
      this.authenticate();
    };

    this.ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      this.handleMessage(msg);
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = (event) => {
      console.log('WebSocket closed:', event.code, event.reason);
      this.isAuthenticated = false;
      this.stopHeartbeat();
      this.handleReconnect();
    };
  }

  // 发送认证
  authenticate() {
    this.send({
      type: 'Auth',
      payload: { token: this.token }
    });
  }

  // 处理消息
  handleMessage(msg) {
    console.log('Received:', msg);

    switch (msg.type) {
      case 'AuthResult':
        if (msg.payload.success) {
          this.isAuthenticated = true;
          console.log('Authenticated successfully');
          this.startHeartbeat();
        } else {
          console.error('Authentication failed:', msg.payload.message);
        }
        break;

      case 'Ping':
        // 回复 Pong
        this.send({ type: 'Pong' });
        break;

      case 'Error':
        console.error('Server error:', msg.payload);
        if (msg.payload.code === 'TOKEN_EXPIRED') {
          this.refreshTokenAndReconnect();
        }
        break;

      // 处理其他消息类型...
      default:
        console.log('Unhandled message type:', msg.type);
    }
  }

  // 发送消息
  send(message) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.error('WebSocket is not connected');
    }
  }

  // 启动心跳
  startHeartbeat() {
    // 服务端会主动发送 Ping，这里只需要确保能收到并回复
    console.log('Heartbeat mechanism started');
  }

  // 停止心跳
  stopHeartbeat() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  // 处理重连
  handleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = this.reconnectDelay * this.reconnectAttempts;
      console.log(`Reconnecting in ${delay}ms... (attempt ${this.reconnectAttempts})`);
      setTimeout(() => this.connect(this.token), delay);
    } else {
      console.error('Max reconnection attempts reached');
    }
  }

  // 使用 Reconnect 消息重连
  reconnect() {
    this.send({
      type: 'Reconnect',
      payload: {
        token: this.token,
        last_disconnect_at: new Date().toISOString()
      }
    });
  }

  // 断开连接
  disconnect() {
    this.reconnectAttempts = this.maxReconnectAttempts; // 防止自动重连
    if (this.ws) {
      this.ws.close();
    }
  }
}

// 使用示例
const client = new WebSocketClient('ws://localhost:8080/ws');
client.connect('your-jwt-token');
```

### Python 示例

```python
import asyncio
import json
import websockets

class WebSocketClient:
    def __init__(self, url, token):
        self.url = url
        self.token = token
        self.ws = None
        self.authenticated = False

    async def connect(self):
        self.ws = await websockets.connect(self.url)
        
        # 发送认证
        await self.send({
            "type": "Auth",
            "payload": {"token": self.token}
        })
        
        # 处理消息
        async for message in self.ws:
            msg = json.loads(message)
            await self.handle_message(msg)

    async def send(self, message):
        await self.ws.send(json.dumps(message))

    async def handle_message(self, msg):
        msg_type = msg.get("type")
        
        if msg_type == "AuthResult":
            if msg["payload"]["success"]:
                self.authenticated = True
                print("Authenticated successfully")
            else:
                print(f"Authentication failed: {msg['payload']['message']}")
        
        elif msg_type == "Ping":
            # 回复 Pong
            await self.send({"type": "Pong"})
        
        elif msg_type == "Error":
            print(f"Error: {msg['payload']}")

# 使用
async def main():
    client = WebSocketClient("ws://localhost:8080/ws", "your-token")
    await client.connect()

asyncio.run(main())
```

---

## 相关文档

- [index.md](./index.md) - WebSocket 协议总览
- [room.md](./room.md) - 房间管理
- [message.md](./message.md) - 消息通信
- [notification.md](./notification.md) - 通知系统
- [user-status.md](./user-status.md) - 用户状态管理

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
