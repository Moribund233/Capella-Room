# WebSocket 协议总览

> **协议版本**: v1  
> **传输格式**: JSON

## 目录

- [概述](#概述)
- [连接流程](#连接流程)
- [消息格式](#消息格式)
- [消息类型概览](#消息类型概览)
- [连接管理](#连接管理)
- [错误处理](#错误处理)
- [最佳实践](#最佳实践)

---

## 概述

Seredeli Room 的 WebSocket 接口提供实时双向通信能力，支持以下核心功能：

- **实时消息**: 即时收发聊天消息
- **房间管理**: 加入/离开房间，获取在线用户列表
- **用户状态**: 在线状态同步与通知
- **消息操作**: 编辑、删除、回复消息
- **通知系统**: @提及、私信、系统通知
- **断线重连**: 自动恢复会话和同步离线消息

### 技术特性

| 特性 | 说明 |
|------|------|
| 协议 | WebSocket (RFC 6455) |
| 消息格式 | JSON |
| 认证方式 | JWT Token |
| 心跳机制 | Ping/Pong |
| 断线重连 | 支持自动重连和会话恢复 |
| 分布式支持 | Redis Pub/Sub 跨节点广播 |

---

## 连接流程

### 1. 建立连接

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
```

### 2. 连接认证

连接建立后，必须在 **30 秒** 内完成认证。详见 [auth.md](./auth.md)。

**简要示例**:
```json
{
  "type": "Auth",
  "payload": {
    "token": "eyJhbGciOiJIUzI1NiIs..."
  }
}
```

> 📖 **详细认证文档**: [auth.md](./auth.md) - 包含完整的认证流程、Token 刷新、断线重连等

---

## 消息格式

所有 WebSocket 消息采用统一的 JSON 格式：

```json
{
  "type": "消息类型",
  "payload": { ... }
}
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 消息类型标识符 |
| payload | object | 视类型而定 | 消息载荷数据（unit variant 类型如 Ping/Pong 不需要）|

### 消息类型分类

根据 Rust 枚举定义，消息分为两类：

**1. Struct Variant（需要 payload）**
```json
{
  "type": "Auth",
  "payload": {
    "token": "..."
  }
}
```

**2. Unit Variant（不需要 payload）**
```json
{
  "type": "Ping"
}
```

> **注意**: 服务端发送的消息也是同样的格式。客户端应该根据 `type` 字段处理消息，而不是依赖消息的顺序。

### 消息方向

- **C→S**: Client to Server (客户端发送)
- **S→C**: Server to Client (服务端推送)
- **双向**: 客户端和服务端都可以发送

---

## 消息类型概览

### 连接管理

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `Auth` | C→S | 连接认证 |
| `AuthResult` | S→C | 认证结果 |
| `Ping` | 双向 | 心跳请求 |
| `Pong` | 双向 | 心跳响应 |
| `Reconnect` | C→S | 断线重连 |
| `ReconnectResult` | S→C | 重连结果 |
| `Error` | S→C | 错误消息 |

### 房间管理

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `JoinRoom` | C→S | 加入房间 |
| `LeaveRoom` | C→S | 离开房间 |
| `RoomJoined` | S→C | 加入房间成功 |
| `RoomLeft` | S→C | 离开房间成功 |
| `UserJoined` | S→C | 用户加入房间通知 |
| `UserLeft` | S→C | 用户离开房间通知 |
| `OnlineUsers` | S→C | 房间在线用户列表 |

### 消息通信

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `ChatMessage` | C→S | 发送聊天消息 |
| `NewMessage` | S→C | 新消息通知 |
| `Typing` | C→S | 正在输入状态 |
| `StopTyping` | C→S | 停止输入状态 |
| `MessageRead` | C→S | 消息已读确认 |
| `MessageReadReceipt` | S→C | 消息已读回执 |
| `EditMessage` | C→S | 编辑消息 |
| `MessageEdited` | S→C | 消息已编辑通知 |
| `DeleteMessage` | C→S | 删除消息 |
| `MessageDeleted` | S→C | 消息已删除通知 |

### 用户状态

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `UpdateStatus` | C→S | 更新用户状态 |
| `UserStatusChanged` | S→C | 用户状态变更通知 |
| `GetOnlineUsers` | C→S | 获取全局在线用户 |
| `GlobalOnlineUsers` | S→C | 全局在线用户列表 |

### 断线重连

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `GetMissedMessages` | C→S | 请求离线消息 |
| `MissedMessages` | S→C | 离线消息列表 |
| `SessionRestored` | S→C | 会话恢复完成 |

### 通知系统

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `PrivateMessage` | S→C | 私信通知 |
| `Mentioned` | S→C | @提及通知 |
| `RoomInvitation` | S→C | 房间邀请通知 |
| `SystemNotification` | S→C | 系统通知 |
| `FileUploadComplete` | S→C | 文件上传完成通知 |
| `GetOfflineNotifications` | C→S | 获取离线通知 |
| `MarkNotificationRead` | C→S | 标记通知已读 |
| `MarkAllNotificationsRead` | C→S | 标记所有通知已读 |
| `OfflineNotifications` | S→C | 离线通知列表 |
| `NotificationReadConfirm` | S→C | 通知已读确认 |

### 系统日志

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `SubscribeLogs` | C→S | 订阅系统日志 |
| `UnsubscribeLogs` | C→S | 取消订阅日志 |
| `LogEntry` | S→C | 日志条目推送 |
| `LogSubscriptionConfirmed` | S→C | 订阅确认 |

---

## 连接管理

### 心跳机制

服务端会定期发送 `Ping` 消息，客户端必须回复 `Pong`。

**心跳配置**:
- 心跳间隔: 30秒
- 心跳超时: 90秒（未收到 Pong 则断开连接）

> 📖 **详细说明**: [auth.md](./auth.md#心跳机制)

### 断线重连

当连接意外断开时，客户端可以使用重连机制恢复会话。

> 📖 **详细说明**: [auth.md](./auth.md#断线重连)

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

### 常见错误码

| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| `AUTH_REQUIRED` | 需要认证 | 发送 Auth 消息进行认证 |
| `AUTH_FAILED` | 认证失败 | 检查 Token 是否有效 |
| `TOKEN_EXPIRED` | Token 已过期 | 刷新 Token 后重新连接 |
| `INVALID_MESSAGE` | 消息格式错误 | 检查消息 JSON 格式 |
| `NOT_IN_ROOM` | 不在房间中 | 先发送 JoinRoom 加入房间 |
| `ROOM_NOT_FOUND` | 房间不存在 | 检查房间 ID 是否正确 |
| `RATE_LIMITED` | 请求过于频繁 | 降低发送频率 |
| `HEARTBEAT_TIMEOUT` | 心跳超时 | 检查网络连接，重新连接 |
| `IP_BLOCKED` | IP 被阻止 | 联系管理员 |
| `SERVER_ERROR` | 服务器内部错误 | 稍后重试 |

---

## 最佳实践

### 1. 连接管理

```javascript
class WebSocketClient {
  constructor(url) {
    this.url = url;
    this.ws = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.reconnectDelay = 1000;
  }

  connect() {
    this.ws = new WebSocket(this.url);
    
    this.ws.onopen = () => {
      this.reconnectAttempts = 0;
      this.authenticate();
    };
    
    this.ws.onclose = () => {
      this.handleReconnect();
    };
    
    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
    
    this.ws.onmessage = (event) => {
      this.handleMessage(JSON.parse(event.data));
    };
  }

  handleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      setTimeout(() => this.connect(), this.reconnectDelay * this.reconnectAttempts);
    }
  }
}
```

### 2. 消息发送封装

```javascript
sendMessage(type, payload) {
  if (this.ws && this.ws.readyState === WebSocket.OPEN) {
    this.ws.send(JSON.stringify({ type, payload }));
  } else {
    console.error('WebSocket is not connected');
  }
}
```

### 3. 心跳处理

```javascript
handleMessage(msg) {
  switch (msg.type) {
    case 'Ping':
      this.sendMessage('Pong', {});
      break;
    // 处理其他消息...
  }
}
```

---

## 文档索引

| 文档 | 说明 |
|------|------|
| [index.md](./index.md) | 本文档 - WebSocket 协议总览 |
| [auth.md](./auth.md) | 连接认证和心跳机制 |
| [room.md](./room.md) | 房间管理（加入/离开/在线用户） |
| [message.md](./message.md) | 消息通信（发送/接收/编辑/删除） |
| [notification.md](./notification.md) | 通知系统（私信/@提及/系统通知） |
| [user-status.md](./user-status.md) | 用户状态管理 |

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
