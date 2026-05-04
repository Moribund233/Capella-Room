# WebSocket 房间管理

本文档描述 WebSocket 房间管理相关的消息类型和交互流程。

---

## 概述

房间管理是实时聊天系统的核心功能，包括：
- 加入/离开房间
- 获取在线用户列表
- 接收用户进出通知

---

## 消息类型

### 客户端发送的消息

| 消息类型 | 说明 | 需要认证 |
|----------|------|----------|
| `JoinRoom` | 加入房间 | ✅ |
| `LeaveRoom` | 离开房间 | ✅ |

### 服务端发送的消息

| 消息类型 | 说明 |
|----------|------|
| `RoomJoined` | 加入房间成功 |
| `RoomLeft` | 离开房间成功 |
| `UserJoined` | 其他用户加入房间（广播） |
| `UserLeft` | 其他用户离开房间（广播） |
| `OnlineUsers` | 房间在线用户列表 |
| `RoomMessageSummary` | 房间消息摘要（用于房间列表实时更新） |
| `Error` | 错误响应 |

---

## 加入房间

### 请求消息

```json
{
  "type": "JoinRoom",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 固定为 "JoinRoom" |
| payload.room_id | string (UUID) | 是 | 房间 ID |

### 成功响应

**1. 加入成功确认**:

```json
{
  "type": "RoomJoined",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "username": "user123"
  }
}
```

**2. 在线用户列表**:

```json
{
  "type": "OnlineUsers",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "users": [
      {
        "id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
        "username": "user123",
        "avatar_url": null,
        "status": "Online"
      },
      {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "user456",
        "avatar_url": "https://example.com/avatar.jpg",
        "status": "Away"
      }
    ]
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| room_id | string (UUID) | 房间 ID |
| user_id | string (UUID) | 当前用户 ID |
| username | string | 当前用户名称 |
| users | array | 在线用户列表 |
| users[].id | string (UUID) | 用户 ID |
| users[].username | string | 用户名称 |
| users[].avatar_url | string \| null | 头像 URL |
| users[].status | string | 用户状态: Online, Away, Busy, Offline |

**3. 广播给其他用户**:

其他房间成员会收到 `UserJoined` 消息：

```json
{
  "type": "UserJoined",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "username": "user123"
  }
}
```

### 错误响应

| 错误码 | 说明 | 场景 |
|--------|------|------|
| `ROOM_NOT_FOUND` | 房间不存在 | 房间 ID 无效 |
| `NOT_MEMBER` | 不是房间成员 | 私有房间且未被邀请 |
| `JOIN_FAILED` | 加入失败 | 数据库操作失败 |

**示例**:

```json
{
  "type": "Error",
  "payload": {
    "code": "NOT_MEMBER",
    "message": "You are not a member of this room"
  }
}
```

### 加入流程

```
客户端                              服务端
  │                                   │
  │────── JoinRoom ─────────────────▶│
  │                                   │
  │◀───── RoomJoined ────────────────│ (确认)
  │◀───── OnlineUsers ───────────────│ (用户列表)
  │                                   │
  │         广播给房间其他成员          │
  │◀───── UserJoined ────────────────│ (其他用户收到)
```

---

## 离开房间

### 请求消息

```json
{
  "type": "LeaveRoom",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 固定为 "LeaveRoom" |
| payload.room_id | string (UUID) | 是 | 房间 ID |

### 成功响应

**1. 离开成功确认**:

```json
{
  "type": "RoomLeft",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "username": "user123"
  }
}
```

**2. 广播给其他用户**:

其他房间成员会收到 `UserLeft` 消息：

```json
{
  "type": "UserLeft",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "username": "user123"
  }
}
```

### 离开流程

```
客户端                              服务端
  │                                   │
  │────── LeaveRoom ────────────────▶│
  │                                   │
  │◀───── RoomLeft ──────────────────│ (确认)
  │                                   │
  │         广播给房间其他成员          │
  │◀───── UserLeft ──────────────────│ (其他用户收到)
