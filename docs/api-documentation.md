# Seredeli Room API 接口文档

> 版本: v1.0.0  
> 基础URL: `http://localhost:3000`  
> API版本: `v1`

## 目录

1. [通用说明](#通用说明)
2. [认证接口](#认证接口)
3. [用户接口](#用户接口)
4. [聊天室接口](#聊天室接口)
5. [消息接口](#消息接口)
6. [文件上传接口](#文件上传接口)
7. [管理员接口](#管理员接口)
8. [审计系统接口](#审计系统接口)
9. [WebSocket协议](#websocket协议)
10. [数据模型](#数据模型)
11. [错误码说明](#错误码说明)

---

## 通用说明

### 请求格式

- 所有请求和响应均使用 JSON 格式
- 请求头需包含 `Content-Type: application/json`
- 认证接口需在请求头包含 `Authorization: Bearer <token>`

### 响应格式

所有 API 响应均遵循以下统一格式：

```json
{
  "success": true,
  "data": { ... },
  "pagination": {
    "total": 100,
    "limit": 20,
    "offset": 0
  }
}
```

错误响应：

```json
{
  "success": false,
  "code": "ERROR_CODE",
  "error": "错误类型",
  "message": "详细错误信息"
}
```

### 分页参数

支持分页的接口接受以下查询参数：

| 参数 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `limit` | integer | 每页数量 | 20 |
| `offset` | integer | 偏移量 | 0 |

---

## 认证接口

### 1. 用户注册

**POST** `/api/v1/auth/register`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `username` | string | 是 | 用户名，3-20字符，只允许字母、数字、下划线 |
| `email` | string | 是 | 邮箱地址 |
| `password` | string | 是 | 密码，8-32字符，需包含大小写字母和数字 |

#### 请求示例

```json
{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePass123"
}
```

#### 响应示例

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_doe",
    "email": "john@example.com",
    "avatar_url": null,
    "status": "offline",
    "role": "user",
    "created_at": "2024-01-15T08:30:00Z"
  }
}
```

#### 错误码

- `VALIDATION_ERROR` - 请求参数验证失败
- `CONFLICT` - 用户名或邮箱已存在

---

### 2. 用户登录

**POST** `/api/v1/auth/login`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `email` | string | 是 | 邮箱地址 |
| `password` | string | 是 | 密码 |

#### 请求示例

```json
{
  "email": "john@example.com",
  "password": "SecurePass123"
}
```

#### 响应示例

```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_in": 86400,
    "token_type": "Bearer",
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "john_doe",
      "email": "john@example.com",
      "avatar_url": null,
      "status": "online",
      "role": "user",
      "created_at": "2024-01-15T08:30:00Z"
    }
  }
}
```

#### 错误码

- `AUTH_ERROR` - 邮箱或密码错误
- `VALIDATION_ERROR` - 请求参数验证失败

---

### 3. 刷新Token

**POST** `/api/v1/auth/refresh`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `refresh_token` | string | 是 | 刷新令牌 |

#### 响应示例

```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_in": 86400,
    "token_type": "Bearer"
  }
}
```

---

### 4. 用户登出

**POST** `/api/v1/auth/logout`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": {
    "message": "登出成功"
  }
}
```

---

## 用户接口

### 1. 获取当前用户信息

**GET** `/api/v1/users/me`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_doe",
    "email": "john@example.com",
    "avatar_url": "https://example.com/avatar.jpg",
    "status": "online",
    "role": "user",
    "created_at": "2024-01-15T08:30:00Z"
  }
}
```

---

### 2. 更新用户信息

**PUT** `/api/v1/users/me`

> 需要认证

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `username` | string | 否 | 新用户名 |
| `avatar_url` | string | 否 | 头像URL |

#### 请求示例

```json
{
  "username": "john_new",
  "avatar_url": "https://example.com/new_avatar.jpg"
}
```

---

### 3. 修改密码

**PUT** `/api/v1/users/me/password`

> 需要认证

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `old_password` | string | 是 | 旧密码 |
| `new_password` | string | 是 | 新密码，8-32字符 |

#### 请求示例

```json
{
  "old_password": "OldPass123",
  "new_password": "NewPass456"
}
```

---

### 4. 获取用户列表

**GET** `/api/v1/users`

> 需要认证

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `search` | string | 搜索关键词（用户名或邮箱） |
| `limit` | integer | 每页数量（1-100） |
| `offset` | integer | 偏移量 |

#### 响应示例

```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "john_doe",
        "email": "john@example.com",
        "avatar_url": null,
        "status": "online",
        "role": "user",
        "created_at": "2024-01-15T08:30:00Z"
      }
    ],
    "total": 100
  },
  "pagination": {
    "total": 100,
    "limit": 20,
    "offset": 0
  }
}
```

---

### 5. 获取指定用户信息

**GET** `/api/v1/users/:user_id`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_doe",
    "email": "john@example.com",
    "avatar_url": null,
    "status": "online",
    "role": "user",
    "created_at": "2024-01-15T08:30:00Z"
  }
}
```

