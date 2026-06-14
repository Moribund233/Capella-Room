# 聊天室接口文档

> **认证要求**: 所有接口均需要认证（需要携带 Access Token）

## 概述

聊天室接口提供完整的房间管理功能，包括创建、查询、加入/离开房间等操作。

### 房间列表实时更新

房间列表接口返回的数据包含 `last_message`（最后消息预览）和 `unread_count`（未读计数）字段。为了实现实时更新：

1. **初始加载**: 使用 HTTP API 获取房间列表（包含初始的最后消息和未读数）
2. **实时更新**: 通过 WebSocket 的 `RoomMessageSummary` 消息接收更新

详见 [WebSocket 房间管理文档](../websocket/room.md#房间消息摘要)

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/rooms` | 获取聊天室列表 |
| POST | `/api/v1/rooms` | 创建聊天室 |
| GET | `/api/v1/rooms/recent` | 获取最近更新的聊天室 |
| **POST** | **`/api/v1/rooms/direct`** | **创建或获取私聊房间** |
| **GET** | **`/api/v1/rooms/direct/list`** | **获取私聊房间列表** |
| GET | `/api/v1/rooms/:room_id` | 获取聊天室详情 |
| PUT | `/api/v1/rooms/:room_id` | 更新聊天室信息 |
| DELETE | `/api/v1/rooms/:room_id` | 删除聊天室 |
| POST | `/api/v1/rooms/:room_id/join` | 加入聊天室 |
| DELETE | `/api/v1/rooms/:room_id/leave` | 离开聊天室 |
| **POST** | **`/api/v1/rooms/join-by-invite`** | **通过邀请码加入房间** |
| **GET** | **`/api/v1/rooms/validate-invite`** | **验证邀请码** |
| GET | `/api/v1/rooms/:room_id/members` | 获取成员列表 |
| DELETE | `/api/v1/rooms/:room_id/members/:user_id` | 踢出成员 |
| PUT | `/api/v1/rooms/:room_id/members/:user_id/role` | 设置成员角色 |
| **GET** | **`/api/v1/rooms/:room_id/invitations`** | **获取房间邀请列表** |
| **POST** | **`/api/v1/rooms/:room_id/invitations`** | **创建房间邀请** |
| **DELETE** | **`/api/v1/rooms/:room_id/invitations/:invitation_id`** | **撤销房间邀请** |
| GET | `/api/v1/rooms/:room_id/messages` | 获取房间消息历史 |
| GET | `/api/v1/rooms/:room_id/pinned-messages` | 获取房间置顶消息列表 |

---

## 获取聊天室列表

### 请求

```http
GET /api/v1/rooms?search={keyword}&limit={limit}&offset={offset}
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `search` | string | 否 | 搜索关键词（匹配聊天室名称） |
| `limit` | number | 否 | 每页数量，默认 20，最大 100 |
| `offset` | number | 否 | 偏移量，默认 0 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "技术交流群",
      "description": "讨论各种技术话题",
      "owner": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "admin",
        "avatar_url": null
      },
      "is_private": false,
      "max_members": 100,
      "member_count": 25,
      "unread_count": 5,
      "last_message": {
        "id": "660e8400-e29b-41d4-a716-446655440001",
        "content": "有人在线吗？",
        "sender_name": "user123",
        "created_at": "2024-01-20T10:00:00Z"
      },
      "created_at": "2024-01-15T08:30:00Z",
      "updated_at": "2024-01-20T10:00:00Z"
    }
  ]
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 聊天室唯一标识 |
| `name` | string | 聊天室名称 |
| `description` | string \| null | 聊天室描述 |
| `owner` | object | 房主信息 |
| `owner.id` | string (UUID) | 房主用户 ID |
| `owner.username` | string | 房主用户名 |
| `owner.avatar_url` | string \| null | 房主头像 URL |
| `is_private` | boolean | 是否为私有聊天室 |
| `max_members` | number | 最大成员数限制 |
| `member_count` | number | 当前成员数量 |
| `unread_count` | number | 未读消息数（仅对已加入房间） |
| `last_message` | object \| null | 最后一条消息预览 |
| `last_message.id` | string (UUID) | 消息 ID |
| `last_message.content` | string | 消息内容 |
| `last_message.sender_name` | string | 发送者名称 |
| `last_message.created_at` | string (ISO 8601) | 消息发送时间 |
| `created_at` | string (ISO 8601) | 创建时间 |
| `updated_at` | string (ISO 8601) | 最后更新时间 |

---

## 创建聊天室

### 请求

