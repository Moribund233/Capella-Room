# 通知接口文档

> **API 前缀**: `/api/v1`
> **认证要求**: 所有接口均需要认证（需要携带 Access Token）

## 概述

通知系统用于管理用户的各类通知，包括：
- @提及通知
- 私信通知
- 房间邀请通知
- 系统通知
- 文件上传完成通知
- 待办通知

## 架构说明

通知系统采用 **双写模式** + **HTTP API** 架构：

1. **后端双写**: 所有通知先写入数据库，再推送 WebSocket（如果用户在线）
2. **WebSocket**: 仅用于实时推送新通知
3. **HTTP API**: 用于获取通知列表、标记已读/未读

这种架构确保：
- 通知不丢失（持久化存储）
- 多设备同步
- 支持历史查询
- 页面刷新后通知仍然显示

---

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/notifications` | 获取通知列表 |
| GET | `/api/v1/notifications/unread-count` | 获取未读通知数量 |
| POST | `/api/v1/notifications/:id/read` | 标记通知已读 |
| POST | `/api/v1/notifications/read-all` | 标记所有通知已读 |

---

## 获取通知列表

获取当前用户的通知列表，支持分页和筛选。

### 请求

```http
GET /api/v1/notifications?unread_only=true&limit=50&offset=0
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| unread_only | boolean | 否 | 是否只返回未读通知，默认 `true` |
| limit | number | 否 | 返回数量限制，默认 50，最大 100 |
| offset | number | 否 | 分页偏移量，默认 0 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "notifications": [
      {
        "id": "990e8400-e29b-41d4-a716-446655440001",
        "notification_type": "mention",
        "title": "有人提到了你",
        "content": "user123 在 Project Discussion 中提到了你",
        "data": {
          "message_id": "660e8400-e29b-41d4-a716-446655440001",
          "room_id": "550e8400-e29b-41d4-a716-446655440000",
          "mentioned_by": "44777268-d040-4ef5-81de-9aad6ea3ead3",
          "mentioned_by_name": "user123",
          "content_preview": "Hey @user456, check this out!"
        },
        "is_read": false,
        "read_at": null,
        "created_at": "2026-04-26T10:30:00.000Z"
      }
    ],
    "unread_count": 5,
    "has_more": false
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| notifications | array | 通知列表 |
| notifications[].id | string (UUID) | 通知唯一 ID |
| notifications[].notification_type | string | 通知类型: `mention`, `private_message`, `room_invitation`, `system`, `file_upload`, `pending_action` |
| notifications[].title | string \| null | 通知标题 |
| notifications[].content | string | 通知内容 |
| notifications[].data | object | 附加数据，根据通知类型不同 |
| notifications[].is_read | boolean | 是否已读 |
| notifications[].read_at | string (ISO 8601) \| null | 读取时间 |
| notifications[].created_at | string (ISO 8601) | 创建时间 |
| unread_count | number | 未读通知总数 |
| has_more | boolean | 是否还有更多通知 |

### 通知类型说明

| 类型 | 说明 | data 字段 |
|------|------|-----------|
| `mention` | @提及通知 | `message_id`, `room_id`, `mentioned_by`, `mentioned_by_name`, `content_preview` |
| `private_message` | 私信通知 | `message_id`, `sender_id`, `sender_name`, `content` |
| `room_invitation` | 房间邀请 | `invitation_id`, `room_id`, `room_name`, `invited_by`, `invited_by_name` |
| `system` | 系统通知 | `notification_type`, `title`, `content`, `data` |
| `file_upload` | 文件上传完成 | `file_id`, `file_name`, `file_url`, `file_size` |
| `pending_action` | 待办通知 | `action_type`, `title`, `description`, `deadline` |

---

## 获取未读通知数量

获取当前用户的未读通知总数。

### 请求

```http
GET /api/v1/notifications/unread-count
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "count": 5
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| count | number | 未读通知数量 |

---

## 标记通知已读

将指定通知标记为已读。

### 请求

```http
POST /api/v1/notifications/:id/read
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| id | string (UUID) | 通知 ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": null
}
```

**失败 - 通知不存在 (404 Not Found)**

```json
{
  "success": false,
  "error": {
    "code": "NOTIFICATION_NOT_FOUND",
    "message": "通知不存在"
  }
}
```

**失败 - 无权访问 (403 Forbidden)**

```json
{
  "success": false,
  "error": {
    "code": "FORBIDDEN",
    "message": "无权访问该通知"
  }
}
```

---

## 标记所有通知已读

将当前用户的所有未读通知标记为已读。

### 请求

```http
POST /api/v1/notifications/read-all
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "marked_count": 5
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| marked_count | number | 被标记为已读的通知数量 |

---

## 使用流程

### 页面加载时获取通知

```javascript
// 1. 获取未读通知数量（用于显示 badge）
const countResponse = await fetch('/api/v1/notifications/unread-count');
const unreadCount = countResponse.data.count;

// 2. 获取通知列表
const listResponse = await fetch('/api/v1/notifications?unread_only=true&limit=50');
const notifications = listResponse.data.notifications;
```

### 实时接收新通知

```javascript
// WebSocket 连接后，订阅通知消息
ws.onMessage('Mentioned', (payload) => {
  // 收到新的 @提及通知
  // 1. 更新未读计数
  unreadCount++;
  // 2. 添加到通知列表
  notifications.unshift(payload);
});
```

### 标记通知已读

```javascript
// 点击通知时标记已读
async function markAsRead(notificationId) {
  await fetch(`/api/v1/notifications/${notificationId}/read`, {
    method: 'POST'
  });
  // 更新本地状态
  unreadCount--;
}

// 标记所有已读
async function markAllAsRead() {
  await fetch('/api/v1/notifications/read-all', {
    method: 'POST'
  });
  // 更新本地状态
  unreadCount = 0;
}
```

---

## 与 WebSocket 通知的关系

| 功能 | HTTP API | WebSocket |
|------|----------|-----------|
| 获取通知列表 | ✅ 主要方式 | ⚠️ 已弃用 (`GetOfflineNotifications`) |
| 标记已读 | ✅ 主要方式 | ❌ 已移除 |
| 实时推送新通知 | ❌ 不支持 | ✅ 主要方式 |

**推荐用法**:
1. 页面加载时使用 HTTP API 获取历史通知
2. 通过 WebSocket 实时接收新通知
3. 使用 HTTP API 标记通知已读