---

### 6. 获取当前用户的聊天室列表

**GET** `/api/v1/users/me/rooms`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "name": "General Chat",
      "description": "General discussion room",
      "owner_id": "550e8400-e29b-41d4-a716-446655440000",
      "is_private": false,
      "max_members": 100,
      "member_count": 25,
      "created_at": "2024-01-15T08:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

---

## 聊天室接口

### 1. 创建聊天室

**POST** `/api/v1/rooms`

> 需要认证

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | 房间名称，2-50字符 |
| `description` | string | 否 | 房间描述，最多500字符 |
| `is_private` | boolean | 否 | 是否私有房间 | false |
| `max_members` | integer | 否 | 最大成员数（1-1000） | 100 |

#### 请求示例

```json
{
  "name": "开发者交流",
  "description": "技术讨论和分享",
  "is_private": false,
  "max_members": 200
}
```

#### 响应示例

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "开发者交流",
    "description": "技术讨论和分享",
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "is_private": false,
    "max_members": 200,
    "member_count": 1,
    "created_at": "2024-01-15T08:30:00Z",
    "updated_at": "2024-01-15T08:30:00Z"
  }
}
```

---

### 2. 获取聊天室列表

**GET** `/api/v1/rooms`

> 需要认证

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `search` | string | 搜索关键词 |
| `limit` | integer | 每页数量（1-100） |
| `offset` | integer | 偏移量 |

#### 响应示例

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "name": "开发者交流",
      "description": "技术讨论和分享",
      "owner_id": "550e8400-e29b-41d4-a716-446655440000",
      "is_private": false,
      "max_members": 200,
      "member_count": 25,
      "created_at": "2024-01-15T08:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

---

### 3. 获取最近活跃的聊天室

**GET** `/api/v1/rooms/recent`

> 可选认证（匿名用户只能看到公开房间）

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `limit` | integer | 每页数量 |
| `offset` | integer | 偏移量 |

---

### 4. 获取聊天室详情

**GET** `/api/v1/rooms/:room_id`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "开发者交流",
    "description": "技术讨论和分享",
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "is_private": false,
    "max_members": 200,
    "member_count": 25,
    "created_at": "2024-01-15T08:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

---

### 5. 更新聊天室信息

**PUT** `/api/v1/rooms/:room_id`

> 需要认证，仅房间所有者或管理员可操作

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 否 | 新房间名称 |
| `description` | string | 否 | 新房间描述 |
| `is_private` | boolean | 否 | 是否设为私有 |
| `max_members` | integer | 否 | 新的最大成员数 |

---

### 6. 删除聊天室

**DELETE** `/api/v1/rooms/:room_id`

> 需要认证，仅房间所有者或管理员可操作

---

### 7. 加入聊天室

**POST** `/api/v1/rooms/:room_id/join`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": {
    "message": "成功加入房间"
  }
}
```

---

### 8. 离开聊天室