```http
POST /api/v1/rooms
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "name": "技术交流群",
  "description": "讨论各种技术话题",
  "is_private": false,
  "max_members": 100
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | 聊天室名称，1-50 个字符 |
| `description` | string | 否 | 聊天室描述，最多 200 个字符 |
| `is_private` | boolean | 是 | 是否为私有聊天室 |
| `max_members` | number | 否 | 最大成员数，2-1000，默认 100 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "技术交流群",
    "description": "讨论各种技术话题",
    "owner": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "username": "admin",
      "avatar_url": null
    },
    "is_private": false,
    "max_members": 100,
    "member_count": 1,
    "created_at": "2024-01-15T08:30:00Z",
    "updated_at": "2024-01-15T08:30:00Z"
  }
}
```

**失败 - 参数验证失败 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "验证失败: name: 聊天室名称长度必须在1-50个字符之间"
}
```

### 说明

- 创建者自动成为房主（Owner）
- 创建后会自动加入该聊天室
- 会记录审计日志

---

## 获取最近更新的聊天室

### 请求

```http
GET /api/v1/rooms/recent?limit={limit}&offset={offset}
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `limit` | number | 否 | 每页数量，默认 20，最大 100 |
| `offset` | number | 否 | 偏移量，默认 0 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "活跃的技术群",
      "description": "最近很活跃的群",
      "owner": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "admin",
        "avatar_url": null
      },
      "is_private": false,
      "max_members": 100,
      "member_count": 50,
      "unread_count": 12,
      "last_message": {
        "id": "660e8400-e29b-41d4-a716-446655440002",
        "content": "今天讨论了 Rust 的异步编程",
        "sender_name": "developer",
        "created_at": "2024-01-20T18:00:00Z"
      },
      "created_at": "2024-01-15T08:30:00Z",
      "updated_at": "2024-01-20T18:00:00Z"
    }
  ]
}
```

### 说明

- 按 `updated_at` 降序排序
- 返回最近活跃的聊天室
- 包含 `last_message` 和 `unread_count` 用于房间列表展示
- 可用于首页展示热门房间

> **提示**: 配合 WebSocket 的 `RoomMessageSummary` 消息可实现房间列表的实时更新

---

## 获取聊天室详情

### 请求

```http
GET /api/v1/rooms/{room_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "技术交流群",
    "description": "讨论各种技术话题",
    "owner": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "username": "admin",
      "avatar_url": null
    },
    "is_private": false,
    "max_members": 100,
    "member_count": 25,
    "created_at": "2024-01-15T08:30:00Z",
    "updated_at": "2024-01-20T10:00:00Z"
  }
}
```

**失败 - 聊天室不存在 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源未找到",
  "message": "未找到资源"
}
```

---

## 更新聊天室信息

### 请求

```http
PUT /api/v1/rooms/{room_id}
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 请求体

```json
{
  "name": "新的聊天室名称",
  "description": "新的描述",
  "is_private": true,
  "max_members": 200
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 否 | 新名称，1-50 个字符 |
| `description` | string | 否 | 新描述，最多 200 个字符 |
| `is_private` | boolean | 否 | 是否改为私有 |
| `max_members` | number | 否 | 新的成员上限，2-1000 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "新的聊天室名称",
    "description": "新的描述",
    "owner": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "username": "admin",
      "avatar_url": null
    },
    "is_private": true,
    "max_members": 200,
    "member_count": 25,
    "created_at": "2024-01-15T08:30:00Z",
    "updated_at": "2024-01-21T12:00:00Z"
  }
}
```

**失败 - 无权限 (403 Forbidden)**

```json
{
  "success": false,
  "code": "FORBIDDEN",
  "error": "权限不足",
  "message": "权限不足: 您没有权限执行此操作"
}
```

### 说明

- 只有房主（Owner）和管理员（Admin）可以更新
- 可以只更新部分字段
- 会更新 `updated_at` 时间戳

---

## 删除聊天室

### 请求

```http
DELETE /api/v1/rooms/{room_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "聊天室已删除"
}
```

**失败 - 无权限 (403 Forbidden)**

```json
{
  "success": false,
  "code": "FORBIDDEN",
  "error": "权限不足",
  "message": "权限不足: 您没有权限执行此操作"
}
```

### 说明

- 只有房主或系统管理员可以删除
- 删除后房间所有消息和成员关系将被清除
- 会记录审计日志

---

## 加入聊天室

### 请求

```http
POST /api/v1/rooms/{room_id}/join
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "成功加入聊天室"
}
```

**失败 - 已在聊天室 (409 Conflict)**

```json
{
  "success": false,
  "code": "CONFLICT",
  "error": "资源已存在",
  "message": "您已经是该聊天室的成员"
}
```

**失败 - 聊天室已满 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "聊天室成员已满"
}
```

