# WebSocket 系统广播消息

本文档描述 WebSocket 系统广播消息的使用场景和交互流程。

---

## 概述

系统广播消息（`SystemMessage`）用于在聊天房间内显示系统级别的通知，与 `SystemNotification`（全局通知弹窗）不同，`SystemMessage` 直接显示在聊天消息流中。

### 与 SystemNotification 的区别

| 特性 | SystemMessage | SystemNotification |
|------|---------------|-------------------|
| 显示位置 | 聊天消息框 | 全局通知弹窗/通知中心 |
| 存储方式 | 不存储，仅实时广播 | 存储到数据库，支持离线同步 |
| 使用场景 | 房间级事件（成员进出、管理员操作） | 用户级事件（私信、@提及、系统告警） |
| 目标用户 | 房间内所有在线成员 | 指定用户或全体用户 |

---

## 消息类型

### 服务端发送的消息

| 消息类型 | 说明 |
|----------|------|
| `SystemMessage` | 系统广播消息（显示在聊天框） |
| `RoomUpdated` | 房间信息更新通知 |
| `SessionRestored` | 会话恢复完成通知 |

---

## 系统广播消息

当房间内发生系统级事件时，会广播 `SystemMessage` 给房间内所有在线成员：

```json
{
  "type": "SystemMessage",
  "payload": {
    "content": "系统管理员已将用户 user123 移出房间"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| content | string | 系统消息内容 |

### 使用场景

1. **管理员删除消息**
   - 触发时机：管理员调用 `DELETE /api/v1/admin/messages/:message_id`
   - 消息示例：`"系统管理员撤回了用户 user123 的消息"`

2. **管理员踢出成员**
   - 触发时机：管理员调用 `DELETE /api/v1/admin/rooms/:room_id/members/:user_id`
   - 消息示例：`"系统管理员已将用户 user123 移出房间"`

3. **管理员设置成员角色**
   - 触发时机：管理员调用 `PUT /api/v1/admin/rooms/:room_id/members/:user_id/role`
   - 消息示例：`"系统管理员已将用户 user123 设置为管理员"`

---

## 房间信息更新通知

当房间信息被修改时，会广播 `RoomUpdated` 给房间内所有成员：

```json
{
  "type": "RoomUpdated",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "新的房间名称",
    "description": "新的房间描述"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| room_id | string (UUID) | 房间 ID |
| name | string \| null | 新的房间名称（如有变更） |
| description | string \| null | 新的房间描述（如有变更） |

### 使用场景

- 房间所有者或管理员修改房间信息时
- 用于实时同步房间信息变更到所有在线成员

---

## 会话恢复完成通知

当客户端断线重连并成功恢复会话后，服务端会发送 `SessionRestored`：

```json
{
  "type": "SessionRestored",
  "payload": {
    "restored_at": "2026-04-26T10:30:00.000Z",
    "rooms_restored": 3,
    "total_unread": 15
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| restored_at | string (ISO 8601) | 恢复完成时间 |
| rooms_restored | number | 成功恢复的房间数量 |
| total_unread | number | 恢复期间累计的未读消息数 |

### 使用场景

- 客户端断线后使用 `Reconnect` 消息重连成功
- 服务端自动恢复房间订阅后通知客户端
- 客户端可根据此消息更新未读消息计数

---

## 前端处理建议

### SystemMessage 显示

```javascript
// 示例：在聊天消息列表中显示系统消息
function renderSystemMessage(message) {
  return (
    <div className="system-message">
      <span className="system-icon">🔔</span>
      <span className="system-content">{message.content}</span>
      <span className="system-time">{formatTime(message.timestamp)}</span>
    </div>
  );
}
```

### RoomUpdated 处理

```javascript
// 示例：更新房间信息
socket.on('RoomUpdated', (data) => {
  if (data.name) {
    updateRoomName(data.room_id, data.name);
  }
  if (data.description) {
    updateRoomDescription(data.room_id, data.description);
  }
});
```

### SessionRestored 处理

```javascript
// 示例：显示恢复提示
socket.on('SessionRestored', (data) => {
  showToast(`会话已恢复，${data.total_unread} 条新消息`);
  updateUnreadCount(data.total_unread);
});
```

---

## 阶段归属

- **阶段 4 (WebSocket 通信)**: 基础消息广播机制
- **阶段 5 (消息系统)**: 消息显示和同步
- **阶段 8 (配置化与运维管理)**: 管理员操作系统广播
