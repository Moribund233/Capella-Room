# WebSocket 通知系统

本文档描述 WebSocket 通知系统相关的消息类型和交互流程。

---

## 概述

通知系统用于实时推送各类通知给客户端，包括：
- 私信通知
- @提及通知
- 房间邀请通知
- 系统通知
- 离线通知同步
- 待办通知

---

## 通知类型

### 服务端推送的通知

| 通知类型 | 说明 | 触发条件 |
|----------|------|----------|
| `PrivateMessage` | 私信通知 | 收到私信时 |
| `Mentioned` | @提及通知 | 被@提及时 |
| `RoomInvitation` | 房间邀请通知 | 被邀请加入房间时 |
| `SystemNotification` | 系统通知 | 系统广播时 |
| `FileUploadComplete` | 文件上传完成 | 文件上传完成时 |
| `PendingAction` | 待办通知 | 有待办事项时 |

### 客户端管理通知的消息

| 消息类型 | 说明 |
|----------|------|
| `GetOfflineNotifications` | 获取离线通知 |
| `MarkNotificationRead` | 标记通知已读 |
| `MarkAllNotificationsRead` | 标记所有通知已读 |
| `RespondPendingAction` | 响应待办通知 |
| `GetPendingActions` | 获取待办列表 |

---

## 私信通知

当收到私信时，会收到 `PrivateMessage` 通知：