**DELETE** `/api/v1/rooms/:room_id/leave`

> 需要认证

---

### 9. 获取聊天室成员列表

**GET** `/api/v1/rooms/:room_id/members`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": [
    {
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "john_doe",
      "avatar_url": null,
      "role": "owner",
      "joined_at": "2024-01-15T08:30:00Z"
    },
    {
      "user_id": "550e8400-e29b-41d4-a716-446655440002",
      "username": "jane_doe",
      "avatar_url": null,
      "role": "member",
      "joined_at": "2024-01-15T09:00:00Z"
    }
  ]
}
```

---

### 10. 踢出成员

**DELETE** `/api/v1/rooms/:room_id/members/:user_id`

> 需要认证，仅房间所有者或管理员可操作

---

### 11. 设置成员角色

**PUT** `/api/v1/rooms/:room_id/members/:user_id/role`

> 需要认证，仅房间所有者或管理员可操作

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `role` | string | 是 | 角色：`owner`、`admin`、`member` |

#### 请求示例

```json
{
  "role": "admin"
}
```

---

### 12. 获取聊天室消息历史

**GET** `/api/v1/rooms/:room_id/messages`

> 需要认证

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `limit` | integer | 每页数量（1-100，默认50） |
| `before` | UUID | 游标：获取此ID之前的消息 |

#### 响应示例

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440010",
      "room_id": "550e8400-e29b-41d4-a716-446655440001",
      "sender_id": "550e8400-e29b-41d4-a716-446655440000",
      "sender_name": "john_doe",
      "content": "大家好！",
      "message_type": "text",
      "reply_to": null,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": null,
      "edit_count": 0
    }
  ]
}
```

---

## 消息接口

### 1. 搜索消息

**GET** `/api/v1/messages/search`

> 需要认证

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `q` | string | 搜索关键词（必填） |
| `room_id` | UUID | 限定在某个房间搜索 |
| `limit` | integer | 结果数量限制（1-100，默认50） |

#### 响应示例

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440010",
      "room_id": "550e8400-e29b-41d4-a716-446655440001",
      "sender_id": "550e8400-e29b-41d4-a716-446655440000",
      "sender_name": "john_doe",
      "content": "搜索到的消息内容",
      "message_type": "text",
      "created_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

---

### 2. 编辑消息

**PUT** `/api/v1/messages/:message_id`

> 需要认证，仅消息发送者可编辑

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `content` | string | 是 | 新消息内容，1-2000字符 |

#### 请求示例

```json
{
  "content": "编辑后的消息内容"
}
```

---

### 3. 删除消息

**DELETE** `/api/v1/messages/:message_id`

> 需要认证，仅消息发送者可删除

---

### 4. 获取消息编辑历史

**GET** `/api/v1/messages/:message_id/history`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440020",
      "message_id": "550e8400-e29b-41d4-a716-446655440010",
      "old_content": "原始内容",
      "new_content": "编辑后的内容",
      "edited_by": "550e8400-e29b-41d4-a716-446655440000",
      "edited_at": "2024-01-15T11:00:00Z"
    }
  ]
}
```

---

## 文件上传接口

### 1. 通用文件上传

**POST** `/api/v1/upload`

> 需要认证

#### 请求格式

Content-Type: `multipart/form-data`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file` | File | 是 | 文件数据 |
| `usage_type` | string | 否 | 用途：`general`、`avatar`、`message`、`room_cover` |
| `room_id` | UUID | 否 | 关联的房间ID |

#### 响应示例

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440030",
    "original_name": "document.pdf",
    "file_path": "uploads/documents/2024/01/uuid.pdf",
    "file_size": 1024567,
    "mime_type": "application/pdf",
    "category": "document",
    "usage_type": "general",
    "url": "/uploads/documents/2024/01/uuid.pdf",
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

---

### 2. 上传图片

**POST** `/api/v1/upload/image`

> 需要认证

#### 请求格式