```

---

## 房间消息摘要

当用户已加入的任意房间有新消息时，服务端会自动推送 `RoomMessageSummary` 消息，用于更新房间列表中的消息预览和未读计数。

> **特性说明**:
> - 用户 WebSocket 认证成功后，后端自动订阅该用户所有已加入房间
> - 无需手动发送订阅消息
> - 当任意已加入房间有新消息时自动推送
> - 即使用户不在该房间界面，也能收到消息摘要

### 消息格式

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

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| room_id | string (UUID) | 房间 ID |
| last_message | object | 最后一条消息预览 |
| last_message.id | string (UUID) | 消息 ID |
| last_message.content | string | 消息内容（已截断） |
| last_message.sender_name | string | 发送者名称 |
| last_message.created_at | string (ISO 8601) | 消息发送时间 |
| unread_count | number | 未读消息数 |

### 工作流程

```
用户连接 WebSocket
    │
    ├── 认证成功
    │   └── 后端自动查询用户所有已加入房间
    │   └── 将用户添加到各房间的摘要订阅列表
    │
    ├── 任意已加入房间有新消息
    │   └── 后端推送 RoomMessageSummary 给该用户
    │
    └── 断开连接
        └── 后端自动从所有订阅列表中移除用户
```

### JavaScript 示例

```javascript
// 监听房间消息摘要
ws.addEventListener('message', (event) => {
  const msg = JSON.parse(event.data);
  
  if (msg.type === 'RoomMessageSummary') {
    const { room_id, last_message, unread_count } = msg.payload;
    
    // 更新房间列表中的消息预览
    updateRoomPreview(room_id, {
      content: last_message.content,
      senderName: last_message.sender_name,
      createdAt: last_message.created_at
    });
    
    // 更新未读计数
    updateUnreadCount(room_id, unread_count);
  }
});
```

### 与 JoinRoom 的区别

| 特性 | `RoomMessageSummary` | `JoinRoom` |
|------|---------------------|------------|
| 触发方式 | 自动订阅（认证后） | 手动发送 |
| 接收内容 | 消息摘要（预览） | 完整消息内容 |
| 适用场景 | 房间列表页面 | 聊天室页面 |
| 消息类型 | 仅最后消息预览 | 所有实时消息 |
| 未读计数 | 包含 | 不包含 |

---

## 在线用户状态

### 用户状态类型

| 状态 | 说明 |
|------|------|
| `Online` | 在线 |
| `Away` | 离开 |
| `Busy` | 忙碌 |
| `Offline` | 离线 |

### 状态变更通知

当用户更新状态时，会广播给所有相关房间的成员：

```json
{
  "type": "UserStatusChanged",
  "payload": {
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "username": "user123",
    "status": "Away"
  }
}
```

---

## 完整示例

### JavaScript 示例

```javascript
class RoomManager {
  constructor(ws) {
    this.ws = ws;
    this.currentRoom = null;
    this.onlineUsers = new Map();
  }

  // 加入房间
  async joinRoom(roomId) {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Join room timeout'));
      }, 10000);

      const handleMessage = (event) => {
        const msg = JSON.parse(event.data);

        if (msg.type === 'RoomJoined' && msg.payload.room_id === roomId) {
          clearTimeout(timeout);
          this.ws.removeEventListener('message', handleMessage);
          this.currentRoom = roomId;
          resolve(msg.payload);
        } else if (msg.type === 'Error') {
          clearTimeout(timeout);
          this.ws.removeEventListener('message', handleMessage);
          reject(new Error(msg.payload.message));
        } else if (msg.type === 'OnlineUsers' && msg.payload.room_id === roomId) {
          // 保存在线用户列表
          this.onlineUsers.set(roomId, msg.payload.users);
          console.log('在线用户:', msg.payload.users);
        }
      };

      this.ws.addEventListener('message', handleMessage);

      // 发送加入请求
      this.ws.send(JSON.stringify({
        type: 'JoinRoom',
        payload: { room_id: roomId }
      }));
    });
  }

  // 离开房间
  async leaveRoom(roomId) {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Leave room timeout'));
      }, 10000);

      const handleMessage = (event) => {
        const msg = JSON.parse(event.data);

        if (msg.type === 'RoomLeft' && msg.payload.room_id === roomId) {
          clearTimeout(timeout);
          this.ws.removeEventListener('message', handleMessage);
          this.currentRoom = null;
          this.onlineUsers.delete(roomId);
          resolve(msg.payload);
        }
      };

      this.ws.addEventListener('message', handleMessage);

      this.ws.send(JSON.stringify({
        type: 'LeaveRoom',
        payload: { room_id: roomId }
      }));
    });
  }

  // 处理用户进出通知
  handleUserActivity(msg) {
    switch (msg.type) {
      case 'UserJoined':
        console.log(`用户 ${msg.payload.username} 加入房间`);
        // 更新在线用户列表
        break;
      case 'UserLeft':
        console.log(`用户 ${msg.payload.username} 离开房间`);
        // 更新在线用户列表
        break;
      case 'UserStatusChanged':
        console.log(`用户 ${msg.payload.username} 状态变为 ${msg.payload.status}`);
        break;
    }
  }
}

