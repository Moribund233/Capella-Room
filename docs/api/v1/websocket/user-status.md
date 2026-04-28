# 用户状态管理

## 概述

用户状态管理允许用户设置自己的在线状态，并查看其他用户的状态。系统支持四种在线状态：在线、离开、忙碌、离线。

> **注意**：用户状态与账号状态是独立的两个概念：
> - **在线状态**（`status`）：表示用户当前的在线状态，包括 `online`/`away`/`busy`/`offline`
> - **账号状态**（`is_active`）：表示账号是否被启用，由管理员控制。被禁用的账号（`is_active: false`）无法登录系统

## 用户状态类型

| 状态 | 说明 | 行为 |
|------|------|------|
| `online` | 在线 | 正常接收消息和通知 |
| `away` | 离开 | 接收消息，但可能延迟响应 |
| `busy` | 忙碌 | 接收消息，但减少通知打扰 |
| `offline` | 离线 | 不接收实时消息，消息存储为离线通知 |

## 更新用户状态

### 请求消息

```json
{
  "type": "UpdateStatus",
  "payload": {
    "status": "away"
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| status | string | 是 | 在线状态：`online`/`away`/`busy`/`offline` |

### 成功响应

状态更新成功后，会广播给该用户所在的所有房间：

```json
{
  "type": "UserStatusChanged",
  "payload": {
    "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "username": "user123",
    "status": "away"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| user_id | string (UUID) | 用户 ID |
| username | string | 用户名 |
| status | string | 新状态 |

### 错误响应

```json
{
  "type": "Error",
  "payload": {
    "code": "STATUS_UPDATE_FAILED",
    "message": "Failed to update status"
  }
}
```

## 获取全局在线用户列表

### 请求消息

```json
{
  "type": "GetOnlineUsers"
}
```

> **注意**: `GetOnlineUsers` 是 **Unit Variant**，不需要 `payload` 字段

### 成功响应

```json
{
  "type": "GlobalOnlineUsers",
  "payload": {
    "users": [
      {
        "id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
        "username": "user123",
        "avatar_url": null,
        "status": "online"
      },
      {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "user456",
        "avatar_url": "https://example.com/avatar.jpg",
        "status": "busy"
      }
    ],
    "total": 2
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| users | array | 在线用户列表 |
| users[].id | string (UUID) | 用户 ID |
| users[].username | string | 用户名 |
| users[].avatar_url | string \| null | 头像 URL |
| users[].status | string | 在线状态 |
| total | number | 在线用户总数 |

> **注意**: 返回的在线用户数量限制为 100 个

## 接收其他用户状态变更

当其他用户更新状态时，你会收到 `UserStatusChanged` 通知：

```json
{
  "type": "UserStatusChanged",
  "payload": {
    "user_id": "550e8400-e29b-41d4-a716-446655440001",
    "username": "user456",
    "status": "busy"
  }
}
```

**触发场景**:
- 用户主动更新状态
- 用户加入房间
- 用户离开房间
- 用户断线（自动变为 offline）
- 用户重连（恢复之前状态）

## 状态变更流程

```
┌─────────────┐     UpdateStatus      ┌─────────────┐
│   客户端     │ ─────────────────────> │   服务端     │
│  (user123)   │                        │             │
└─────────────┘                        └──────┬──────┘
                                              │
                                              │ 1. 更新数据库
                                              │ 2. 广播状态变更
                                              │
                       UserStatusChanged      ▼
┌─────────────┐     <──────────────────  ┌─────────────┐
│   客户端     │                          │   房间成员   │
│  (user456)   │                          │  (所有房间)  │
└─────────────┘                          └─────────────┘
```

## 最佳实践

### 1. 状态切换建议

```javascript
// 根据用户活动自动切换状态
class UserStatusManager {
  constructor(ws) {
    this.ws = ws;
    this.idleTimer = null;
    this.IDLE_TIMEOUT = 5 * 60 * 1000; // 5分钟无操作视为离开
  }

  // 用户活动时重置计时器
  onUserActivity() {
    if (this.idleTimer) {
      clearTimeout(this.idleTimer);
    }
    
    // 如果当前是 away 状态，切回 online
    this.updateStatus('online');
    
    // 设置新的空闲计时器
    this.idleTimer = setTimeout(() => {
      this.updateStatus('away');
    }, this.IDLE_TIMEOUT);
  }

  // 手动设置状态
  setStatus(status) {
    this.updateStatus(status);
  }

  updateStatus(status) {
    this.ws.send(JSON.stringify({
      type: 'UpdateStatus',
      payload: { status }
    }));
  }

  // 页面可见性变化时更新状态
  handleVisibilityChange() {
    if (document.hidden) {
      this.updateStatus('away');
    } else {
      this.updateStatus('online');
      this.onUserActivity();
    }
  }
}
```

### 2. 状态显示 UI

```javascript
function getStatusIcon(status) {
  const icons = {
    online: '🟢',
    away: '🌙',
    busy: '🔴',
    offline: '⚫'
  };
  return icons[status] || '⚪';
}

function getStatusText(status) {
  const texts = {
    online: '在线',
    away: '离开',
    busy: '忙碌',
    offline: '离线'
  };
  return texts[status] || '未知';
}

// 渲染用户状态
function renderUserStatus(user) {
  return `
    <div class="user-status">
      <span class="status-icon">${getStatusIcon(user.status)}</span>
      <span class="username">${user.username}</span>
      <span class="status-text">${getStatusText(user.status)}</span>
    </div>
  `;
}
```

### 3. 在线用户列表管理

```javascript
class OnlineUsersManager {
  constructor(ws) {
    this.ws = ws;
    this.onlineUsers = new Map();
    this.setupListeners();
  }

  setupListeners() {
    // 接收全局在线用户列表
    this.ws.on('GlobalOnlineUsers', (data) => {
      data.users.forEach(user => {
        this.onlineUsers.set(user.id, user);
      });
      this.renderOnlineUsers();
    });

    // 接收状态变更通知
    this.ws.on('UserStatusChanged', (data) => {
      const user = this.onlineUsers.get(data.user_id);
      if (user) {
        user.status = data.status;
        this.updateUserStatus(data.user_id, data.status);
      }
    });
  }

  // 请求在线用户列表
  fetchOnlineUsers() {
    this.ws.send(JSON.stringify({
      type: 'GetOnlineUsers'
    }));
  }

  // 获取用户状态
  getUserStatus(userId) {
    const user = this.onlineUsers.get(userId);
    return user ? user.status : 'offline';
  }

  renderOnlineUsers() {
    // 渲染在线用户列表
  }

  updateUserStatus(userId, status) {
    // 更新特定用户的状态显示
  }
}
```

## 错误处理

| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| `STATUS_UPDATE_FAILED` | 状态更新失败 | 检查网络连接，稍后重试 |
| `UNAUTHENTICATED` | 未认证 | 先完成 WebSocket 认证 |

## 相关文档

- [认证](./auth.md) - WebSocket 连接认证
- [房间管理](./room.md) - 房间内的在线用户列表