Content-Type: `multipart/form-data`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file` | File | 是 | 图片文件（jpg、png、gif、webp） |
| `usage_type` | string | 否 | 用途 |
| `room_id` | UUID | 否 | 关联的房间ID |

---

### 3. 上传头像

**POST** `/api/v1/upload/avatar`

> 需要认证

上传头像并自动更新用户头像。

#### 请求格式

Content-Type: `multipart/form-data`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file` | File | 是 | 头像图片 |

---

### 4. 获取文件列表

**GET** `/api/v1/files`

> 需要认证

#### 响应示例

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440030",
      "original_name": "document.pdf",
      "file_size": 1024567,
      "mime_type": "application/pdf",
      "category": "document",
      "usage_type": "general",
      "url": "/uploads/documents/2024/01/uuid.pdf",
      "created_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

---

### 5. 获取文件信息

**GET** `/api/v1/files/:file_id`

> 需要认证

---

### 6. 删除文件

**DELETE** `/api/v1/files/:file_id`

> 需要认证，仅文件上传者可删除

---

## 管理员接口

> 所有管理员接口需要管理员权限（Admin 或 SuperAdmin）

### 用户管理

#### 1. 获取用户列表

**GET** `/api/v1/admin/users`

##### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `page` | integer | 页码（默认1） |
| `page_size` | integer | 每页数量 |
| `search` | string | 搜索关键词 |

---

#### 2. 获取用户详情

**GET** `/api/v1/admin/users/:user_id`

---

#### 3. 修改用户角色

**PUT** `/api/v1/admin/users/:user_id/role`

##### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `role` | string | 是 | `user`、`admin`、`super_admin` |

---

#### 4. 设置用户状态

**PUT** `/api/v1/admin/users/:user_id/status`

##### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `disabled` | boolean | 是 | 是否禁用用户 |

---

#### 5. 删除用户

**DELETE** `/api/v1/admin/users/:user_id`

---

### 房间管理

#### 6. 获取房间列表

**GET** `/api/v1/admin/rooms`

##### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `page` | integer | 页码 |
| `page_size` | integer | 每页数量 |
| `search` | string | 搜索关键词 |

---

#### 7. 获取房间详情

**GET** `/api/v1/admin/rooms/:room_id`

---

#### 8. 强制删除房间

**DELETE** `/api/v1/admin/rooms/:room_id`

---

#### 9. 获取房间消息

**GET** `/api/v1/admin/rooms/:room_id/messages`

---

### 消息审核

#### 10. 获取消息列表

**GET** `/api/v1/admin/messages`

##### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `page` | integer | 页码 |
| `page_size` | integer | 每页数量 |
| `search` | string | 搜索关键词 |

---

#### 11. 删除消息

**DELETE** `/api/v1/admin/messages/:message_id`

---

### 系统统计

#### 12. 获取系统统计

**GET** `/api/v1/admin/stats`

##### 响应示例

```json
{
  "success": true,
  "data": {
    "total_users": 1000,
    "total_rooms": 50,
    "total_messages": 50000,
    "online_users": 150,
    "today_new_users": 10,
    "today_messages": 1000
  }
}
```

---

#### 13. 获取活动统计

**GET** `/api/v1/admin/stats/activity`

---

### 系统配置

#### 14. 获取配置列表

**GET** `/api/v1/admin/configs`

---

#### 15. 获取配置详情

**GET** `/api/v1/admin/configs/:key`

---

#### 16. 更新配置

**PUT** `/api/v1/admin/configs/:key`

> 仅 SuperAdmin 可操作

##### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `value` | string | 是 | 配置值 |

---

#### 17. 重置配置

**POST** `/api/v1/admin/configs`

> 仅 SuperAdmin 可操作

---

### 日志查看

#### 18. 查看日志

**GET** `/api/v1/admin/logs`

##### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `level` | string | 日志级别：error、warn、info、debug |
| `start_time` | datetime | 开始时间 |
| `end_time` | datetime | 结束时间 |

---

#### 19. 下载日志

**GET** `/api/v1/admin/logs/download`

