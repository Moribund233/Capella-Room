# WebSocket 消息通信

本文档描述 WebSocket 消息通信相关的消息类型和交互流程。

---

## 概述

消息通信是实时聊天系统的核心功能，包括：
- 发送/接收聊天消息
- 回复消息
- 编辑/删除消息
- 消息已读确认
- 正在输入状态

---

## 消息类型

### 客户端发送的消息

| 消息类型 | 说明 | 需要认证 | 需要在房间中 |
|----------|------|----------|--------------|
| `ChatMessage` | 发送聊天消息 | ✅ | ✅ |
| `Typing` | 正在输入状态 | ✅ | ✅ |
| `StopTyping` | 停止输入状态 | ✅ | ✅ |
| `MessageRead` | 消息已读确认 | ✅ | - |
| `EditMessage` | 编辑消息 | ✅ | - |
| `DeleteMessage` | 删除消息 | ✅ | - |
| `GetMissedMessages` | 获取离线消息 | ✅ | ✅ |

### 服务端发送的消息

| 消息类型 | 说明 |
|----------|------|
| `NewMessage` | 新消息广播 |
| `UserTyping` | 用户正在输入（广播） |
| `UserStopTyping` | 用户停止输入（广播） |
| `MessageReadReceipt` | 消息已读回执 |
| `MessageEdited` | 消息已编辑通知 |
| `MessageDeleted` | 消息已删除通知 |
| `MissedMessages` | 离线消息列表 |
| `Mentioned` | @提及通知 |
| `Error` | 错误响应 |

---

## 发送消息

### 请求消息