### 说明

- 加入后会自动成为普通成员（Member）
- 会记录审计日志
- 如果是私有房间，可能需要邀请

---

## 离开聊天室

### 请求

```http
DELETE /api/v1/rooms/{room_id}/leave
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "已离开聊天室"
}
```

**失败 - 房主不能离开 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "房主不能直接离开聊天室，请先转让所有权或删除聊天室"
}
```

### 说明

- 房主需要先转让所有权或删除房间才能离开
- 离开后不再接收该房间的消息
- 会记录审计日志

---

## 获取成员列表

### 请求

```http
GET /api/v1/rooms/{room_id}/members
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "room_id": "550e8400-e29b-41d4-a716-446655440000",
      "user_id": "550e8400-e29b-41d4-a716-446655440001",
      "role": "owner",
      "joined_at": "2024-01-15T08:30:00Z",
      "username": "admin",
      "email": "admin@example.com",
      "avatar_url": null,
      "user_status": "online"
    },
    {
      "room_id": "550e8400-e29b-41d4-a716-446655440000",
      "user_id": "550e8400-e29b-41d4-a716-446655440002",
      "role": "member",
      "joined_at": "2024-01-16T10:00:00Z",
      "username": "user1",
      "email": "user1@example.com",
      "avatar_url": "https://example.com/avatar.jpg",
      "user_status": "offline"
    }
  ]
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室 ID |
| `user_id` | string (UUID) | 用户 ID |
| `role` | string | 成员角色：`owner` / `admin` / `member` |
| `joined_at` | string (ISO 8601) | 加入时间 |
| `username` | string | 用户名 |
| `email` | string | 邮箱地址 |
| `avatar_url` | string \| null | 头像 URL |
| `user_status` | string | 用户在线状态 |

---

## 踢出成员

### 请求

```http
DELETE /api/v1/rooms/{room_id}/members/{user_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |
| `user_id` | string (UUID) | 要踢出的用户 ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "成员已被踢出"
}
```

**失败 - 无权限 (403 Forbidden)**

```json
{
  "success": false,
  "code": "FORBIDDEN",
  "error": "权限不足",
  "message": "权限不足: 只有房主和管理员可以踢出成员"
}
```

**失败 - 不能踢出房主 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "不能踢出房主"
}
```

### 说明

- 只有房主和管理员可以踢出成员
- 不能踢出房主
- 管理员不能踢出其他管理员（只有房主可以）
- 会记录审计日志

---

## 设置成员角色

### 请求

```http
PUT /api/v1/rooms/{room_id}/members/{user_id}/role
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |
| `user_id` | string (UUID) | 目标用户 ID |

### 请求体

```json
{
  "role": "admin"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `role` | string | 是 | 角色：`owner` / `admin` / `member` |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "成员角色已更新"
}
```

**失败 - 无权限 (403 Forbidden)**

```json
{
  "success": false,
  "code": "FORBIDDEN",
  "error": "权限不足",
  "message": "权限不足: 只有房主可以设置管理员"
}
```

### 说明

- 只有房主可以设置管理员
- 转让所有权需要将角色设为 `owner`
- 会记录审计日志

---

## 获取房间消息历史

### 请求

```http
GET /api/v1/rooms/{room_id}/messages?limit={limit}&before={message_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `limit` | number | 否 | 每页数量，默认 50，最大 100 |
| `before` | string (UUID) | 否 | 游标，获取此消息之前的消息 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "messages": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "room_id": "550e8400-e29b-41d4-a716-446655440001",
        "sender": {
          "id": "550e8400-e29b-41d4-a716-446655440002",
          "username": "user1",
          "avatar_url": null
        },
        "content": "大家好！",
        "message_type": "text",
        "reply_to": null,
        "is_edited": false,
        "created_at": "2024-01-15T08:30:00Z",
        "updated_at": "2024-01-15T08:30:00Z"
      }
    ],
    "total": 1,
    "has_more": false
  }
}
```

### 响应字段说明

#### 分页信息

| 字段 | 类型 | 说明 |
|------|------|------|
| `messages` | array | 消息列表 |
| `total` | number | 本次返回的消息数量 |
| `has_more` | boolean | 是否还有更多消息 |