---

## 审计系统接口

> 仅 SuperAdmin 可操作

### 1. 查询审计日志

**GET** `/api/v1/admin/audit/logs`

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `event_type` | string | 事件类型 |
| `severity` | string | 严重级别：info、warning、error、critical |
| `actor_id` | UUID | 操作者ID |
| `target_id` | UUID | 目标ID |
| `status` | string | 状态：success、failure |
| `start_time` | datetime | 开始时间 |
| `end_time` | datetime | 结束时间 |
| `limit` | integer | 每页数量 |
| `offset` | integer | 偏移量 |

#### 响应示例

```json
{
  "success": true,
  "data": {
    "logs": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440100",
        "event_type": "user_login",
        "severity": "info",
        "actor_id": "550e8400-e29b-41d4-a716-446655440000",
        "actor_role": "user",
        "action": "post",
        "description": "POST /api/v1/auth/login - 200",
        "metadata": {
          "ip": "192.168.1.1",
          "user_agent": "Mozilla/5.0..."
        },
        "status": "success",
        "created_at": "2024-01-15T10:30:00Z"
      }
    ],
    "total": 1000
  }
}
```

---

### 2. 获取审计日志详情

**GET** `/api/v1/admin/audit/logs/:id`

---

### 3. 获取审计统计

**GET** `/api/v1/admin/audit/stats`

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `start_time` | datetime | 开始时间 |
| `end_time` | datetime | 结束时间 |

#### 响应示例

```json
{
  "success": true,
  "data": {
    "total_logs": 10000,
    "by_severity": {
      "info": 8000,
      "warning": 1500,
      "error": 400,
      "critical": 100
    },
    "by_event_type": {
      "user_login": 3000,
      "message_send": 5000
    },
    "daily_counts": [
      {
        "date": "2024-01-15",
        "count": 500
      }
    ]
  }
}
```

---

### 4. 导出审计日志

**GET** `/api/v1/admin/audit/export`

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `format` | string | 格式：`json` 或 `csv` |
| `start_time` | datetime | 开始时间 |
| `end_time` | datetime | 结束时间 |

---

### 5. 获取告警列表

**GET** `/api/v1/admin/audit/alerts`

#### 查询参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `status` | string | 状态：new、acknowledged、resolved、ignored |
| `severity` | string | 严重级别 |

---

### 6. 处理告警

**PUT** `/api/v1/admin/audit/alerts/:id/status`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `status` | string | 是 | `acknowledged`、`resolved`、`ignored` |

---

### 7. 获取告警规则

**GET** `/api/v1/admin/audit/rules`

---

### 8. 更新告警规则

**PUT** `/api/v1/admin/audit/rules/:id`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `enabled` | boolean | 否 | 是否启用 |
| `severity` | string | 否 | 严重级别 |
| `cooldown_minutes` | integer | 否 | 冷却时间（分钟） |

---

### 9. 清理审计日志

**POST** `/api/v1/admin/audit/cleanup`

#### 请求参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `before` | datetime | 是 | 清理此日期之前的日志 |

---

## WebSocket协议

### 连接地址

```
ws://localhost:3000/ws
```

### 认证

连接后首先发送认证消息：

```json
{
  "type": "Auth",
  "payload": {
    "token": "eyJhbGciOiJIUzI1NiIs..."
  }
}
```

认证结果：

```json
{
  "type": "AuthResult",
  "payload": {
    "success": true,
    "message": "认证成功"
  }
}
```

### 心跳机制

客户端每30秒发送一次Ping：

```json
{
  "type": "Ping"
}
```

服务端响应Pong：

```json
{
  "type": "Pong"
}
```

### 房间管理

#### 加入房间

```json
{
  "type": "JoinRoom",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001"
  }
}
```

加入成功：

```json
{
  "type": "RoomJoined",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_doe"
  }
}
```

其他用户收到通知：

```json
{
  "type": "UserJoined",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_doe"
  }
}
```

#### 离开房间

