# Capella Room API 文档

> **版本**: v1  
> **API 前缀**: `/api/v1`

## 目录

- [概述](#概述)
- [认证机制](#认证机制)
- [WebSocket 实时通信](#websocket-实时通信)
- [API 模块](#api-模块)
  - [系统接口](#系统接口)
  - [认证接口](#认证接口)
  - [用户接口](#用户接口)
  - [聊天室接口](#聊天室接口)
  - [消息接口](#消息接口)
  - [文件接口](#文件接口)
  - [UI 配置接口](#ui-配置接口)
  - [管理员接口](#管理员接口)
- [WebSocket 接口](#websocket-接口)
---

## 概述

Capella Room 是一个基于 Rust + Axum 构建的实时聊天室后端服务，提供以下核心功能：

- **用户管理**: 注册、登录、资料管理
- **用户设置**: 通知、隐私、消息、语言、无障碍、媒体设置
- **账号安全**: 设备管理、登录历史、单设备登录、设备禁用
- **聊天室**: 创建、加入、离开、成员管理
- **实时消息**: WebSocket 双向通信，支持文本/图片/文件消息
- **消息功能**: 发送、编辑、删除、回复、搜索
- **文件上传**: 头像、图片、文档等
- **管理员系统**: 用户管理、房间管理、系统配置
- **审计系统**: 操作日志、安全告警
- **IP 安全**: 白名单/黑名单管理

### 技术栈

- **框架**: Axum 0.7 (Rust Web 框架)
- **数据库**: PostgreSQL + SQLx
- **缓存**: Redis
- **实时通信**: WebSocket
- **认证**: JWT (JSON Web Token)
- **密码加密**: Argon2

---

## 认证机制

### JWT Token

所有受保护的 API 都需要在请求头中携带 JWT Token：

```http
Authorization: Bearer <access_token>
```

### Token 类型

| Token 类型 | 用途 | 有效期 |
|-----------|------|--------|
| Access Token | 访问受保护资源 | 15 分钟 |
| Refresh Token | 刷新 Access Token | 7 天 |

### 刷新 Token

当 Access Token 过期时，使用 Refresh Token 获取新的 Token 对：

```http
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refresh_token": "<refresh_token>"
}
```

---

## WebSocket 实时通信

### 连接端点

```
ws://localhost:8080/ws
```

### 连接流程

1. 先通过 HTTP 登录获取 JWT Token
2. 建立 WebSocket 连接
3. 发送认证消息：`{ "type": "Auth", "payload": { "token": "..." } }`
4. 等待 `AuthResult` 响应
5. 加入房间进行实时通信

### WebSocket 消息类型

| 类别 | 消息类型 | 方向 | 说明 |
|------|---------|------|------|
| **连接管理** | `Auth` | C→S | 连接认证 |
| | `AuthResult` | S→C | 认证结果 |
| | `Ping` / `Pong` | 双向 | 心跳保活 |
| | `Reconnect` | C→S | 断线重连 |
| **房间管理** | `JoinRoom` | C→S | 加入房间 |
| | `LeaveRoom` | C→S | 离开房间 |
| | `RoomJoined` / `RoomLeft` | S→C | 加入/离开结果 |
| | `UserJoined` / `UserLeft` | S→C | 用户进出通知 |
| | `OnlineUsers` | S→C | 在线用户列表 |
| **消息通信** | `ChatMessage` | C→S | 发送消息 |
| | `NewMessage` | S→C | 新消息通知 |
| | `EditMessage` | C→S | 编辑消息 |
| | `MessageEdited` | S→C | 消息已编辑通知 |
| | `DeleteMessage` | C→S | 删除消息 |
| | `MessageDeleted` | S→C | 消息已删除通知 |
| | `Typing` / `StopTyping` | C→S | 输入状态 |
| **用户状态** | `UpdateStatus` | C→S | 更新状态 |
| | `UserStatusChanged` | S→C | 用户状态变更 |
| **通知系统** | `PrivateMessage` | S→C | 私信通知 |
| | `Mentioned` | S→C | @提及通知 |
| | `RoomInvitation` | S→C | 房间邀请通知 |
| | `SystemNotification` | S→C | 系统通知 |
| | `FileUploadComplete` | S→C | 文件上传完成通知 |
| | `PendingAction` | S→C | 待办通知 |
| | `GetOfflineNotifications` | C→S | 获取离线通知 (⚠️ 已弃用) |
| | `OfflineNotifications` | S→C | 离线通知列表 |
| | `MarkNotificationRead` | C→S | 标记通知已读 (❌ 已移除) |
| | `MarkAllNotificationsRead` | C→S | 标记所有已读 (❌ 已移除) |
| | `NotificationReadConfirm` | S→C | 已读确认 |
| | `RespondPendingAction` | C→S | 响应待办通知 |
| | `GetPendingActions` | C→S | 获取待办列表 |
| | `PendingActionsList` | S→C | 待办列表响应 |

> **C→S**: Client to Server (客户端发送)  
> **S→C**: Server to Client (服务端推送)
>
> **注意**: 通知的获取和已读标记已迁移到 HTTP API，详见 [通知接口文档](./v1/http/notifications.md)

---

## API 模块

### 系统接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/health` | 否 | 健康检查 |
| GET | `/health/detail` | 否 | 详细健康状态 |
| GET | `/health/ready` | 否 | 就绪检查 (K8s) |
| GET | `/health/live` | 否 | 存活检查 (K8s) |
| GET | `/api/version` | 否 | API 版本信息 |
| GET | `/api/config/client` | 否 | 客户端配置 |

**详细文档**: [system.md](./v1/http/system.md)

---

### 认证接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| POST | `/api/v1/auth/register` | 否 | 用户注册 |
| POST | `/api/v1/auth/login` | 否 | 用户登录 |
| POST | `/api/v1/auth/refresh` | 否 | 刷新 Token |

**详细文档**: [auth.md](./v1/http/auth.md)

---

### 用户接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/users/me` | 是 | 获取当前用户信息 |
| PUT | `/api/v1/users/me` | 是 | 更新当前用户信息 |
| PUT | `/api/v1/users/me/password` | 是 | 修改密码 |
| GET | `/api/v1/users/me/rooms` | 是 | 获取我的聊天室列表 |
| POST | `/api/v1/users/logout` | 是 | 登出 |
| GET | `/api/v1/users` | 是 | 获取用户列表 |
| GET | `/api/v1/users/:user_id` | 是 | 获取指定用户信息 |

#### 用户设置接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/users/me/settings` | 是 | 获取用户设置 |
| PATCH | `/api/v1/users/me/settings` | 是 | 部分更新用户设置 |

#### 账号安全接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/users/me/security/overview` | 是 | 获取账号安全概览 |
| GET | `/api/v1/users/me/devices` | 是 | 获取登录设备列表 |
| DELETE | `/api/v1/users/me/devices/:device_id` | 是 | 登出指定设备 |
| POST | `/api/v1/users/me/devices/:device_id/block` | 是 | 禁用指定设备 |
| POST | `/api/v1/users/me/devices/:device_id/unblock` | 是 | 启用被禁用的设备 |
| POST | `/api/v1/users/me/devices/terminate-others` | 是 | 登出所有其他设备 |
| GET | `/api/v1/users/me/login-history` | 是 | 获取登录历史 |
| GET | `/api/v1/users/me/login-history/suspicious` | 是 | 获取可疑登录记录 |

**详细文档**: [user.md](./v1/http/user.md)

---

### 聊天室接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/rooms` | 是 | 获取聊天室列表 |
| POST | `/api/v1/rooms` | 是 | 创建聊天室 |
| GET | `/api/v1/rooms/recent` | 是 | 获取最近更新的聊天室 |
| GET | `/api/v1/rooms/:room_id` | 是 | 获取聊天室详情 |
| PUT | `/api/v1/rooms/:room_id` | 是 | 更新聊天室信息 |
| DELETE | `/api/v1/rooms/:room_id` | 是 | 删除聊天室 |
| POST | `/api/v1/rooms/:room_id/join` | 是 | 加入聊天室 |
| DELETE | `/api/v1/rooms/:room_id/leave` | 是 | 离开聊天室 |
| GET | `/api/v1/rooms/:room_id/members` | 是 | 获取成员列表 |
| DELETE | `/api/v1/rooms/:room_id/members/:user_id` | 是 | 踢出成员 |
| PUT | `/api/v1/rooms/:room_id/members/:user_id/role` | 是 | 设置成员角色 |
| GET | `/api/v1/rooms/:room_id/messages` | 是 | 获取房间消息历史 |

**详细文档**: [rooms.md](./v1/http/rooms.md)

---

### 消息接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/messages/search` | 是 | 搜索消息 |
| PUT | `/api/v1/messages/:message_id` | 是 | 编辑消息 |
| DELETE | `/api/v1/messages/:message_id` | 是 | 删除消息 |
| GET | `/api/v1/messages/:message_id/history` | 是 | 获取消息编辑历史 |
| POST | `/api/v1/messages/:message_id/reactions` | 是 | 添加表情反应 |
| DELETE | `/api/v1/messages/:message_id/reactions` | 是 | 移除表情反应 |
| GET | `/api/v1/messages/:message_id/reactions` | 是 | 获取消息反应列表 |

> **注意**: 发送消息通过 WebSocket 实时通信

**详细文档**: [messages.md](./v1/http/messages.md)

---

### 文件接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/files` | 是 | 获取文件列表 |
| GET | `/api/v1/files/:file_id` | 是 | 获取文件详情 |
| DELETE | `/api/v1/files/:file_id` | 是 | 删除文件 |
| POST | `/api/v1/upload` | 是 | 上传文件 |
| POST | `/api/v1/upload/image` | 是 | 上传图片 |
| POST | `/api/v1/upload/avatar` | 是 | 上传头像 |

**详细文档**: [files.md](./v1/http/files.md)

---

### UI 配置接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/ui/config` | 是 | 获取用户 UI 配置 |
| POST | `/api/v1/ui/config` | 是 | 保存用户 UI 配置 |
| DELETE | `/api/v1/ui/config` | 是 | 重置用户 UI 配置 |

**详细文档**: [ui-config.md](./v1/http/ui-config.md)

---

### 通知接口

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/api/v1/notifications` | 是 | 获取通知列表 |
| GET | `/api/v1/notifications/unread-count` | 是 | 获取未读通知数量 |
| POST | `/api/v1/notifications/:id/read` | 是 | 标记通知已读 |
| POST | `/api/v1/notifications/read-all` | 是 | 标记所有通知已读 |

**详细文档**: [notifications.md](./v1/http/notifications.md)

> **架构说明**: 通知系统采用双写模式，所有通知先写入数据库，再推送 WebSocket（如果用户在线）。HTTP API 用于获取通知列表和标记已读，WebSocket 仅用于实时推送新通知。

---

### 管理员接口

#### 用户管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/api/v1/admin/users` | Admin+ | 获取所有用户 |
| GET | `/api/v1/admin/users/:user_id` | Admin+ | 获取用户详情 |
| DELETE | `/api/v1/admin/users/:user_id` | Admin+ | 删除用户 |
| PUT | `/api/v1/admin/users/:user_id/role` | Admin+ | 修改用户角色 |
| PUT | `/api/v1/admin/users/:user_id/status` | Admin+ | 设置用户状态 |
| PUT | `/api/v1/admin/users/:user_id/password` | Admin+ | 重置用户密码 |

#### 房间管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/api/v1/admin/rooms` | Admin+ | 获取所有房间 |
| GET | `/api/v1/admin/rooms/:room_id` | Admin+ | 获取房间详情 |
| DELETE | `/api/v1/admin/rooms/:room_id` | Admin+ | 删除房间 |
| GET | `/api/v1/admin/rooms/:room_id/messages` | Admin+ | 获取房间消息 |

#### 消息管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/api/v1/admin/messages` | Admin+ | 获取消息列表 |
| DELETE | `/api/v1/admin/messages/:message_id` | Admin+ | 删除消息 |

#### 系统统计

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/api/v1/admin/stats` | Admin+ | 系统统计 |
| GET | `/api/v1/admin/stats/activity` | Admin+ | 活动统计 |
| GET | `/api/v1/admin/stats/performance` | Admin+ | 性能指标 |

#### 系统配置

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/api/v1/admin/configs` | Admin+ | 获取配置列表 |
| GET | `/api/v1/admin/configs/:key` | Admin+ | 获取配置项 |
| PUT | `/api/v1/admin/configs/:key` | Admin+ | 更新配置项 |
| POST | `/api/v1/admin/configs` | Admin+ | 重置配置 |

#### 审计系统

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/api/v1/admin/audit/logs` | Admin+ | 审计日志列表 |
| GET | `/api/v1/admin/audit/logs/:id` | Admin+ | 审计日志详情 |
| GET | `/api/v1/admin/audit/stats` | Admin+ | 审计统计 |
| GET | `/api/v1/admin/audit/export` | Admin+ | 导出日志 |
| GET | `/api/v1/admin/audit/alerts` | Admin+ | 告警列表 |
| PUT | `/api/v1/admin/audit/alerts/:id/status` | Admin+ | 更新告警状态 |
| GET | `/api/v1/admin/audit/rules` | Admin+ | 告警规则列表 |
| PUT | `/api/v1/admin/audit/rules/:id` | Admin+ | 更新告警规则 |
| POST | `/api/v1/admin/audit/cleanup` | Admin+ | 清理日志 |

#### IP 安全管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/api/v1/admin/security/ip-list` | Admin+ | IP 列表 |
| POST | `/api/v1/admin/security/ip-list` | Admin+ | 添加 IP |
| POST | `/api/v1/admin/security/ip-list/batch` | Admin+ | 批量添加 IP |
| PUT | `/api/v1/admin/security/ip-list/:id` | Admin+ | 更新 IP |
| DELETE | `/api/v1/admin/security/ip-list/:id` | Admin+ | 删除 IP |
| POST | `/api/v1/admin/security/ip-check` | Admin+ | 检查 IP |
| GET | `/api/v1/admin/security/stats` | Admin+ | 安全统计 |
| POST | `/api/v1/admin/security/refresh-cache` | Admin+ | 刷新缓存 |
| POST | `/api/v1/admin/security/cleanup-expired` | Admin+ | 清理过期项 |
| GET | `/api/v1/admin/security/whitelist-mode` | Admin+ | 获取白名单模式 |
| POST | `/api/v1/admin/security/whitelist-mode` | Admin+ | 设置白名单模式 |

#### Redis 与分布式管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/api/v1/admin/redis/status` | Admin+ | Redis 连接状态 |
| POST | `/api/v1/admin/redis/refresh` | SuperAdmin | 刷新 Redis 连接 |
| GET | `/api/v1/admin/redis/stats` | Admin+ | Redis 统计信息 |
| POST | `/api/v1/admin/config/sync` | SuperAdmin | 触发配置同步 |
| GET | `/api/v1/admin/config/sync/status` | Admin+ | 配置同步状态 |

**详细文档**: [admin.md](./v1/http/admin.md)

---

## 架构文档

- **[架构总览](../architecture/README.md)** - 分布式架构、数据库优化、安全加固详细说明

---

## 数据模型

### 统一响应格式

```json
{
  "success": true,
  "data": { ... },
  "message": "操作成功"
}
```

### 分页数据格式

```json
{
  "items": [ ... ],
  "total": 100,
  "limit": 20,
  "offset": 0
}
```

### 核心模型

#### User (用户)

```json
{
  "id": "uuid",
  "username": "string",
  "email": "string",
  "avatar_url": "string?",
  "status": "online | offline | away",
  "is_active": "boolean",
  "role": "user | admin | super_admin",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### Room (聊天室)

```json
{
  "id": "uuid",
  "name": "string",
  "description": "string?",
  "owner": { "id": "uuid", "username": "string", "avatar_url": "string?" },
  "is_private": false,
  "max_members": 100,
  "member_count": 10,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### Message (消息)

```json
{
  "id": "uuid",
  "room_id": "uuid",
  "sender": { "id": "uuid", "username": "string", "avatar_url": "string?" },
  "content": "string",
  "message_type": "text | image | file | system",
  "reply_to": "uuid?",
  "reply_to_message": { ... },
  "is_deleted": false,
  "created_at": "2024-01-01T00:00:00Z",
  "edit_count": 0,
  "edited_at": "2024-01-01T00:00:00Z?"
}
```

#### File (文件)

```json
{
  "id": "uuid",
  "original_name": "string",
  "file_url": "string",
  "file_size": 1024,
  "mime_type": "image/png",
  "category": "image | document | video | audio | other",
  "usage_type": "avatar | message | room_cover | general",
  "uploader": { ... },
  "created_at": "2024-01-01T00:00:00Z"
}
```

---

## 错误码

### HTTP 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 请求成功 |
| 400 | 请求参数错误 |
| 401 | 未认证 |
| 403 | 无权限 |
| 404 | 资源不存在 |
| 409 | 资源冲突 |
| 422 | 验证失败 |
| 429 | 请求过于频繁 |
| 500 | 服务器内部错误 |

### 业务错误码

| 错误码 | 说明 |
|--------|------|
| `AUTH_INVALID_CREDENTIALS` | 用户名或密码错误 |
| `AUTH_TOKEN_EXPIRED` | Token 已过期 |
| `AUTH_TOKEN_INVALID` | Token 无效 |
| `USER_NOT_FOUND` | 用户不存在 |
| `USER_ALREADY_EXISTS` | 用户已存在 |
| `ROOM_NOT_FOUND` | 聊天室不存在 |
| `ROOM_FULL` | 聊天室已满 |
| `ROOM_ALREADY_MEMBER` | 已是聊天室成员 |
| `ROOM_NOT_MEMBER` | 不是聊天室成员 |
| `MESSAGE_NOT_FOUND` | 消息不存在 |
| `MESSAGE_NOT_AUTHOR` | 不是消息作者 |
| `FILE_TOO_LARGE` | 文件过大 |
| `FILE_INVALID_TYPE` | 文件类型不允许 |
| `RATE_LIMIT_EXCEEDED` | 请求频率限制 |
| `IP_BLOCKED` | IP 被阻止 |
| `MAINTENANCE_MODE` | 系统维护中 |

---

## 快速开始

### 1. 注册账号

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'
```

### 2. 登录获取 Token

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'
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

### 3. 创建聊天室

```bash
curl -X POST http://localhost:8080/api/v1/rooms \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -d '{
    "name": "测试聊天室",
    "description": "这是一个测试聊天室",
    "is_private": false,
    "max_members": 100
  }'
```

### 4. 连接 WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  // 发送认证消息
  ws.send(JSON.stringify({
    type: 'Auth',
    payload: { token: '<access_token>' }
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log('收到消息:', msg);
};

// 加入房间
ws.send(JSON.stringify({
  type: 'JoinRoom',
  payload: { room_id: '<room_uuid>' }
}));

// 发送消息
ws.send(JSON.stringify({
  type: 'ChatMessage',
  payload: {
    room_id: '<room_uuid>',
    content: 'Hello, World!',
    reply_to: null
  }
}));
```

---

## 文档索引

| 文档 | 说明 |
|------|------|
| [index.md](./index.md) | 本文档 - API 总览和目录索引 |
| [system.md](./v1/http/system.md) | 系统接口文档 |
| [auth.md](./v1/http/auth.md) | 认证接口文档 |
| [user.md](./v1/http/user.md) | 用户接口文档 |
| [rooms.md](./v1/http/rooms.md) | 聊天室接口文档 |
| [messages.md](./v1/http/messages.md) | 消息接口文档 |
| [files.md](./v1/http/files.md) | 文件接口文档 |
| [ui-config.md](./v1/http/ui-config.md) | UI 配置接口文档 |
| [admin.md](./v1/http/admin.md) | 管理员接口文档 |
| [websocket.md](./v1/websocket/index.md) | WebSocket 协议总览 |

---

## WebSocket 接口

WebSocket 接口提供实时双向通信能力，包括连接管理、房间管理、消息通信、通知系统等功能。

### WebSocket 文档索引

| 文档 | 说明 |
|------|------|
| [v1/websocket/index.md](./v1/websocket/index.md) | WebSocket 协议总览和连接管理 |
| [v1/websocket/auth.md](./v1/websocket/auth.md) | 连接认证和心跳机制 |
| [v1/websocket/room.md](./v1/websocket/room.md) | 房间管理（加入/离开/在线用户） |
| [v1/websocket/message.md](./v1/websocket/message.md) | 消息通信（发送/接收/编辑/删除） |
| [v1/websocket/notification.md](./v1/websocket/notification.md) | 通知系统（私信/@提及/系统通知） |
| [v1/websocket/user-status.md](./v1/websocket/user-status.md) | 用户状态管理 |

### WebSocket 端点

```
ws://localhost:3000/ws
```

### 消息类型概览

| 类别 | 消息类型 | 方向 | 说明 |
|------|---------|------|------|
| **连接管理** | `Auth` / `AuthResult` | 双向 | 连接认证 |
| | `Ping` / `Pong` | 双向 | 心跳保活 |
| | `Reconnect` | C→S | 断线重连 |
| **房间管理** | `JoinRoom` / `LeaveRoom` | C→S | 加入/离开房间 |
| | `RoomJoined` / `RoomLeft` | S→C | 加入/离开结果 |
| | `UserJoined` / `UserLeft` | S→C | 用户进出通知 |
| | `OnlineUsers` | S→C | 在线用户列表 |
| **消息通信** | `ChatMessage` | C→S | 发送消息 |
| | `NewMessage` | S→C | 新消息通知 |
| | `EditMessage` / `MessageEdited` | 双向 | 编辑消息 |
| | `DeleteMessage` / `MessageDeleted` | 双向 | 删除消息 |
| | `AddReaction` / `ReactionAdded` | 双向 | 添加表情反应 |
| | `RemoveReaction` / `ReactionRemoved` | 双向 | 移除表情反应 |
| | `Typing` / `StopTyping` | C→S | 输入状态 |
| **通知系统** | `PrivateMessage` | S→C | 私信通知 |
| | `Mentioned` | S→C | @提及通知 |
| | `RoomInvitation` | S→C | 房间邀请通知 |
| | `SystemNotification` | S→C | 系统通知 |
| | `FileUploadComplete` | S→C | 文件上传完成通知 |
| **用户状态** | `UpdateStatus` | C→S | 更新用户状态 |
| | `UserStatusChanged` | S→C | 用户状态变更通知 |

> **C→S**: Client to Server (客户端发送)  
> **S→C**: Server to Client (服务端推送)

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