#### 消息对象

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 消息唯一标识 |
| `room_id` | string (UUID) | 所属聊天室 ID |
| `sender` | object | 发送者信息 |
| `sender.id` | string (UUID) | 发送者用户 ID |
| `sender.username` | string | 发送者用户名 |
| `sender.avatar_url` | string \| null | 发送者头像 |
| `content` | string | 消息内容 |
| `message_type` | string | 消息类型：`text` / `image` / `file` |
| `reply_to` | string \| null | 回复的消息 ID |
| `is_edited` | boolean | 是否被编辑过 |
| `created_at` | string (ISO 8601) | 发送时间 |
| `updated_at` | string (ISO 8601) | 最后更新时间 |

### 说明

- 消息按时间倒序排列（最新的在前面）
- 使用游标分页，避免消息重复或遗漏
- 只返回未删除的消息

---

## 使用示例

### cURL 示例

```bash
# 获取聊天室列表
curl -X GET "http://localhost:3000/api/v1/rooms?limit=10" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 创建聊天室
curl -X POST http://localhost:3000/api/v1/rooms \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "技术交流群",
    "description": "讨论各种技术话题",
    "is_private": false,
    "max_members": 100
  }'

# 获取聊天室详情
curl -X GET http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 更新聊天室
curl -X PUT http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "新的名称",
    "description": "新的描述"
  }'

# 删除聊天室
curl -X DELETE http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 加入聊天室
curl -X POST http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000/join \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 离开聊天室
curl -X DELETE http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000/leave \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 获取成员列表
curl -X GET http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000/members \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 踢出成员
curl -X DELETE http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000/members/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 设置成员角色
curl -X PUT http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000/members/550e8400-e29b-41d4-a716-446655440001/role \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{"role": "admin"}'

# 获取消息历史
curl -X GET "http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440000/messages?limit=50" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### JavaScript 示例

```javascript
// 创建聊天室
async function createRoom(name, description, isPrivate = false, maxMembers = 100) {
  const response = await fetch('http://localhost:3000/api/v1/rooms', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      name,
      description,
      is_private: isPrivate,
      max_members: maxMembers
    })
  });
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 加入聊天室
async function joinRoom(roomId) {
  const response = await fetch(`http://localhost:3000/api/v1/rooms/${roomId}/join`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`
    }
  });
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 获取消息历史（支持分页）
async function getRoomMessages(roomId, limit = 50, before = null) {
  const params = new URLSearchParams();
  params.append('limit', limit);
  if (before) params.append('before', before);
  
  const response = await fetch(
    `http://localhost:3000/api/v1/rooms/${roomId}/messages?${params}`,
    {
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 加载更多消息
async function loadMoreMessages(roomId, lastMessageId) {
  const result = await getRoomMessages(roomId, 50, lastMessageId);
  
  if (result.has_more) {
    // 还有更多消息，保存最后一条消息的 ID 用于下次加载
    const lastId = result.messages[result.messages.length - 1]?.id;
    return { messages: result.messages, nextCursor: lastId };
  }
  
  return { messages: result.messages, nextCursor: null };
}
```

---

## 错误码汇总

### HTTP 状态码

| HTTP 状态码 | 错误场景 | 说明 |
|------------|---------|------|
| 200 | 请求成功 | 操作成功 |
| 400 | 请求参数错误 | 参数验证失败 |
| 401 | 认证失败 | Token 无效或过期 |
| 403 | 权限不足 | 没有操作权限 |
| 404 | 资源不存在 | 聊天室或用户不存在 |
| 409 | 资源冲突 | 已在聊天室中 |
| 500 | 服务器错误 | 内部服务器错误 |

### 业务错误码 (code)

| 错误码 | HTTP 状态码 | 说明 | 处理建议 |
|--------|------------|------|---------|
| `VALIDATION_ERROR` | 400 | 参数验证失败 | 检查请求参数是否符合要求 |
| `AUTH_ERROR` | 401 | 认证失败 | 检查 Token 是否过期 |
| `FORBIDDEN` | 403 | 权限不足 | 确认用户是否有操作权限 |
| `NOT_FOUND` | 404 | 资源不存在 | 检查聊天室 ID 是否正确 |
| `CONFLICT` | 409 | 资源冲突 | 用户已在聊天室中 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 | 稍后重试或联系管理员 |

---

## 创建或获取私聊房间

### 请求

```http
POST /api/v1/rooms/direct
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "target_user_id": "550e8400-e29b-41d4-a716-446655440002"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `target_user_id` | string (UUID) | 是 | 对方用户ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440010",
    "name": "target_user",
    "target_user": {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "username": "target_user",
      "avatar_url": null
    },
    "created_at": "2024-01-20T10:00:00Z"
  }
}
```

**失败 - 不能和自己创建私聊 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "不能和自己创建私聊"
}
```

### 说明

- 如果双方已存在私聊房间，则返回现有房间
- 私聊房间名称自动设置为目标用户的用户名（动态更新）
- 私聊房间总是私有的，最多2人

---

## 获取私聊房间列表

### 请求

```http
GET /api/v1/rooms/direct/list
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440010",
      "name": "user_b",
      "target_user": {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "username": "user_b",
        "avatar_url": null
      },
      "created_at": "2024-01-20T10:00:00Z"
    }
  ]
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 私聊房间ID |
| `name` | string | 房间名称（对方用户名，动态获取最新） |
| `target_user` | object | 对方用户信息 |
| `target_user.id` | string (UUID) | 对方用户ID |
| `target_user.username` | string | 对方用户名 |
| `target_user.avatar_url` | string \| null | 对方头像URL |
| `created_at` | string (ISO 8601) | 创建时间 |

---

## 通过邀请码加入房间

### 请求

```http
POST /api/v1/rooms/join-by-invite
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "invite_code": "ABC12345"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `invite_code` | string | 是 | 邀请码（8位字母数字） |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "成功加入聊天室"
}
```

**失败 - 邀请码无效 (403 Forbidden)**

```json
{
  "success": false,
  "code": "FORBIDDEN",
  "error": "权限不足",
  "message": "邀请码已过期或已达到使用次数限制"
}
```

**失败 - 已在房间 (200 OK)**

```json
{
  "success": true,
  "data": "成功加入聊天室"
}
```

---

## 验证邀请码

### 请求

```http
GET /api/v1/rooms/validate-invite?invite_code=ABC12345
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `invite_code` | string | 是 | 邀请码 |