// 使用示例
const roomManager = new RoomManager(ws);

// 加入房间
await roomManager.joinRoom('550e8400-e29b-41d4-a716-446655440000');

// 监听用户活动
ws.addEventListener('message', (event) => {
  const msg = JSON.parse(event.data);
  roomManager.handleUserActivity(msg);
});
```

### Python 示例

```python
import asyncio
import json
from typing import Optional, List, Dict, Any
import websockets

class RoomManager:
    def __init__(self, ws):
        self.ws = ws
        self.current_room: Optional[str] = None
        self.online_users: Dict[str, List[Dict]] = {}

    async def join_room(self, room_id: str, timeout: float = 10.0) -> Dict[str, Any]:
        """加入房间"""
        # 发送加入请求
        await self.ws.send(json.dumps({
            "type": "JoinRoom",
            "payload": {"room_id": room_id}
        }))

        # 等待响应
        start_time = asyncio.get_event_loop().time()
        while asyncio.get_event_loop().time() - start_time < timeout:
            try:
                msg = json.loads(await asyncio.wait_for(
                    self.ws.recv(),
                    timeout=timeout - (asyncio.get_event_loop().time() - start_time)
                ))

                if msg.get("type") == "RoomJoined":
                    if msg["payload"]["room_id"] == room_id:
                        self.current_room = room_id
                        return msg["payload"]
                elif msg.get("type") == "Error":
                    raise Exception(f"加入房间失败: {msg['payload']['message']}")
                elif msg.get("type") == "OnlineUsers":
                    if msg["payload"]["room_id"] == room_id:
                        self.online_users[room_id] = msg["payload"]["users"]
                        print(f"在线用户: {msg['payload']['users']}")

            except asyncio.TimeoutError:
                break

        raise Exception("加入房间超时")

    async def leave_room(self, room_id: str, timeout: float = 10.0) -> Dict[str, Any]:
        """离开房间"""
        await self.ws.send(json.dumps({
            "type": "LeaveRoom",
            "payload": {"room_id": room_id}
        }))

        start_time = asyncio.get_event_loop().time()
        while asyncio.get_event_loop().time() - start_time < timeout:
            try:
                msg = json.loads(await asyncio.wait_for(
                    self.ws.recv(),
                    timeout=timeout - (asyncio.get_event_loop().time() - start_time)
                ))

                if msg.get("type") == "RoomLeft":
                    if msg["payload"]["room_id"] == room_id:
                        self.current_room = None
                        self.online_users.pop(room_id, None)
                        return msg["payload"]

            except asyncio.TimeoutError:
                break

        raise Exception("离开房间超时")

    def handle_user_activity(self, msg: Dict[str, Any]):
        """处理用户进出通知"""
        msg_type = msg.get("type")
        payload = msg.get("payload", {})

        if msg_type == "UserJoined":
            print(f"用户 {payload['username']} 加入房间")
        elif msg_type == "UserLeft"::
            print(f"用户 {payload['username']} 离开房间")
        elif msg_type == "UserStatusChanged":
            print(f"用户 {payload['username']} 状态变为 {payload['status']}")


# 使用示例
async def main():
    async with websockets.connect("ws://localhost:8080/ws") as ws:
        # 先认证
        await ws.send(json.dumps({
            "type": "Auth",
            "payload": {"token": "your_token"}
        }))

        # 等待认证结果
        auth_result = json.loads(await ws.recv())
        if not auth_result["payload"]["success"]:
            raise Exception("认证失败")

        # 创建房间管理器
        room_manager = RoomManager(ws)

        # 加入房间
        await room_manager.join_room("550e8400-e29b-41d4-a716-446655440000")

        # 监听消息
        async for message in ws:
            msg = json.loads(message)
            room_manager.handle_user_activity(msg)

asyncio.run(main())
```

---

## 注意事项

1. **认证要求**: 所有房间管理操作都需要先完成认证
2. **房间成员**: 私有房间需要是成员才能加入，公开房间会自动加入
3. **广播机制**: 用户进出房间会广播给房间内所有其他成员
4. **状态同步**: 在线用户列表在加入房间时发送，后续通过 UserJoined/UserLeft 更新
5. **断线重连**: 重连后会收到 `rooms_to_rejoin` 列表，需要重新加入房间

---

## 相关文档

- [auth.md](./auth.md) - 认证和心跳机制
- [message.md](./message.md) - 消息通信