```json
{
  "type": "PrivateMessage",
  "payload": {
    "message_id": "660e8400-e29b-41d4-a716-446655440001",
    "sender_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "sender_name": "user123",
    "content": "Hey, are you available?",
    "created_at": "2026-04-26T10:30:00.000Z"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| message_id | string (UUID) | 消息唯一 ID |
| sender_id | string (UUID) | 发送者 ID |
| sender_name | string | 发送者名称 |
| content | string | 消息内容 |
| created_at | string (ISO 8601) | 创建时间 |

---

## @提及通知

当在消息中被 @ 提及时，会收到 `Mentioned` 通知：

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

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| message_id | string (UUID) | 消息唯一 ID |
| room_id | string (UUID) | 房间 ID |
| mentioned_by | string (UUID) | 提及者 ID |
| mentioned_by_name | string | 提及者名称 |
| content_preview | string | 消息内容预览（前 100 字符） |
| created_at | string (ISO 8601) | 创建时间 |

---

## 房间邀请通知

当被邀请加入房间时，会收到 `RoomInvitation` 通知：

```json
{
  "type": "RoomInvitation",
  "payload": {
    "invitation_id": "770e8400-e29b-41d4-a716-446655440001",
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "room_name": "Project Discussion",
    "invited_by": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "invited_by_name": "user123",
    "created_at": "2026-04-26T10:30:00.000Z"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| invitation_id | string (UUID) | 邀请唯一 ID |
| room_id | string (UUID) | 房间 ID |
| room_name | string | 房间名称 |
| invited_by | string (UUID) | 邀请者 ID |
| invited_by_name | string | 邀请者名称 |
| created_at | string (ISO 8601) | 创建时间 |

---

## 系统通知

系统广播通知使用 `SystemNotification` 类型：

```json
{
  "type": "SystemNotification",
  "payload": {
    "notification_type": "important",
    "title": "系统维护通知",
    "content": "系统将于今晚 22:00 进行维护，预计持续 2 小时。",
    "data": null,
    "created_at": "2026-04-26T10:30:00.000Z"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| notification_type | string | 通知类型: `new`, `important`, `warning` |
| title | string | 通知标题 |
| content | string | 通知内容 |
| data | object \| null | 附加数据 |
| created_at | string (ISO 8601) | 创建时间 |

### 通知类型说明

| 类型 | 说明 |
|------|------|
| `new` | 新功能、新版本通知 |
| `important` | 重要公告 |
| `warning` | 警告、维护通知 |

---

## 文件上传完成通知

当文件上传完成时，会收到 `FileUploadComplete` 通知：

```json
{
  "type": "FileUploadComplete",
  "payload": {
    "file_id": "880e8400-e29b-41d4-a716-446655440001",
    "file_name": "document.pdf",
    "file_url": "/uploads/files/2026/04/document.pdf",
    "file_size": 1048576,
    "uploaded_at": "2026-04-26T10:30:00.000Z"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| file_id | string (UUID) | 文件唯一 ID |
| file_name | string | 文件名称 |
| file_url | string | 文件访问 URL |
| file_size | number | 文件大小（字节） |
| uploaded_at | string (ISO 8601) | 上传时间 |

---

## 离线通知同步

### 获取离线通知

客户端可以在连接后获取离线期间的通知：

**请求**:

```json
{
  "type": "GetOfflineNotifications",
  "payload": {
    "last_notification_id": null,
    "limit": 50
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| last_notification_id | string (UUID) \| null | 否 | 上次获取的最后通知 ID，null 表示从头获取 |
| limit | number | 否 | 获取数量限制，默认 50 |

**响应**:

```json
{
  "type": "OfflineNotifications",
  "payload": {
    "notifications": [
      {
        "id": "990e8400-e29b-41d4-a716-446655440001",
        "notification_type": "mention",
        "title": null,
        "content": "user123 在 Project Discussion 中提到了你",
        "data": {
          "message_id": "660e8400-e29b-41d4-a716-446655440001",
          "room_id": "550e8400-e29b-41d4-a716-446655440000"
        },
        "created_at": "2026-04-26T10:30:00.000Z"
      }
    ],
    "has_more": false
  }
}
```

---

## 标记通知已读

### 标记单个通知已读

```json
{
  "type": "MarkNotificationRead",
  "payload": {
    "notification_id": "990e8400-e29b-41d4-a716-446655440001"
  }
}
```

**响应**:

```json
{
  "type": "NotificationReadConfirm",
  "payload": {
    "notification_id": "990e8400-e29b-41d4-a716-446655440001"
  }
}
```

### 标记所有通知已读

```json
{
  "type": "MarkAllNotificationsRead"
}
```

> **注意**: 
> - `MarkAllNotificationsRead` 是 **Unit Variant**，不需要 `payload` 字段
> - 标记所有通知已读没有单独的确认响应，操作成功后服务端静默处理

---

## 待办通知系统

### 待办通知

当有需要处理的待办事项时，会收到 `PendingAction` 通知：

```json
{
  "type": "PendingAction",
  "payload": {
    "notification_id": "aa0e8400-e29b-41d4-a716-446655440001",
    "action_type": "config_change",
    "title": "配置变更待确认",
    "description": "系统配置 'max_users' 从 100 变更为 200，需要确认。",
    "deadline": "2026-04-27T10:30:00.000Z",
    "data": {
      "config_key": "max_users",
      "old_value": "100",
      "new_value": "200"
    },
    "created_at": "2026-04-26T10:30:00.000Z"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| notification_id | string (UUID) | 通知唯一 ID |
| action_type | string | 待办类型 |
| title | string | 待办标题 |
| description | string | 待办描述 |
| deadline | string (ISO 8601) \| null | 截止时间 |
| data | object \| null | 附加数据 |
| created_at | string (ISO 8601) | 创建时间 |

### 响应待办通知

```json
{
  "type": "RespondPendingAction",
  "payload": {
    "notification_id": "aa0e8400-e29b-41d4-a716-446655440001",
    "action": "approve",
    "comment": "同意变更"
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| notification_id | string (UUID) | 是 | 待办通知 ID |
| action | string | 是 | 响应动作: `approve`, `reject`, `snooze` |
| comment | string | 否 | 备注说明 |

**响应动作说明**:

| 动作 | 说明 |
|------|------|
| `approve` | 确认执行 |
| `reject` | 拒绝变更 |
| `snooze` | 稍后提醒 |

**响应**:

```json
{
  "type": "PendingActionResponse",
  "payload": {
    "notification_id": "aa0e8400-e29b-41d4-a716-446655440001",
    "success": true,
    "message": "配置变更已确认",
    "new_status": "approved"
  }
}
```

### 获取待办列表

```json
{
  "type": "GetPendingActions",
  "payload": {
    "action_type": null
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| action_type | string \| null | 否 | 筛选特定类型的待办，null 表示获取所有 |

**响应**:

```json
{
  "type": "PendingActionsList",
  "payload": {
    "actions": [
      {
        "notification_id": "aa0e8400-e29b-41d4-a716-446655440001",
        "action_type": "config_change",
        "title": "配置变更待确认",
        "description": "系统配置变更需要确认",
        "deadline": "2026-04-27T10:30:00.000Z",
        "action_status": "pending",
        "created_at": "2026-04-26T10:30:00.000Z"
      }
    ],
    "total": 1
  }
}
```

---

## 完整示例

### JavaScript 示例

```javascript
class NotificationManager {
  constructor(ws) {
    this.ws = ws;
    this.notifications = [];
    this.pendingActions = [];
  }

  // 获取离线通知
  getOfflineNotifications(lastId = null, limit = 50) {
    this.ws.send(JSON.stringify({
      type: 'GetOfflineNotifications',
      payload: {
        last_notification_id: lastId,
        limit: limit
      }
    }));
  }

  // 标记通知已读
  markAsRead(notificationId) {
    this.ws.send(JSON.stringify({
      type: 'MarkNotificationRead',
      payload: { notification_id: notificationId }
    }));
  }

  // 标记所有通知已读
  markAllAsRead() {
    this.ws.send(JSON.stringify({
      type: 'MarkAllNotificationsRead',
      payload: {}
    }));
  }

  // 响应待办通知
  respondPendingAction(notificationId, action, comment = null) {
    this.ws.send(JSON.stringify({
      type: 'RespondPendingAction',
      payload: {
        notification_id: notificationId,
        action: action,
        comment: comment
      }
    }));
  }

  // 获取待办列表
  getPendingActions(actionType = null) {
    this.ws.send(JSON.stringify({
      type: 'GetPendingActions',
      payload: { action_type: actionType }
    }));
  }

  // 处理收到的通知
  handleNotification(msg) {
    switch (msg.type) {
      case 'PrivateMessage':
        this.handlePrivateMessage(msg.payload);
        break;
      case 'Mentioned':
        this.handleMentioned(msg.payload);
        break;
      case 'RoomInvitation':
        this.handleRoomInvitation(msg.payload);
        break;
      case 'SystemNotification':
        this.handleSystemNotification(msg.payload);
        break;
      case 'FileUploadComplete':
        this.handleFileUploadComplete(msg.payload);
        break;
      case 'PendingAction':
        this.handlePendingAction(msg.payload);
        break;
      case 'OfflineNotifications':
        this.handleOfflineNotifications(msg.payload);
        break;
      case 'NotificationReadConfirm':
        this.handleNotificationReadConfirm(msg.payload);
        break;
      case 'PendingActionResponse':
        this.handlePendingActionResponse(msg.payload);
        break;
      case 'PendingActionsList':
        this.handlePendingActionsList(msg.payload);
        break;
    }
  }

  handlePrivateMessage(payload) {
    console.log(`收到私信来自 ${payload.sender_name}: ${payload.content}`);
    this.showNotification('私信', `${payload.sender_name}: ${payload.content}`);
  }

  handleMentioned(payload) {
    console.log(`${payload.mentioned_by_name} 提到了你`);
    this.showNotification('@提及', `${payload.mentioned_by_name}: ${payload.content_preview}`);
  }

  handleRoomInvitation(payload) {
    console.log(`${payload.invited_by_name} 邀请你加入 ${payload.room_name}`);
    this.showNotification('房间邀请', `${payload.invited_by_name} 邀请你加入 ${payload.room_name}`);
  }

  handleSystemNotification(payload) {
    console.log(`[${payload.notification_type}] ${payload.title}: ${payload.content}`);
    this.showNotification(payload.title, payload.content);
  }

  handleFileUploadComplete(payload) {
    console.log(`文件上传完成: ${payload.file_name}`);
    this.showNotification('文件上传', `${payload.file_name} 上传完成`);
  }

  handlePendingAction(payload) {
    console.log(`待办: ${payload.title}`);
    this.pendingActions.push(payload);
    this.showNotification('待办事项', payload.title);
  }

  handleOfflineNotifications(payload) {
    console.log(`收到 ${payload.notifications.length} 条离线通知`);
    this.notifications.push(...payload.notifications);
  }

  handleNotificationReadConfirm(payload) {
    console.log(`通知 ${payload.notification_id} 已标记为已读`);
  }

  handlePendingActionResponse(payload) {
    if (payload.success) {
      console.log(`待办响应成功: ${payload.message}`);
    } else {
      console.error(`待办响应失败: ${payload.message}`);
    }
  }

  handlePendingActionsList(payload) {
    console.log(`待办列表: ${payload.total} 项`);
    this.pendingActions = payload.actions;
  }

  showNotification(title, body) {
    // 使用浏览器通知 API
    if ('Notification' in window && Notification.permission === 'granted') {
      new Notification(title, { body });
    }
  }
}

// 使用示例
const notifManager = new NotificationManager(ws);

ws.addEventListener('message', (event) => {
  const msg = JSON.parse(event.data);
  notifManager.handleNotification(msg);
});

// 连接后获取离线通知
ws.addEventListener('open', () => {
  notifManager.getOfflineNotifications();
});
```

### Python 示例

```python
import asyncio
import json
from typing import Optional, Dict, Any, List
import websockets

class NotificationManager:
    def __init__(self, ws):
        self.ws = ws
        self.notifications: List[Dict] = []
        self.pending_actions: List[Dict] = []

    async def get_offline_notifications(self, last_id: str = None, limit: int = 50):
        """获取离线通知"""
        await self.ws.send(json.dumps({
            "type": "GetOfflineNotifications",
            "payload": {
                "last_notification_id": last_id,
                "limit": limit
            }
        }))

    async def mark_as_read(self, notification_id: str):
        """标记通知已读"""
        await self.ws.send(json.dumps({
            "type": "MarkNotificationRead",
            "payload": {"notification_id": notification_id}
        }))

    async def mark_all_as_read(self):
        """标记所有通知已读"""
        await self.ws.send(json.dumps({
            "type": "MarkAllNotificationsRead",
            "payload": {}
        }))

    async def respond_pending_action(self, notification_id: str, action: str, comment: str = None):
        """响应待办通知"""
        payload = {
            "type": "RespondPendingAction",
            "payload": {
                "notification_id": notification_id,
                "action": action
            }
        }
        if comment:
            payload["payload"]["comment"] = comment
        await self.ws.send(json.dumps(payload))

    async def get_pending_actions(self, action_type: str = None):
        """获取待办列表"""
        await self.ws.send(json.dumps({
            "type": "GetPendingActions",
            "payload": {"action_type": action_type}
        }))

    def handle_notification(self, msg: Dict[str, Any]):
        """处理收到的通知"""
        msg_type = msg.get("type")
        payload = msg.get("payload", {})

        handlers = {
            "PrivateMessage": self._handle_private_message,
            "Mentioned": self._handle_mentioned,
            "RoomInvitation": self._handle_room_invitation,
            "SystemNotification": self._handle_system_notification,
            "FileUploadComplete": self._handle_file_upload_complete,
            "PendingAction": self._handle_pending_action,
            "OfflineNotifications": self._handle_offline_notifications,
            "NotificationReadConfirm": self._handle_notification_read_confirm,
            "PendingActionResponse": self._handle_pending_action_response,
            "PendingActionsList": self._handle_pending_actions_list,
        }

        handler = handlers.get(msg_type)
        if handler:
            handler(payload)

    def _handle_private_message(self, payload: Dict[str, Any]):
        """处理私信"""
        print(f"收到私信来自 {payload['sender_name']}: {payload['content']}")

    def _handle_mentioned(self, payload: Dict[str, Any]):
        """处理@提及"""
        print(f"{payload['mentioned_by_name']} 提到了你: {payload['content_preview']}")

    def _handle_room_invitation(self, payload: Dict[str, Any]):
        """处理房间邀请"""
        print(f"{payload['invited_by_name']} 邀请你加入 {payload['room_name']}")

    def _handle_system_notification(self, payload: Dict[str, Any]):
        """处理系统通知"""
        print(f"[{payload['notification_type']}] {payload['title']}: {payload['content']}")

    def _handle_file_upload_complete(self, payload: Dict[str, Any]):
        """处理文件上传完成"""
        print(f"文件上传完成: {payload['file_name']} ({payload['file_size']} bytes)")

    def _handle_pending_action(self, payload: Dict[str, Any]):
        """处理待办通知"""
        print(f"待办: {payload['title']}")
        self.pending_actions.append(payload)

    def _handle_offline_notifications(self, payload: Dict[str, Any]):
        """处理离线通知"""
        notifications = payload.get("notifications", [])
        print(f"收到 {len(notifications)} 条离线通知")
        self.notifications.extend(notifications)

    def _handle_notification_read_confirm(self, payload: Dict[str, Any]):
        """处理通知已读确认"""
        print(f"通知 {payload['notification_id']} 已标记为已读")

    def _handle_pending_action_response(self, payload: Dict[str, Any]):
        """处理待办响应"""
        if payload.get("success"):
            print(f"待办响应成功: {payload['message']}")
        else:
            print(f"待办响应失败: {payload['message']}")

    def _handle_pending_actions_list(self, payload: Dict[str, Any]):
        """处理待办列表"""
        actions = payload.get("actions", [])
        print(f"待办列表: {len(actions)} 项")
        self.pending_actions = actions


# 使用示例
async def main():
    async with websockets.connect("ws://localhost:8080/ws") as ws:
        # 先认证
        await ws.send(json.dumps({
            "type": "Auth",
            "payload": {"token": "your_token"}
        }))

        # 创建通知管理器
        notif_manager = NotificationManager(ws)

        # 获取离线通知
        await notif_manager.get_offline_notifications()

        # 监听消息
        async for message in ws:
            msg = json.loads(message)
            notif_manager.handle_notification(msg)

asyncio.run(main())
```

---

## 注意事项

1. **通知持久化**: 通知会持久化到数据库，离线用户可以在重新连接后获取
2. **已读状态**: 已读状态会同步到服务端，多端登录时状态一致
3. **待办截止**: 待办通知有过期时间，过期后可能自动处理或提醒
4. **通知权限**: 浏览器端需要请求通知权限才能显示桌面通知
5. **频率限制**: 某些通知类型可能有频率限制，避免过度推送

---

## 相关文档

- [auth.md](./auth.md) - 认证和心跳机制
- [message.md](./message.md) - 消息通信