```json
{
  "type": "LeaveRoom",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001"
  }
}
```

#### 获取在线用户

```json
{
  "type": "GetOnlineUsers",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001"
  }
}
```

响应：

```json
{
  "type": "OnlineUsers",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "users": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "john_doe",
        "status": "online"
      }
    ]
  }
}
```

### 消息通信

#### 发送消息

```json
{
  "type": "ChatMessage",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "content": "大家好！",
    "reply_to": null
  }
}
```

回复消息：

```json
{
  "type": "ChatMessage",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "content": "回复你的消息",
    "reply_to": "550e8400-e29b-41d4-a716-446655440010"
  }
}
```

#### 接收新消息

房间内所有成员收到：

```json
{
  "type": "NewMessage",
  "payload": {
    "message_id": "550e8400-e29b-41d4-a716-446655440011",
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "sender_id": "550e8400-e29b-41d4-a716-446655440000",
    "sender_name": "john_doe",
    "content": "大家好！",
    "reply_to": null,
    "reply_to_message": null,
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

#### 编辑消息

```json
{
  "type": "EditMessage",
  "payload": {
    "message_id": "550e8400-e29b-41d4-a716-446655440011",
    "new_content": "编辑后的内容"
  }
}
```

编辑通知：

```json
{
  "type": "MessageEdited",
  "payload": {
    "message_id": "550e8400-e29b-41d4-a716-446655440011",
    "new_content": "编辑后的内容",
    "edited_at": "2024-01-15T10:35:00Z"
  }
}
```

#### 删除消息

```json
{
  "type": "DeleteMessage",
  "payload": {
    "message_id": "550e8400-e29b-41d4-a716-446655440011"
  }
}
```

删除通知：

```json
{
  "type": "MessageDeleted",
  "payload": {
    "message_id": "550e8400-e29b-41d4-a716-446655440011"
  }
}
```

#### 正在输入

```json
{
  "type": "Typing",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001"
  }
}
```

停止输入：

```json
{
  "type": "StopTyping",
  "payload": {
    "room_id": "550e8400-e29b-41d4-a716-446655440001"
  }
}
```

#### 消息已读

```json
{
  "type": "MessageRead",
  "payload": {
    "message_id": "550e8400-e29b-41d4-a716-446655440011"
  }
}
```

已读回执：

```json
{
  "type": "MessageReadReceipt",
  "payload": {
    "message_id": "550e8400-e29b-41d4-a716-446655440011",
    "user_id": "550e8400-e29b-41d4-a716-446655440002"
  }
}
```

### 用户状态

#### 更新状态

```json
{
  "type": "UpdateStatus",
  "payload": {
    "status": "away"
  }
}
```

状态值：`online`、`offline`、`away`

#### 状态变更通知

```json
{
  "type": "UserStatusChanged",
  "payload": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_doe",
    "status": "away"
  }
}
```

### 系统消息

```json
{
  "type": "SystemMessage",
  "payload": {
    "content": "系统维护通知：将于今晚12点进行维护"
  }
}
```

### 错误消息

```json
{
  "type": "Error",
  "payload": {
    "code": "AUTH_ERROR",
    "message": "认证失败"
  }
}
```

---

## 数据模型

### User（用户）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | UUID | 用户ID |
| `username` | string | 用户名 |
| `email` | string | 邮箱 |
| `avatar_url` | string | 头像URL |
| `status` | string | 状态：online、offline、away |
| `role` | string | 角色：user、admin、super_admin |
| `created_at` | datetime | 创建时间 |

### Room（聊天室）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | UUID | 房间ID |
| `name` | string | 房间名称 |
| `description` | string | 房间描述 |
| `owner_id` | UUID | 所有者ID |
| `is_private` | boolean | 是否私有 |
| `max_members` | integer | 最大成员数 |
| `member_count` | integer | 当前成员数 |
| `created_at` | datetime | 创建时间 |
| `updated_at` | datetime | 更新时间 |

### Message（消息）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | UUID | 消息ID |
| `room_id` | UUID | 房间ID |
| `sender_id` | UUID | 发送者ID |
| `sender_name` | string | 发送者用户名 |
| `content` | string | 消息内容 |
| `message_type` | string | 类型：text、image、file |
| `reply_to` | UUID | 回复的消息ID |
| `created_at` | datetime | 创建时间 |
| `updated_at` | datetime | 更新时间 |
| `edit_count` | integer | 编辑次数 |

### File（文件）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | UUID | 文件ID |
| `original_name` | string | 原始文件名 |
| `file_path` | string | 文件路径 |
| `file_size` | integer | 文件大小（字节） |
| `mime_type` | string | MIME类型 |
| `category` | string | 分类：image、document、video、audio、other |
| `usage_type` | string | 用途 |
| `url` | string | 访问URL |
| `created_at` | datetime | 创建时间 |

---

## 错误码说明

| 错误码 | HTTP状态 | 说明 |
|--------|----------|------|
| `VALIDATION_ERROR` | 400 | 请求参数验证失败 |
| `AUTH_ERROR` | 401 | 认证失败（未登录或Token无效） |
| `FORBIDDEN` | 403 | 权限不足 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `CONFLICT` | 409 | 资源冲突（如用户名已存在） |
| `RATE_LIMIT` | 429 | 请求过于频繁 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |

---

## 前端开发建议

### 1. API客户端封装

建议使用 Axios 或 Fetch API 封装请求：

```typescript
// api/client.ts
const apiClient = axios.create({
  baseURL: 'http://localhost:3000/api/v1',
  headers: {
    'Content-Type': 'application/json'
  }
});