```json
{
  "type": "ChatMessage",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Hello, everyone!",
    "reply_to": null
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 固定为 "ChatMessage" |
| payload.room_id | string (UUID) | 是 | 房间 ID |
| payload.content | string | 是 | 消息内容（1-2000 字符） |
| payload.reply_to | string (UUID) \| null | 否 | 回复的消息 ID |

### 成功响应

消息发送成功后，会广播给房间内所有成员（包括发送者自己）：

```json
{
  "type": "NewMessage",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001",
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "sender_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "sender_name": "user123",
    "content": "Hello, everyone!",
    "reply_to": null,
    "reply_to_message": null,
    "created_at": "2026-04-26T10:30:00.000Z"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| message_id | string (UUID) | 消息唯一 ID |
| room_id | string (UUID) | 房间 ID |
| sender_id | string (UUID) | 发送者 ID |
| sender_name | string | 发送者名称 |
| content | string | 消息内容 |
| reply_to | string (UUID) \| null | 回复的消息 ID |
| reply_to_message | object \| null | 被回复消息的详细信息 |
| created_at | string (ISO 8601) | 创建时间 |

### 回复消息

发送回复消息时，需要指定 `reply_to` 字段：

```json
{
  "type": "ChatMessage",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Thanks for the info!",
    "reply_to": "660e8400-e29b-41d4-a716-446655440001"
  }
}
```

回复消息的响应会包含 `reply_to_message` 字段：

```json
{
  "type": "NewMessage",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440002",
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "sender_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "sender_name": "user123",
    "content": "Thanks for the info!",
    "reply_to": "660e8400-e29b-41d4-a716-446655440001",
    "reply_to_message": {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "sender_id": "550e8400-e29b-41d4-a716-446655440002",
      "sender_name": "user456",
      "content": "Here's the document...",
      "created_at": "2026-04-26T10:25:00.000Z"
    },
    "created_at": "2026-04-26T10:30:00.000Z"
  }
}
```

### 错误响应

| 错误码 | 说明 | 场景 |
|--------|------|------|
| `INVALID_CONTENT` | 内容无效 | 消息为空或超过长度限制 |
| `NOT_IN_ROOM` | 不在房间中 | 用户未加入该房间 |
| `INVALID_REPLY` | 无效的回复 | 回复的消息不存在或不在同一房间 |
| `SAVE_FAILED` | 保存失败 | 数据库操作失败 |

**示例**:

```json
{
  "type": "Error",
  "payload": {
    "code": "NOT_IN_ROOM",
    "message": "You are not in this room"
  }
}
```

---

## 正在输入状态

### 发送正在输入

```json
{
  "type": "Typing",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### 广播给其他用户

其他房间成员会收到 `UserTyping` 消息：

```json
{
  "type": "UserTyping",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "username": "user123"
  }
}
```

### 发送停止输入

```json
{
  "type": "StopTyping",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### 广播给其他用户

```json
{
  "type": "UserStopTyping",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "username": "user123"
  }
}
```

---

## 消息已读

### 发送已读确认

```json
{
  "type": "MessageRead",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001"
  }
}
```

### 已读回执

```json
{
  "type": "MessageReadReceipt",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001",
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3"
  }
}
```

---

## 编辑消息

### 发送编辑请求

```json
{
  "type": "EditMessage",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001",
    "new_content": "Hello, everyone! (edited)"
  }
}
```

### 编辑成功通知

```json
{
  "type": "MessageEdited",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001",
    "new_content": "Hello, everyone! (edited)",
    "edited_at": "2026-04-26T10:35:00.000Z"
  }
}
```

---

## 删除消息

### 发送删除请求

```json
{
  "type": "DeleteMessage",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001"
  }
}
```

### 删除成功通知

```json
{
  "type": "MessageDeleted",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001"
  }
}
```

---

## 获取离线消息

### 请求消息

```json
{
  "type": "GetMissedMessages",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "last_message_id": "660e8400-e29b-41d4-a716-446655440000"
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| room_id | string (UUID) | 是 | 房间 ID |
| last_message_id | string (UUID) \| null | 否 | 上次接收的消息 ID，null 表示获取所有 |

### 响应消息

```json
{
  "type": "MissedMessages",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "messages": [
      {
        "message_id": "660e8400-e29b-41d4-a716-446655440001",
        "room_id": "550e8400-e29b-41d4-a716-446655440000",
        "sender_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
        "sender_name": "user123",
        "content": "Message 1",
        "reply_to": null,
        "reply_to_message": null,
        "created_at": "2026-04-26T10:30:00.000Z"
      }
    ],
    "has_more": false
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| room_id | string (UUID) | 房间 ID |
| messages | array | 消息列表，格式与 NewMessage 相同 |
| has_more | boolean | 是否有更多消息 |

---

## @提及通知

当消息中包含 `@username` 时，被提及的用户会收到 `Mentioned` 通知：

```json
{
  "type": "Mentioned",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001",
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "mentioned_by": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "mentioned_by_name": "user123",
    "content_preview": "Hey @user456, check this out!",
    "created_at": "2026-04-26T10:30:00.000Z"
  }
}
```

---

## 完整示例

### JavaScript 示例

```javascript
class MessageManager {
  constructor(ws) {
    this.ws = ws;
    this.messages = new Map();
    this.typingUsers = new Map();
  }

  // 发送消息
  sendMessage(roomId, content, replyTo = null) {
    this.ws.send(JSON.stringify({
      type: 'ChatMessage',
      payload: {
        room_id: roomId,
        content: content,
        reply_to: replyTo
      }
    }));
  }

  // 发送正在输入
  sendTyping(roomId) {
    this.ws.send(JSON.stringify({
      type: 'Typing',
      payload: { room_id: roomId }
    }));
  }

  // 发送停止输入
  sendStopTyping(roomId) {
    this.ws.send(JSON.stringify({
      type: 'StopTyping',
      payload: { room_id: roomId }
    }));
  }

  // 标记消息已读
  markAsRead(messageId) {
    this.ws.send(JSON.stringify({
      type: 'MessageRead',
      payload: { message_id: messageId }
    }));
  }

  // 编辑消息
  editMessage(messageId, newContent) {
    this.ws.send(JSON.stringify({
      type: 'EditMessage',
      payload: {
        message_id: messageId,
        new_content: newContent
      }
    }));
  }

  // 删除消息
  deleteMessage(messageId) {
    this.ws.send(JSON.stringify({
      type: 'DeleteMessage',
      payload: { message_id: messageId }
    }));
  }

  // 获取离线消息
  getMissedMessages(roomId, lastMessageId = null) {
    this.ws.send(JSON.stringify({
      type: 'GetMissedMessages',
      payload: {
        room_id: roomId,
        last_message_id: lastMessageId
      }
    }));
  }

  // 处理收到的消息
  handleMessage(msg) {
    switch (msg.type) {
      case 'NewMessage':
        this.handleNewMessage(msg.payload);
        break;
      case 'UserTyping':
        this.handleUserTyping(msg.payload);
        break;
      case 'UserStopTyping':
        this.handleUserStopTyping(msg.payload);
        break;
      case 'MessageEdited':
        this.handleMessageEdited(msg.payload);
        break;
      case 'MessageDeleted':
        this.handleMessageDeleted(msg.payload);
        break;
      case 'Mentioned':
        this.handleMentioned(msg.payload);
        break;
      case 'MissedMessages':
        this.handleMissedMessages(msg.payload);
        break;
    }
  }

  handleNewMessage(payload) {
    console.log(`[${payload.sender_name}] ${payload.content}`);
    this.messages.set(payload.message_id, payload);

    // 如果是回复消息，显示被回复的内容
    if (payload.reply_to_message) {
      console.log(`  ↳ 回复: ${payload.reply_to_message.content}`);
    }
  }

  handleUserTyping(payload) {
    this.typingUsers.set(payload.user_id, payload.username);
    this.updateTypingIndicator();
  }

  handleUserStopTyping(payload) {
    this.typingUsers.delete(payload.user_id);
    this.updateTypingIndicator();
  }

  updateTypingIndicator() {
    const users = Array.from(this.typingUsers.values());
    if (users.length > 0) {
      console.log(`${users.join(', ')} 正在输入...`);
    }
  }

  handleMentioned(payload) {
    console.log(`你被 ${payload.mentioned_by_name} 提及了!`);
    console.log(`内容: ${payload.content_preview}`);
  }

  handleMissedMessages(payload) {
    console.log(`收到 ${payload.messages.length} 条离线消息`);
    payload.messages.forEach(msg => {
      this.messages.set(msg.message_id, msg);
    });
  }
}

// 使用示例
const msgManager = new MessageManager(ws);

ws.addEventListener('message', (event) => {
  const msg = JSON.parse(event.data);
  msgManager.handleMessage(msg);
});

// 发送消息
msgManager.sendMessage('room-id', 'Hello!');

// 回复消息
msgManager.sendMessage('room-id', 'Thanks!', 'message-id-to-reply');

// 发送正在输入（可以配合防抖使用）
msgManager.sendTyping('room-id');
```

### Python 示例

```python
import asyncio
import json
from typing import Optional, Dict, Any, List
import websockets

class MessageManager:
    def __init__(self, ws):
        self.ws = ws
        self.messages: Dict[str, Any] = {}
        self.typing_users: Dict[str, str] = {}

    async def send_message(self, room_id: str, content: str, reply_to: str = None):
        """发送消息"""
        payload = {
            "type": "ChatMessage",
            "payload": {
                "room_id": room_id,
                "content": content,
                "reply_to": reply_to
            }
        }
        await self.ws.send(json.dumps(payload))

    async def send_typing(self, room_id: str):
        """发送正在输入"""
        await self.ws.send(json.dumps({
            "type": "Typing",
            "payload": {"room_id": room_id}
        }))

    async def send_stop_typing(self, room_id: str):
        """发送停止输入"""
        await self.ws.send(json.dumps({
            "type": "StopTyping",
            "payload": {"room_id": room_id}
        }))

    async def mark_as_read(self, message_id: str):
        """标记消息已读"""
        await self.ws.send(json.dumps({
            "type": "MessageRead",
            "payload": {"message_id": message_id}
        }))

    async def edit_message(self, message_id: str, new_content: str):
        """编辑消息"""
        await self.ws.send(json.dumps({
            "type": "EditMessage",
            "payload": {
                "message_id": message_id,
                "new_content": new_content
            }
        }))

    async def delete_message(self, message_id: str):
        """删除消息"""
        await self.ws.send(json.dumps({
            "type": "DeleteMessage",
            "payload": {"message_id": message_id}
        }))

    async def get_missed_messages(self, room_id: str, last_message_id: str = None):
        """获取离线消息"""
        await self.ws.send(json.dumps({
            "type": "GetMissedMessages",
            "payload": {
                "room_id": room_id,
                "last_message_id": last_message_id
            }
        }))

    def handle_message(self, msg: Dict[str, Any]):
        """处理收到的消息"""
        msg_type = msg.get("type")
        payload = msg.get("payload", {})

        handlers = {
            "NewMessage": self._handle_new_message,
            "UserTyping": self._handle_user_typing,
            "UserStopTyping": self._handle_user_stop_typing,
            "MessageEdited": self._handle_message_edited,
            "MessageDeleted": self._handle_message_deleted,
            "Mentioned": self._handle_mentioned,
            "MissedMessages": self._handle_missed_messages,
        }

        handler = handlers.get(msg_type)
        if handler:
            handler(payload)

    def _handle_new_message(self, payload: Dict[str, Any]):
        """处理新消息"""
        print(f"[{payload['sender_name']}] {payload['content']}")
        self.messages[payload['message_id']] = payload

        if payload.get('reply_to_message'):
            reply = payload['reply_to_message']
            print(f"  ↳ 回复: {reply['content']}")

    def _handle_user_typing(self, payload: Dict[str, Any]):
        """处理正在输入"""
        self.typing_users[payload['user_id']] = payload['username']
        self._update_typing_indicator()

    def _handle_user_stop_typing(self, payload: Dict[str, Any]):
        """处理停止输入"""
        self.typing_users.pop(payload['user_id'], None)
        self._update_typing_indicator()

    def _update_typing_indicator(self):
        """更新输入指示器"""
        if self.typing_users:
            users = ', '.join(self.typing_users.values())
            print(f"{users} 正在输入...")

    def _handle_mentioned(self, payload: Dict[str, Any]):
        """处理被提及"""
        print(f"你被 {payload['mentioned_by_name']} 提及了!")
        print(f"内容: {payload['content_preview']}")

    def _handle_missed_messages(self, payload: Dict[str, Any]):
        """处理离线消息"""
        messages = payload.get('messages', [])
        print(f"收到 {len(messages)} 条离线消息")
        for msg in messages:
            self.messages[msg['message_id']] = msg


# 使用示例
async def main():
    async with websockets.connect("ws://localhost:8080/ws") as ws:
        # 先认证
        await ws.send(json.dumps({
            "type": "Auth",
            "payload": {"token": "your_token"}
        }))

        # 创建消息管理器
        msg_manager = MessageManager(ws)

        # 监听消息
        async for message in ws:
            msg = json.loads(message)
            msg_manager.handle_message(msg)

asyncio.run(main())
```

---

## 注意事项

1. **消息长度限制**: 消息内容限制在 1-2000 字符之间
2. **回复消息**: 回复的消息必须存在于同一房间
3. **广播机制**: 新消息会广播给房间内所有成员（包括发送者）
4. **@提及**: 消息中包含 `@username` 会自动发送提及通知
5. **离线消息**: 断线重连后应调用 `GetMissedMessages` 获取离线期间的消息
6. **输入状态**: 建议配合防抖使用，避免频繁发送

---

## 相关文档

- [auth.md](./auth.md) - 认证和心跳机制
- [room.md](./room.md) - 房间管理