### 响应

**成功 - 邀请码有效 (200 OK)**

```json
{
  "success": true,
  "data": {
    "valid": true,
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "expires_at": "2024-01-21T10:00:00Z",
    "max_uses": 5,
    "used_count": 2
  }
}
```

**成功 - 邀请码无效 (200 OK)**

```json
{
  "success": true,
  "data": {
    "valid": false
  }
}
```

---

## 获取房间邀请列表

### 请求

```http
GET /api/v1/rooms/{room_id}/invitations
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440020",
      "room_id": "550e8400-e29b-41d4-a716-446655440000",
      "inviter": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "admin",
        "avatar_url": null
      },
      "invite_code": "ABC12345",
      "expires_at": "2024-01-21T10:00:00Z",
      "max_uses": 5,
      "used_count": 2,
      "is_active": true,
      "created_at": "2024-01-20T10:00:00Z"
    }
  ]
}
```

---

## 创建房间邀请

### 请求

```http
POST /api/v1/rooms/{room_id}/invitations
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |

### 请求体

```json
{
  "expires_in_hours": 24,
  "max_uses": 5
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `expires_in_hours` | number | 否 | 有效期（小时），null表示永不过期 |
| `max_uses` | number | 否 | 最大使用次数，null表示无限制（1-1000） |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440020",
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "inviter": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "username": "admin",
      "avatar_url": null
    },
    "invite_code": "ABC12345",
    "expires_at": "2024-01-21T10:00:00Z",
    "max_uses": 5,
    "used_count": 0,
    "is_active": true,
    "created_at": "2024-01-20T10:00:00Z"
  }
}
```

**失败 - 无权限 (403 Forbidden)**

```json
{
  "success": false,
  "code": "FORBIDDEN",
  "error": "权限不足",
  "message": "权限不足: 只有房主和管理员可以创建邀请"
}
```

---

## 撤销房间邀请

### 请求

```http
DELETE /api/v1/rooms/{room_id}/invitations/{invitation_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `room_id` | string (UUID) | 聊天室唯一标识 |
| `invitation_id` | string (UUID) | 邀请ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "邀请已撤销"
}
```

### 说明

- 房主和管理员可以撤销任何邀请
- 普通成员只能撤销自己创建的邀请

---

## 权限说明

### 成员角色权限表

| 操作 | Owner | Admin | Member |
|------|-------|-------|--------|
| 查看房间信息 | ✅ | ✅ | ✅ |
| 发送消息 | ✅ | ✅ | ✅ |
| 更新房间信息 | ✅ | ✅ | ❌ |
| 删除房间 | ✅ | ❌ | ❌ |
| 踢出成员 | ✅ | ✅ | ❌ |
| 设置成员角色 | ✅ | ❌ | ❌ |
| 转让所有权 | ✅ | ❌ | ❌ |

### 特殊规则

- **Owner（房主）**: 拥有所有权限，可以转让所有权
- **Admin（管理员）**: 可以协助管理，但不能踢出其他管理员或设置角色
- **Member（成员）**: 只能发送消息和查看信息

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