// 请求拦截器添加Token
apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截器处理Token刷新
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      // 尝试刷新Token
      const refreshToken = localStorage.getItem('refresh_token');
      if (refreshToken) {
        try {
          const { data } = await axios.post('/auth/refresh', { refresh_token: refreshToken });
          localStorage.setItem('access_token', data.data.access_token);
          // 重试原请求
          return apiClient(error.config);
        } catch {
          // 刷新失败，跳转到登录页
          window.location.href = '/login';
        }
      }
    }
    return Promise.reject(error);
  }
);
```

### 2. WebSocket管理

```typescript
// websocket/manager.ts
class WebSocketManager {
  private ws: WebSocket | null = null;
  private reconnectInterval = 5000;
  private heartbeatInterval: number | null = null;

  connect(token: string) {
    this.ws = new WebSocket('ws://localhost:3000/ws');
    
    this.ws.onopen = () => {
      // 发送认证
      this.send({ type: 'Auth', payload: { token } });
      // 启动心跳
      this.startHeartbeat();
    };

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };

    this.ws.onclose = () => {
      this.stopHeartbeat();
      // 自动重连
      setTimeout(() => this.connect(token), this.reconnectInterval);
    };
  }

  private startHeartbeat() {
    this.heartbeatInterval = window.setInterval(() => {
      this.send({ type: 'Ping' });
    }, 30000);
  }

  private stopHeartbeat() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
    }
  }

  send(message: any) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  private handleMessage(message: any) {
    // 分发消息到各组件
    EventBus.emit('websocket:message', message);
  }
}
```

### 3. 状态管理

建议使用 Pinia 管理全局状态：

```typescript
// stores/auth.ts
export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null as User | null,
    token: localStorage.getItem('access_token'),
    isAuthenticated: false
  }),
  
  actions: {
    async login(credentials: LoginCredentials) {
      const { data } = await apiClient.post('/auth/login', credentials);
      this.user = data.data.user;
      this.token = data.data.access_token;
      this.isAuthenticated = true;
      localStorage.setItem('access_token', data.data.access_token);
      localStorage.setItem('refresh_token', data.data.refresh_token);
    },
    
    logout() {
      this.user = null;
      this.token = null;
      this.isAuthenticated = false;
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
    }
  }
});
```

---

*文档版本: 1.0.0*  
*最后更新: 2024-01*
