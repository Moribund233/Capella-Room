# 管理员接口文档

> **认证要求**: 所有接口均需要管理员权限（Admin 或 SuperAdmin）
> 
> **基础路径**: `/api/v1/admin`

## 权限说明

管理员系统采用三级角色体系：

| 角色 | 标识 | 权限范围 |
|------|------|----------|
| 普通用户 | `user` | 基础功能访问 |
| 管理员 | `admin` | 用户管理、房间管理、消息审核、查看统计 |
| 超级管理员 | `super_admin` | 所有权限，包括配置管理、重置密码、修改角色 |

### 权限继承关系

- **SuperAdmin**: 拥有所有权限，可以管理其他管理员
- **Admin**: 可以管理普通用户，但不能管理其他管理员或修改系统配置
- **User**: 普通用户，无法访问管理员接口

### 特殊限制

- SuperAdmin 不能被禁用、删除或降级
- Admin 不能修改其他 Admin 或 SuperAdmin 的信息
- 重置用户密码仅 SuperAdmin 可操作

---

## 接口列表

### 用户管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/users` | 获取用户列表 | Admin |
| GET | `/users/:user_id` | 获取用户详情 | Admin |
| PUT | `/users/:user_id/role` | 修改用户角色 | SuperAdmin |
| PUT | `/users/:user_id/status` | 禁用/启用用户 | Admin |
| DELETE | `/users/:user_id` | 删除用户 | Admin |
| PUT | `/users/:user_id/password` | 重置用户密码 | SuperAdmin |

### 房间管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/rooms` | 获取房间列表 | Admin |
| GET | `/rooms/:room_id` | 获取房间详情 | Admin |
| DELETE | `/rooms/:room_id` | 强制删除房间 | Admin |
| GET | `/rooms/:room_id/messages` | 获取房间消息记录 | Admin |

### 消息审核

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/messages` | 获取所有消息 | Admin |
| DELETE | `/messages/:message_id` | 删除违规消息 | Admin |

### 系统统计

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/stats` | 系统统计概览 | Admin |
| GET | `/stats/activity` | 活跃度统计 | Admin |
| GET | `/stats/performance` | 性能指标 | Admin |

### 配置管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/configs` | 获取所有配置项 | Admin |
| GET | `/configs/:key` | 获取指定配置 | Admin |
| PUT | `/configs/:key` | 修改配置项 | SuperAdmin |
| POST | `/configs/reset` | 重置配置到默认值 | SuperAdmin |

### 审计系统

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/audit/logs` | 查询审计日志 | Admin |
| GET | `/audit/logs/:id` | 获取日志详情 | Admin |
| GET | `/audit/stats` | 审计统计信息 | Admin |
| GET | `/audit/export` | 导出审计日志 | Admin |
| GET | `/audit/alerts` | 获取安全告警列表 | Admin |
| PUT | `/audit/alerts/:id/status` | 更新告警状态 | Admin |
| GET | `/audit/rules` | 获取告警规则 | Admin |
| PUT | `/audit/rules/:id` | 修改告警规则 | SuperAdmin |
| POST | `/audit/cleanup` | 清理过期日志 | SuperAdmin |

### IP 安全管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/security/ip-list` | 查询 IP 列表 | Admin |
| POST | `/security/ip-list` | 添加 IP 到列表 | Admin |
| POST | `/security/ip-list/batch` | 批量添加 IP | Admin |
| PUT | `/security/ip-list/:id` | 更新 IP 条目 | Admin |
| DELETE | `/security/ip-list/:id` | 移除 IP | Admin |
| POST | `/security/ip-check` | 检查 IP 状态 | Admin |
| GET | `/security/stats` | 获取安全统计 | Admin |
| POST | `/security/refresh-cache` | 刷新缓存 | Admin |
| POST | `/security/cleanup-expired` | 清理过期条目 | Admin |
| GET | `/security/whitelist-mode` | 获取白名单模式状态 | Admin |
| POST | `/security/whitelist-mode` | 设置白名单模式 | SuperAdmin |

### Redis 与分布式管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/redis/status` | 获取 Redis 连接状态 | Admin |
| POST | `/redis/refresh` | 刷新 Redis 连接 | SuperAdmin |
| GET | `/redis/stats` | 获取 Redis 统计信息 | Admin |
| POST | `/config/sync` | 触发配置同步到所有节点 | SuperAdmin |
| GET | `/config/sync/status` | 获取配置同步状态 | Admin |

### 系统监控

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/monitor` | 获取系统监控数据（内存、磁盘、数据库连接池） | Admin |

---

## 用户管理接口

### 获取用户列表

获取系统中所有用户的列表，支持分页和搜索。

#### 请求

```http
GET /api/v1/admin/users?page=1&page_size=20&search=keyword
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | integer | 否 | 页码，默认 1 |
| page_size | integer | 否 | 每页数量，默认 20 |
| search | string | 否 | 搜索关键词（用户名或邮箱） |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "admin",
        "email": "admin@example.com",
        "avatar_url": null,
        "status": "online",
    "is_active": true,
    "role": "super_admin",
    "created_at": "2024-01-15T08:30:00+00:00"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "username": "testuser",
    "email": "user@example.com",
    "avatar_url": "https://example.com/avatar.jpg",
    "status": "offline",
    "is_active": true,
    "role": "user",
    "created_at": "2024-01-15T09:00:00+00:00"
  }
    ],
    "total": 100,
    "page": 1,
    "page_size": 20
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| users | array | 用户列表 |
| users[].id | string (UUID) | 用户唯一标识 |
| users[].username | string | 用户名 |
| users[].email | string | 邮箱地址 |
| users[].avatar_url | string/null | 头像URL |
| users[].status | string | 在线状态：`online`/`offline`/`away` |
| users[].is_active | boolean | 账号状态：`true` 启用，`false` 禁用 |
| users[].role | string | 用户角色：`user`/`admin`/`super_admin` |
| users[].created_at | string | 创建时间（ISO 8601格式） |
| total | integer | 总用户数 |
| page | integer | 当前页码 |
| page_size | integer | 每页数量 |

---

### 获取用户详情

获取指定用户的详细信息。

#### 请求

```http
GET /api/v1/admin/users/:user_id
```

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| user_id | string (UUID) | 用户ID |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "user@example.com",
    "avatar_url": null,
    "status": "online",
    "is_active": true,
    "role": "user",
    "created_at": "2024-01-15T08:30:00+00:00"
  }
}
```

**错误响应**

- `404 Not Found`: 用户不存在

---

### 修改用户角色

> **权限**: SuperAdmin 专属

修改指定用户的角色。

#### 请求

```http
PUT /api/v1/admin/users/:user_id/role
Content-Type: application/json

{
  "role": "admin"
}
```

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| user_id | string (UUID) | 用户ID |

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| role | string | 是 | 新角色：`user`/`admin`/`super_admin` |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "user@example.com",
    "avatar_url": null,
    "status": "online",
    "is_active": true,
    "role": "admin",
    "created_at": "2024-01-15T08:30:00+00:00"
  }
}
```

**错误响应**

- `403 Forbidden`: 权限不足（需要 SuperAdmin）
- `403 Forbidden`: 不能修改 SuperAdmin 的角色
- `400 Bad Request`: 无效的角色值
- `404 Not Found`: 用户不存在

---

### 禁用/启用用户

禁用或启用指定用户账号。

#### 请求

```http
PUT /api/v1/admin/users/:user_id/status
Content-Type: application/json

{
  "disabled": true
}
```

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| user_id | string (UUID) | 用户ID |

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| disabled | boolean | 是 | `true` 禁用用户，`false` 启用用户 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "user@example.com",
    "avatar_url": null,
    "status": "offline",
    "is_active": false,
    "role": "user",
    "created_at": "2024-01-15T08:30:00+00:00"
  }
}
```

**说明**

- 禁用用户时，`is_active` 设置为 `false`，`status` 保持不变
- 启用用户时，`is_active` 设置为 `true`，`status` 保持不变
- 被禁用的用户无法登录系统

**错误响应**

- `403 Forbidden`: 不能禁用 SuperAdmin
- `403 Forbidden`: Admin 不能禁用其他 Admin
- `404 Not Found`: 用户不存在

---

### 删除用户

永久删除指定用户账号。

#### 请求

```http
DELETE /api/v1/admin/users/:user_id
```

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| user_id | string (UUID) | 用户ID |

#### 响应

**成功 (204 No Content)**

无响应体

**错误响应**

- `403 Forbidden`: 不能删除 SuperAdmin
- `403 Forbidden`: Admin 不能删除其他 Admin
- `404 Not Found`: 用户不存在

---

### 重置用户密码

> **权限**: SuperAdmin 专属

管理员重置指定用户的密码。

#### 请求

```http
PUT /api/v1/admin/users/:user_id/password
Content-Type: application/json

{
  "new_password": "NewPassword123"
}
```

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| user_id | string (UUID) | 用户ID |

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| new_password | string | 是 | 新密码（至少8位） |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "user@example.com",
    "avatar_url": null,
    "status": "online",
    "role": "user",
    "created_at": "2024-01-15T08:30:00+00:00"
  }
}
```

**错误响应**

- `403 Forbidden`: 权限不足（需要 SuperAdmin）
- `403 Forbidden`: 不能重置 SuperAdmin 的密码
- `400 Bad Request`: 密码长度不足
- `404 Not Found`: 用户不存在

---

## 房间管理接口

### 获取房间列表

获取系统中所有房间的列表，支持分页和搜索。

#### 请求

```http
GET /api/v1/admin/rooms?page=1&page_size=20&search=keyword
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | integer | 否 | 页码，默认 1 |
| page_size | integer | 否 | 每页数量，默认 20 |
| search | string | 否 | 搜索关键词（房间名称） |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "rooms": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "General Chat",
        "description": "Public chat room",
        "owner": {
          "id": "550e8400-e29b-41d4-a716-446655440001",
          "username": "admin",
          "avatar_url": null
        },
        "is_private": false,
        "max_members": 50,
        "member_count": 42,
        "created_at": "2024-01-15T08:30:00+00:00",
        "updated_at": "2024-01-15T10:00:00+00:00"
      }
    ],
    "total": 50,
    "page": 1,
    "page_size": 20
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| id | string (UUID) | 房间唯一标识 |
| name | string | 房间名称 |
| description | string | 房间描述 |
| owner | object | 房间所有者信息 |
| owner.id | string (UUID) | 所有者ID |
| owner.username | string | 所有者用户名 |
| owner.avatar_url | string/null | 所有者头像URL |
| is_private | boolean | 是否为私有房间 |
| max_members | integer | 最大成员数 |
| member_count | integer | 当前成员数 |
| created_at | string | 创建时间 |
| updated_at | string | 最后更新时间 |

---

### 获取房间详情

获取指定房间的详细信息。

#### 请求

```http
GET /api/v1/admin/rooms/:room_id
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "General Chat",
    "description": "Public chat room",
    "owner": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "username": "admin",
      "avatar_url": null
    },
    "is_private": false,
    "max_members": 50,
    "member_count": 42,
    "created_at": "2024-01-15T08:30:00+00:00",
    "updated_at": "2024-01-15T10:00:00+00:00"
  }
}
```

---

### 强制删除房间

强制删除指定房间（包括所有消息和成员关系）。

#### 请求

```http
DELETE /api/v1/admin/rooms/:room_id
```

#### 响应

**成功 (204 No Content)**

无响应体

**错误响应**

- `404 Not Found`: 房间不存在

---

### 获取房间消息记录

获取指定房间的消息历史记录。

#### 请求

```http
GET /api/v1/admin/rooms/:room_id/messages?limit=50&before=message_id
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| limit | integer | 否 | 返回消息数量，默认 50，最大 100 |
| before | string (UUID) | 否 | 游标，获取此消息之前的消息 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "room_id": "550e8400-e29b-41d4-a716-446655440001",
      "sender_id": "550e8400-e29b-41d4-a716-446655440002",
      "content": "Hello, world!",
      "message_type": "text",
      "created_at": "2024-01-15T08:30:00+00:00",
      "updated_at": "2024-01-15T08:30:00+00:00",
      "edit_count": 0,
      "edited_at": null
    }
  ]
}
```

---

### 踢出房间成员

管理员强制将指定用户踢出房间。

#### 请求

```http
DELETE /api/v1/admin/rooms/:room_id/members/:user_id
```

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| room_id | string (UUID) | 房间ID |
| user_id | string (UUID) | 要踢出的用户ID |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "message": "成员已被踢出"
}
```

**错误响应**

- `401 Unauthorized`: 未登录或Token无效
- `403 Forbidden`: 非管理员身份
- `404 Not Found`: 房间不存在或成员不存在

---

### 设置房间成员角色

管理员设置指定用户在房间中的角色（包括转让房主权限）。

#### 请求

```http
PUT /api/v1/admin/rooms/:room_id/members/:user_id/role
Content-Type: application/json

{
  "role": "admin"
}
```

#### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| room_id | string (UUID) | 房间ID |
| user_id | string (UUID) | 目标用户ID |

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| role | string | 是 | 角色类型：`owner`、`admin`、`member` |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "message": "成员角色已更新"
}
```

**错误响应**

- `400 Bad Request`: 无效的角色类型
- `401 Unauthorized`: 未登录或Token无效
- `403 Forbidden`: 非管理员身份
- `404 Not Found`: 房间不存在或成员不存在

---

## 消息审核接口

### 获取所有消息

获取系统中所有消息，支持关键词搜索和房间过滤。

#### 请求

```http
GET /api/v1/admin/messages?page=1&page_size=50&search=keyword&room_id=uuid
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | integer | 否 | 页码，默认 1 |
| page_size | integer | 否 | 每页数量，默认 50 |
| search | string | 否 | 关键词搜索（消息内容） |
| room_id | string (UUID) | 否 | 过滤特定房间的消息 |

#### 响应

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
          "username": "testuser",
          "avatar_url": "https://example.com/avatar.jpg"
        },
        "content": "Message content",
        "message_type": "text",
        "reply_to": null,
        "is_deleted": false,
        "created_at": "2024-01-15T08:30:00+00:00",
        "edit_count": 0,
        "edited_at": null
      }
    ],
    "total": 1000,
    "page": 1,
    "page_size": 50
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| messages | array | 消息列表 |
| messages[].id | string (UUID) | 消息唯一标识 |
| messages[].room_id | string (UUID) | 所属房间 ID |
| messages[].sender | object | 发送者信息 |
| messages[].sender.id | string (UUID) | 发送者 ID |
| messages[].sender.username | string | 发送者用户名 |
| messages[].sender.avatar_url | string/null | 发送者头像 URL |
| messages[].content | string | 消息内容 |
| messages[].message_type | string | 消息类型：`text`/`image`/`file`/`system` |
| messages[].reply_to | string (UUID)/null | 回复的消息 ID |
| messages[].is_deleted | boolean | 是否已删除 |
| messages[].created_at | string | 创建时间（ISO 8601格式） |
| messages[].edit_count | integer | 编辑次数 |
| messages[].edited_at | string/null | 最后编辑时间 |
| total | integer | 总消息数 |
| page | integer | 当前页码 |
| page_size | integer | 每页数量 |

---

### 删除违规消息

删除指定的违规消息。

#### 请求

```http
DELETE /api/v1/admin/messages/:message_id
```

#### 响应

**成功 (204 No Content)**

无响应体

**错误响应**

- `404 Not Found`: 消息不存在

---

## 系统统计接口

### 系统统计概览

获取系统整体统计数据。

#### 请求

```http
GET /api/v1/admin/stats
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "total_users": 150,
    "total_rooms": 25,
    "total_messages": 5000,
    "online_users": 42,
    "active_connections": 50
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| total_users | integer | 总注册用户数 |
| total_rooms | integer | 总房间数 |
| total_messages | integer | 总消息数 |
| online_users | integer | 当前在线用户数 |
| active_connections | integer | 当前活跃 WebSocket 连接数 |

---

### 活跃度统计

获取用户活跃度统计数据。

#### 请求

```http
GET /api/v1/admin/stats/activity
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "daily_active_users": 80,
    "weekly_active_users": 120,
    "monthly_active_users": 140,
    "daily_messages": 500,
    "weekly_messages": 3500,
    "monthly_messages": 15000
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| daily_active_users | integer | 日活跃用户（DAU） |
| weekly_active_users | integer | 周活跃用户（WAU） |
| monthly_active_users | integer | 月活跃用户（MAU） |
| daily_messages | integer | 今日消息数 |
| weekly_messages | integer | 本周消息数 |
| monthly_messages | integer | 本月消息数 |

---

### 性能指标

获取系统性能指标。

#### 请求

```http
GET /api/v1/admin/stats/performance
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "total_messages": 5000,
    "total_connections": 1200,
    "current_online_users": 42,
    "active_rooms": 15,
    "timestamp": "2024-01-15T10:00:00+00:00"
  }
}
```

---

## 配置管理接口

### 获取所有配置项

获取系统中的所有配置项。

#### 请求

```http
GET /api/v1/admin/configs?category=general
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| category | string | 否 | 按分类过滤配置 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "key": "system.name",
      "value": "Capella Room",
      "value_type": "string",
      "description": "系统名称",
      "category": "system",
      "is_editable": true,
      "is_hot_reloadable": true
    },
    {
      "key": "websocket.heartbeat_interval",
      "value": "30",
      "value_type": "integer",
      "description": "心跳间隔（秒）",
      "category": "websocket",
      "is_editable": true,
      "is_hot_reloadable": true
    }
  ]
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| key | string | 配置键 |
| value | string | 配置值 |
| value_type | string | 值类型：`string`/`integer`/`boolean`/`float` |
| description | string | 配置描述 |
| category | string | 配置分类 |
| is_editable | boolean | 是否可编辑 |
| is_hot_reloadable | boolean | 是否支持热重载 |

---

### 获取指定配置

获取单个配置项的详细信息。

#### 请求

```http
GET /api/v1/admin/configs/system.name
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "key": "system.name",
    "value": "Capella Room",
    "value_type": "string",
    "description": "系统名称",
    "category": "system",
    "is_editable": true,
    "is_hot_reloadable": true
  }
}
```

**错误响应**

- `404 Not Found`: 配置项不存在

---

### 修改配置项

> **权限**: SuperAdmin 专属

修改指定配置项的值。

#### 请求

```http
PUT /api/v1/admin/configs/system.name
Content-Type: application/json

{
  "value": "New System Name"
}
```

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| value | string | 是 | 新的配置值 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "key": "system.name",
    "value": "New System Name",
    "value_type": "string",
    "description": "系统名称",
    "category": "system",
    "is_editable": true,
    "is_hot_reloadable": true
  }
}
```

**错误响应**

- `403 Forbidden`: 权限不足（需要 SuperAdmin）
- `400 Bad Request`: 配置值格式错误
- `404 Not Found`: 配置项不存在

---

## 系统监控接口

### 获取系统监控数据

获取系统监控数据，包括内存使用情况、磁盘空间、应用进程内存占用以及数据库连接池状态。

#### 请求

```http
GET /api/v1/admin/monitor
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "system": {
      "memory": {
        "total_mb": 16384,
        "used_mb": 8192,
        "available_mb": 8192,
        "usage_percent": 50.0
      },
      "disk": {
        "total_gb": 512,
        "used_gb": 256,
        "available_gb": 256,
        "usage_percent": 50.0
      },
      "process_memory_mb": 128
    },
    "database": {
      "max_connections": 10,
      "active_connections": 5,
      "idle_connections": 5,
      "waiting_requests": 0
    },
    "timestamp": "2024-01-15T10:00:00+00:00"
  }
}
```

#### 响应字段说明

**系统信息 (system)**

| 字段 | 类型 | 说明 |
|------|------|------|
| memory | object | 内存使用情况 |
| memory.total_mb | number | 总内存（MB） |
| memory.used_mb | number | 已使用内存（MB） |
| memory.available_mb | number | 可用内存（MB） |
| memory.usage_percent | number | 内存使用率（%） |
| disk | object | 磁盘使用情况 |
| disk.total_gb | number | 总磁盘空间（GB） |
| disk.used_gb | number | 已使用磁盘空间（GB） |
| disk.available_gb | number | 可用磁盘空间（GB） |
| disk.usage_percent | number | 磁盘使用率（%） |
| process_memory_mb | number | 当前应用进程内存占用（MB） |

**数据库连接池信息 (database)**

| 字段 | 类型 | 说明 |
|------|------|------|
| max_connections | integer | 连接池最大连接数 |
| active_connections | integer | 当前活跃连接数 |
| idle_connections | integer | 空闲连接数 |
| waiting_requests | integer | 等待连接的请求数 |

**其他字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| timestamp | string | 数据获取时间（ISO 8601格式） |

---

### 重置配置到默认值

> **权限**: SuperAdmin 专属

将所有配置重置为默认值（依据 config.toml）。

#### 请求

```http
POST /api/v1/admin/configs/reset
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "key": "system.name",
      "value": "Capella Room",
      "value_type": "string",
      "description": "系统名称",
      "category": "system",
      "is_editable": true,
      "is_hot_reloadable": true
    }
  ]
}
```

---

## 审计系统接口

### 查询审计日志

查询系统审计日志，支持多种过滤条件。

#### 请求

```http
GET /api/v1/admin/audit/logs?event_type=user_login&severity=warning&actor_id=uuid&start_time=2024-01-01T00:00:00Z&limit=50&offset=0
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| event_type | string | 否 | 事件类型过滤，详见下方事件类型列表 |
| severity | string | 否 | 严重级别：`info`/`warning`/`error`/`critical` |
| actor_id | string (UUID) | 否 | 操作者ID过滤（谁执行了操作） |
| target_id | string (UUID) | 否 | 目标ID过滤（操作对象） |
| target_type | string | 否 | 目标类型过滤，如 `user`、`room`、`message`、`config` |
| status | string | 否 | 状态过滤：`success`/`failure` |
| start_time | string | 否 | 开始时间（ISO 8601 格式，如 `2024-01-01T00:00:00Z`） |
| end_time | string | 否 | 结束时间（ISO 8601 格式） |
| limit | integer | 否 | 返回数量，默认 50，最大 200 |
| offset | integer | 否 | 偏移量，默认 0 |

#### 参数组合示例

```
# 查询某用户的所有登录记录
GET /api/v1/admin/audit/logs?event_type=user_login&actor_id=550e8400-e29b-41d4-a716-446655440000

# 查询某时间段内的所有错误和严重事件
GET /api/v1/admin/audit/logs?severity=error&start_time=2024-01-01T00:00:00Z&end_time=2024-01-31T23:59:59Z

# 查询针对特定房间的操作
GET /api/v1/admin/audit/logs?target_type=room&target_id=550e8400-e29b-41d4-a716-446655440001

# 分页查询
GET /api/v1/admin/audit/logs?limit=50&offset=100
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "logs": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "event_type": "user_login",
        "severity": "info",
        "actor_id": "550e8400-e29b-41d4-a716-446655440001",
        "actor_name": "testuser",
        "actor_role": "user",
        "target_type": "user",
        "target_id": "550e8400-e29b-41d4-a716-446655440001",
        "action": "login",
        "description": "User logged in successfully",
        "metadata": {
          "ip_address": "192.168.1.1",
          "user_agent": "Mozilla/5.0..."
        },
        "status": "success",
        "error_message": null,
        "created_at": "2024-01-15T08:30:00+00:00"
      }
    ],
    "total": 1000,
    "limit": 50,
    "offset": 0
  }
}
```

#### 事件类型列表

| 事件类型 | 说明 |
|----------|------|
| user_login | 用户登录 |
| user_logout | 用户登出 |
| user_register | 用户注册 |
| user_password_change | 密码修改 |
| user_profile_update | 资料更新 |
| room_create | 创建房间 |
| room_delete | 删除房间 |
| room_member_add | 添加成员 |
| room_member_remove | 移除成员 |
| room_member_role_change | 成员角色变更 |
| message_send | 发送消息 |
| message_edit | 编辑消息 |
| message_delete | 删除消息 |
| message_report | 举报消息 |
| admin_user_disable | 禁用用户 |
| admin_user_role_change | 修改用户角色 |
| admin_user_delete | 删除用户 |
| admin_room_delete | 删除房间 |
| admin_message_delete | 删除消息 |
| admin_config_update | 更新配置 |
| system_login_failure | 登录失败 |
| system_unauthorized_access | 未授权访问 |
| system_rate_limit_triggered | 触发限流 |
| audit_query | 查询审计日志 |
| audit_export | 导出审计日志 |
| audit_stats_query | 查询统计 |
| alert_query | 查询告警 |
| alert_rule_update | 更新告警规则 |
| audit_cleanup | 清理日志 |
| ip_blocked | IP被阻止 |
| ip_whitelist_denied | 白名单拒绝 |
| ip_rate_limited | IP被限流 |
| ip_list_added | 添加IP到列表 |
| ip_list_removed | 从列表移除IP |
| ip_list_updated | 更新IP条目 |

---

### 获取日志详情

获取单条审计日志的详细信息。

#### 请求

```http
GET /api/v1/admin/audit/logs/:id
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "event_type": "user_login",
    "severity": "info",
    "actor_id": "550e8400-e29b-41d4-a716-446655440001",
    "actor_name": "testuser",
    "actor_role": "user",
    "target_type": "user",
    "target_id": "550e8400-e29b-41d4-a716-446655440001",
    "action": "login",
    "description": "User logged in successfully",
    "metadata": {
      "ip_address": "192.168.1.1",
      "user_agent": "Mozilla/5.0...",
      "request_path": "/api/v1/auth/login",
      "request_method": "POST"
    },
    "status": "success",
    "error_message": null,
    "created_at": "2024-01-15T08:30:00+00:00"
  }
}
```

---

### 审计统计信息

获取审计日志的统计信息。

#### 请求

```http
GET /api/v1/admin/audit/stats?start_time=2024-01-01T00:00:00Z&end_time=2024-01-31T23:59:59Z
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "total_logs": 10000,
    "today_logs": 500,
    "week_logs": 3500,
    "month_logs": 15000,
    "logs_by_severity": [
      { "severity": "info", "count": 8000 },
      { "severity": "warning", "count": 1500 },
      { "severity": "error", "count": 400 },
      { "severity": "critical", "count": 100 }
    ],
    "logs_by_event_type": [
      { "event_type": "user_login", "count": 3000 },
      { "event_type": "message_send", "count": 5000 }
    ],
    "logs_by_day": [
      { "date": "2024-01-15", "count": 500 },
      { "date": "2024-01-14", "count": 480 }
    ],
    "alerts_count": 25,
    "new_alerts_count": 5
  }
}
```

---

### 导出审计日志

导出审计日志为 JSON 或 CSV 格式。

#### 请求

```http
GET /api/v1/admin/audit/export?format=json&event_type=user_login&start_time=2024-01-01T00:00:00Z
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| format | string | 是 | 导出格式：`json` 或 `csv` |
| event_type | string | 否 | 事件类型过滤 |
| severity | string | 否 | 严重级别过滤 |
| actor_id | string (UUID) | 否 | 操作者ID过滤 |
| start_time | string | 否 | 开始时间 |
| end_time | string | 否 | 结束时间 |

#### 响应

**成功 (200 OK)**

返回文件下载响应：

- Content-Type: `application/json` 或 `text/csv`
- Content-Disposition: `attachment; filename=audit_logs.json`

---

### 获取安全告警列表

获取系统安全告警列表。

#### 请求

```http
GET /api/v1/admin/audit/alerts?status=new&severity=critical&limit=50&offset=0
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| status | string | 否 | 告警状态：`new`/`acknowledged`/`resolved`/`ignored` |
| severity | string | 否 | 严重级别过滤 |
| alert_type | string | 否 | 告警类型过滤 |
| affected_user_id | string (UUID) | 否 | 受影响用户ID |
| start_time | string | 否 | 开始时间 |
| end_time | string | 否 | 结束时间 |
| limit | integer | 否 | 返回数量，默认 50 |
| offset | integer | 否 | 偏移量，默认 0 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "alerts": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "rule_id": "550e8400-e29b-41d4-a716-446655440001",
        "alert_type": "brute_force_attack",
        "severity": "critical",
        "title": "暴力破解攻击检测",
        "description": "检测到来自 192.168.1.100 的多次登录失败",
        "related_logs": ["uuid1", "uuid2"],
        "source_ip": "192.168.1.100",
        "affected_user": {
          "id": "550e8400-e29b-41d4-a716-446655440002",
          "username": "targetuser",
          "avatar_url": null
        },
        "status": "new",
        "acknowledged_by": null,
        "acknowledged_at": null,
        "resolved_by": null,
        "resolved_at": null,
        "created_at": "2024-01-15T08:30:00+00:00",
        "updated_at": "2024-01-15T08:30:00+00:00"
      }
    ],
    "total": 25,
    "limit": 50,
    "offset": 0
  }
}
```

---

### 更新告警状态

更新安全告警的处理状态。

#### 请求

```http
PUT /api/v1/admin/audit/alerts/:id/status
Content-Type: application/json

{
  "status": "acknowledged"
}
```

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| status | string | 是 | 新状态：`acknowledged`/`resolved`/`ignored` |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "rule_id": "550e8400-e29b-41d4-a716-446655440001",
    "alert_type": "brute_force_attack",
    "severity": "critical",
    "title": "暴力破解攻击检测",
    "description": "检测到来自 192.168.1.100 的多次登录失败",
    "related_logs": ["uuid1", "uuid2"],
    "source_ip": "192.168.1.100",
    "affected_user": {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "username": "targetuser",
      "avatar_url": null
    },
    "status": "acknowledged",
    "acknowledged_by": {
      "id": "550e8400-e29b-41d4-a716-446655440003",
      "username": "admin",
      "avatar_url": null
    },
    "acknowledged_at": "2024-01-15T09:00:00+00:00",
    "resolved_by": null,
    "resolved_at": null,
    "created_at": "2024-01-15T08:30:00+00:00",
    "updated_at": "2024-01-15T09:00:00+00:00"
  }
}
```

---

### 获取告警规则

获取所有告警规则列表。

#### 请求

```http
GET /api/v1/admin/audit/rules
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "rules": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "暴力破解检测",
        "description": "检测多次登录失败",
        "event_type": "system_login_failure",
        "condition": {
          "threshold": 5,
          "time_window_minutes": 10
        },
        "severity": "critical",
        "enabled": true,
        "cooldown_minutes": 30,
        "notify_admins": true,
        "created_at": "2024-01-15T08:30:00+00:00",
        "updated_at": "2024-01-15T08:30:00+00:00"
      }
    ]
  }
}
```

---

### 修改告警规则

> **权限**: SuperAdmin 专属

修改告警规则配置。

#### 请求

```http
PUT /api/v1/admin/audit/rules/:id
Content-Type: application/json

{
  "enabled": false,
  "severity": "warning",
  "cooldown_minutes": 60
}
```

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| name | string | 否 | 规则名称 |
| description | string | 否 | 规则描述 |
| condition | object | 否 | 触发条件，详见下方说明 |
| severity | string | 否 | 严重级别 |
| enabled | boolean | 否 | 是否启用 |
| cooldown_minutes | integer | 否 | 冷却时间（分钟） |
| notify_admins | boolean | 否 | 是否通知管理员 |

#### 告警规则 Condition 格式

`condition` 字段定义告警触发条件，支持以下类型：

**1. 阈值类型（threshold）**

用于检测事件在指定时间窗口内发生次数超过阈值：

```json
{
  "type": "threshold",
  "threshold": 5,
  "time_window_minutes": 10,
  "group_by": ["source_ip"]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 条件类型：`threshold` |
| threshold | integer | 是 | 触发阈值（事件次数） |
| time_window_minutes | integer | 是 | 时间窗口（分钟） |
| group_by | array | 否 | 分组字段，如 `["source_ip"]` 按IP分组统计 |

**2. 频率类型（frequency）**

用于检测事件频率异常：

```json
{
  "type": "frequency",
  "min_count": 10,
  "max_count": 100,
  "time_window_minutes": 60
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 条件类型：`frequency` |
| min_count | integer | 否 | 最小次数（低于此值触发） |
| max_count | integer | 否 | 最大次数（超过此值触发） |
| time_window_minutes | integer | 是 | 时间窗口（分钟） |

**3. 模式匹配类型（pattern）**

用于检测特定模式：

```json
{
  "type": "pattern",
  "pattern": "regex_pattern",
  "field": "description"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 是 | 条件类型：`pattern` |
| pattern | string | 是 | 正则表达式模式 |
| field | string | 是 | 匹配的字段名 |

---

### 告警类型说明

系统支持以下告警类型：

| 告警类型 | 说明 | 默认触发条件 | 严重级别 |
|----------|------|-------------|----------|
| `brute_force_attack` | 暴力破解攻击 | 5分钟内同一IP登录失败超过5次 | critical |
| `unusual_login_pattern` | 异常登录模式 | 用户从不常用IP登录 | warning |
| `privilege_escalation` | 权限提升 | 用户角色被提升为管理员 | warning |
| `data_exfiltration` | 数据泄露风险 | 短时间内大量数据导出 | error |
| `system_anomaly` | 系统异常 | 系统错误率超过阈值 | error |
| `rate_limit_triggered` | 触发限流 | 同一IP频繁触发限流 | warning |
| `unauthorized_access` | 未授权访问 | 多次尝试访问无权限资源 | warning |

---

### 审计日志 Metadata 结构

审计日志的 `metadata` 字段包含以下结构化信息：

```json
{
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
  "request_path": "/api/v1/auth/login",
  "request_method": "POST",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "sess_abc123",
  "device_info": {
    "os": "Windows 10",
    "browser": "Chrome 120.0",
    "device_type": "desktop"
  },
  "location": {
    "country": "CN",
    "city": "Beijing",
    "coordinates": [116.4074, 39.9042]
  }
}
```

**Metadata 字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| ip_address | string | 客户端IP地址 |
| user_agent | string | 用户代理字符串 |
| request_path | string | 请求路径 |
| request_method | string | HTTP方法：GET/POST/PUT/DELETE等 |
| request_id | string | 请求唯一标识 |
| session_id | string | 会话标识 |
| device_info | object | 设备信息 |
| device_info.os | string | 操作系统 |
| device_info.browser | string | 浏览器信息 |
| device_info.device_type | string | 设备类型：desktop/mobile/tablet |
| location | object | 地理位置信息（如启用GeoIP） |
| location.country | string | 国家代码 |
| location.city | string | 城市 |
| location.coordinates | array | 经纬度坐标 [lng, lat] |

---

### 审计日志存储机制

审计日志支持以下存储机制：

**1. Redis Stream 异步写入（阶段 8.6）**
- 审计日志先写入 Redis Stream
- 后台 Consumer Group 异步消费写入 PostgreSQL
- 支持多节点负载均衡
- Redis 不可用时自动降级为直接写入

**2. 本地 Buffer 降级**
- Redis 不可用时，日志暂存内存 Buffer
- Buffer 满或定时刷新时批量写入数据库
- 默认 Buffer 大小：1000条
- 默认刷新间隔：30秒

**3. 自动清理策略**
- 支持按时间自动清理过期日志
- 支持归档后再清理
- 可配置保留天数（默认：90天）

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "暴力破解检测",
    "description": "检测多次登录失败",
    "event_type": "system_login_failure",
    "condition": {
      "threshold": 5,
      "time_window_minutes": 10
    },
    "severity": "warning",
    "enabled": false,
    "cooldown_minutes": 60,
    "notify_admins": true,
    "created_at": "2024-01-15T08:30:00+00:00",
    "updated_at": "2024-01-15T09:00:00+00:00"
  }
}
```

---

### 清理过期日志

> **权限**: SuperAdmin 专属

清理指定天数之前的审计日志。

#### 请求

```http
POST /api/v1/admin/audit/cleanup
Content-Type: application/json

{
  "days": 90,
  "archive": true,
  "archive_dir": "/var/log/capella/archive"
}
```

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| days | integer | 是 | 保留天数，删除此天数之前的日志 |
| archive | boolean | 否 | 是否先归档再删除 |
| archive_dir | string | 否 | 归档目录（archive为true时必填） |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "deleted": 5000,
    "before": "2023-10-17T10:00:00+00:00",
    "archived": true
  }
}
```

---

## IP 安全管理接口

### 查询 IP 列表

查询 IP 黑名单/白名单列表。

#### 请求

```http
GET /api/v1/admin/security/ip-list?list_type=blacklist&limit=50&offset=0
```

#### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| list_type | string | 否 | 列表类型：`whitelist`/`blacklist` |
| ip_address | string | 否 | 按IP地址搜索 |
| limit | integer | 否 | 返回数量，默认 50 |
| offset | integer | 否 | 偏移量，默认 0 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "entries": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "ip_address": "192.168.1.100",
        "ip_range_cidr": null,
        "list_type": "blacklist",
        "description": "暴力破解攻击源",
        "created_by": "550e8400-e29b-41d4-a716-446655440001",
        "created_by_username": "admin",
        "expires_at": "2024-02-15T08:30:00+00:00",
        "created_at": "2024-01-15T08:30:00+00:00",
        "updated_at": "2024-01-15T08:30:00+00:00"
      }
    ],
    "total": 100,
    "limit": 50,
    "offset": 0
  }
}
```

---

### 添加 IP 到列表

将 IP 地址添加到黑名单或白名单。

#### 请求

```http
POST /api/v1/admin/security/ip-list
Content-Type: application/json

{
  "ip_address": "192.168.1.100",
  "ip_range_cidr": null,
  "list_type": "blacklist",
  "description": "暴力破解攻击源",
  "expires_at": "2024-02-15T08:30:00Z"
}
```

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| ip_address | string | 是 | IP 地址 |
| ip_range_cidr | string | 否 | CIDR 范围（如 192.168.1.0/24） |
| list_type | string | 是 | 列表类型：`whitelist`/`blacklist` |
| description | string | 否 | 描述说明 |
| expires_at | string | 否 | 过期时间（ISO 8601） |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "entry": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "ip_address": "192.168.1.100",
      "ip_range_cidr": null,
      "list_type": "blacklist",
      "description": "暴力破解攻击源",
      "created_by": "550e8400-e29b-41d4-a716-446655440001",
      "expires_at": "2024-02-15T08:30:00+00:00",
      "created_at": "2024-01-15T08:30:00+00:00",
      "updated_at": "2024-01-15T08:30:00+00:00"
    },
    "message": "IP entry added successfully"
  }
}
```

---

### 批量添加 IP

批量添加 IP 地址到列表。

#### 请求

```http
POST /api/v1/admin/security/ip-list/batch
Content-Type: application/json

{
  "entries": [
    {
      "ip_address": "192.168.1.100",
      "list_type": "blacklist",
      "description": "攻击源1"
    },
    {
      "ip_address": "192.168.1.101",
      "list_type": "blacklist",
      "description": "攻击源2"
    }
  ]
}
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "success_count": 2,
    "failed_count": 0,
    "failed_ips": [],
    "message": "Added 2 entries, 0 failed"
  }
}
```

---

### 更新 IP 条目

更新 IP 列表条目的信息。

#### 请求

```http
PUT /api/v1/admin/security/ip-list/:id
Content-Type: application/json

{
  "description": "更新后的描述",
  "expires_at": "2024-03-15T08:30:00Z"
}
```

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| description | string | 否 | 新的描述 |
| expires_at | string | 否 | 新的过期时间 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "entry": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "ip_address": "192.168.1.100",
      "list_type": "blacklist",
      "description": "更新后的描述",
      "created_by": "550e8400-e29b-41d4-a716-446655440001",
      "expires_at": "2024-03-15T08:30:00+00:00",
      "created_at": "2024-01-15T08:30:00+00:00",
      "updated_at": "2024-01-15T09:00:00+00:00"
    },
    "message": "IP entry updated successfully"
  }
}
```

---

### 移除 IP

从列表中移除指定的 IP 条目。

#### 请求

```http
DELETE /api/v1/admin/security/ip-list/:id
```

#### 响应

**成功 (204 No Content)**

无响应体

---

### 检查 IP 状态

检查指定 IP 地址的安全状态。

#### 请求

```http
POST /api/v1/admin/security/ip-check
Content-Type: application/json

{
  "ip_address": "192.168.1.100"
}
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "ip_address": "192.168.1.100",
    "allowed": false,
    "reason": "IP address is in blacklist",
    "list_type": "blacklist"
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| ip_address | string | IP 地址 |
| allowed | boolean | 是否允许访问 |
| reason | string | 拒绝原因（如果不允许） |
| list_type | string | 所属的列表类型 |

---

### 获取安全统计

获取 IP 安全系统的统计信息。

#### 请求

```http
GET /api/v1/admin/security/stats
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "total_whitelist": 10,
    "total_blacklist": 50,
    "expired_entries": 5,
    "active_entries": 55
  }
}
```

---

### 刷新缓存

刷新 IP 安全列表的内存缓存。

#### 请求

```http
POST /api/v1/admin/security/refresh-cache
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "message": "IP security cache refreshed successfully"
  }
}
```

---

### 清理过期条目

清理所有已过期的 IP 列表条目。

#### 请求

```http
POST /api/v1/admin/security/cleanup-expired
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "deleted_count": 5,
    "message": "Cleaned up 5 expired entries"
  }
}
```

---

### 获取白名单模式状态

获取当前白名单模式的状态。

#### 请求

```http
GET /api/v1/admin/security/whitelist-mode
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "enabled": false
  }
}
```

---

### 设置白名单模式

> **权限**: SuperAdmin 专属

启用或禁用白名单模式。启用后，只有白名单中的 IP 可以访问系统。

#### 请求

```http
POST /api/v1/admin/security/whitelist-mode
Content-Type: application/json

{
  "enabled": true
}
```

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| enabled | boolean | 是 | `true` 启用白名单模式，`false` 禁用 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "enabled": true,
    "message": "Whitelist mode enabled"
  }
}
```

---

## 错误响应

管理员接口使用统一的错误响应格式：

```json
{
  "success": false,
  "data": null,
  "message": "错误描述信息"
}
```

### 状态码说明

| 状态码 | 说明 |
|--------|------|
| 400 | 请求参数错误 |
| 401 | 未认证（缺少 Token） |
| 403 | 权限不足（需要更高权限） |
| 404 | 资源不存在 |
| 500 | 服务器内部错误 |

### 常见错误

| 错误信息 | 说明 | 处理建议 |
|----------|------|----------|
| 需要管理员权限 | 当前用户不是管理员 | 使用管理员账号登录 |
| 需要超级管理员权限 | 需要 SuperAdmin 角色 | 使用超级管理员账号 |
| 不能修改超级管理员 | 试图修改 SuperAdmin | 超级管理员不能被修改 |
| 不能禁用超级管理员 | 试图禁用 SuperAdmin | 超级管理员不能被禁用 |
| 不能删除超级管理员 | 试图删除 SuperAdmin | 超级管理员不能被删除 |

---

## 使用示例

### cURL 示例

```bash
# 设置基础变量
BASE_URL="http://localhost:8080"
ADMIN_TOKEN="your_admin_jwt_token"

# 获取用户列表
curl -X GET "${BASE_URL}/api/v1/admin/users?page=1&page_size=20" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}"

# 禁用用户
curl -X PUT "${BASE_URL}/api/v1/admin/users/550e8400-e29b-41d4-a716-446655440000/status" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"disabled": true}'

# 修改用户角色（需要 SuperAdmin）
curl -X PUT "${BASE_URL}/api/v1/admin/users/550e8400-e29b-41d4-a716-446655440000/role" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"role": "admin"}'

# 获取系统统计
curl -X GET "${BASE_URL}/api/v1/admin/stats" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}"

# 查询审计日志
curl -X GET "${BASE_URL}/api/v1/admin/audit/logs?limit=50&severity=warning" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}"

# 添加 IP 到黑名单
curl -X POST "${BASE_URL}/api/v1/admin/security/ip-list" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "ip_address": "192.168.1.100",
    "list_type": "blacklist",
    "description": "暴力破解攻击源"
  }'

# 导出审计日志
curl -X GET "${BASE_URL}/api/v1/admin/audit/export?format=json" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -o audit_logs.json
```

### JavaScript 示例

```javascript
const BASE_URL = 'http://localhost:8080';
const ADMIN_TOKEN = 'your_admin_jwt_token';

// 封装请求函数
async function adminRequest(endpoint, options = {}) {
  const response = await fetch(`${BASE_URL}${endpoint}`, {
    ...options,
    headers: {
      'Authorization': `Bearer ${ADMIN_TOKEN}`,
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || 'Request failed');
  }
  
  return response.json();
}

// 获取用户列表
async function getUsers(page = 1, pageSize = 20, search = '') {
  const params = new URLSearchParams({ page, page_size: pageSize });
  if (search) params.append('search', search);
  
  return adminRequest(`/api/v1/admin/users?${params}`);
}

// 禁用用户
async function disableUser(userId) {
  return adminRequest(`/api/v1/admin/users/${userId}/status`, {
    method: 'PUT',
    body: JSON.stringify({ disabled: true }),
  });
}

// 获取系统统计
async function getSystemStats() {
  return adminRequest('/api/v1/admin/stats');
}

// 查询审计日志
async function queryAuditLogs(filters = {}) {
  const params = new URLSearchParams();
  Object.entries(filters).forEach(([key, value]) => {
    if (value) params.append(key, value);
  });
  
  return adminRequest(`/api/v1/admin/audit/logs?${params}`);
}

// 使用示例
(async () => {
  try {
    // 获取用户列表
    const users = await getUsers(1, 10);
    console.log('Users:', users.data);
    
    // 获取系统统计
    const stats = await getSystemStats();
    console.log('Stats:', stats.data);
    
    // 查询最近的告警
    const logs = await queryAuditLogs({ 
      severity: 'warning',
      limit: 20 
    });
    console.log('Audit Logs:', logs.data);
  } catch (error) {
    console.error('Error:', error.message);
  }
})();
```

---

## WebSocket 管理员功能

管理员可以通过 WebSocket 接收实时系统通知和安全告警。

### 连接方式

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

// 连接后发送认证消息
ws.onopen = () => {
  ws.send(JSON.stringify({
    type: 'Auth',
    token: 'your_admin_jwt_token'
  }));
};

// 接收消息
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  switch (message.type) {
    case 'SystemNotification':
      console.log('系统通知:', message.data);
      break;
    case 'AlertNotification':
      console.log('安全告警:', message.data);
      break;
  }
};
```

### 管理员专属消息类型

| 消息类型 | 说明 | 触发条件 |
|----------|------|----------|
| `SystemNotification` | 系统通知 | 系统状态变更 |
| `AlertNotification` | 安全告警 | 检测到安全事件 |
| `AuditLogNotification` | 审计日志通知 | 重要操作记录 |

---

## Redis 与分布式管理接口

### 获取 Redis 连接状态

获取当前 Redis 连接状态和连接池信息。

#### 请求

```http
GET /api/v1/admin/redis/status
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "enabled": true,
    "connected": true,
    "pool_size": 10,
    "active_connections": 3,
    "idle_connections": 7,
    "cluster_mode": false,
    "nodes": [
      {
        "id": "node-1",
        "address": "redis://localhost:6379",
        "connected": true,
        "latency_ms": 0.5
      }
    ]
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| enabled | boolean | Redis 功能是否启用 |
| connected | boolean | 是否已连接到 Redis |
| pool_size | integer | 连接池总大小 |
| active_connections | integer | 当前活跃连接数 |
| idle_connections | integer | 当前空闲连接数 |
| cluster_mode | boolean | 是否为集群模式 |
| nodes | array | 节点列表（集群模式下有多个） |
| nodes[].id | string | 节点标识 |
| nodes[].address | string | 节点地址 |
| nodes[].connected | boolean | 节点连接状态 |
| nodes[].latency_ms | number/null | 节点延迟（毫秒） |

---

### 获取 Redis 统计信息

获取 Redis 服务器的详细统计信息，包括 Pub/Sub、Stream、内存使用等。

#### 请求

```http
GET /api/v1/admin/redis/stats
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "pubsub_channels": 15,
    "pubsub_patterns": 3,
    "stream_messages": 1250,
    "stream_consumers": 2,
    "memory_used": 5242880,
    "memory_peak": 8388608,
    "total_commands_processed": 150000,
    "ops_per_second": 150,
    "hit_rate": 0.95,
    "uptime_seconds": 86400
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| pubsub_channels | integer | 当前 Pub/Sub 频道数 |
| pubsub_patterns | integer | 当前 Pub/Sub 模式订阅数 |
| stream_messages | integer | Stream 中待处理的消息数 |
| stream_consumers | integer | Stream 消费者数量 |
| memory_used | integer | 当前内存使用量（字节） |
| memory_peak | integer | 内存使用峰值（字节） |
| total_commands_processed | integer | 处理的命令总数 |
| ops_per_second | integer | 每秒操作数 |
| hit_rate | number | 缓存命中率（0-1） |
| uptime_seconds | integer | Redis 运行时间（秒） |

---

### 刷新 Redis 连接

> **权限**: SuperAdmin 专属

关闭并重新建立所有 Redis 连接，用于连接异常恢复。

#### 请求

```http
POST /api/v1/admin/redis/refresh
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "refreshed": true,
    "message": "Redis connections refreshed successfully"
  }
}
```

**错误响应**

- `403 Forbidden`: 权限不足（需要 SuperAdmin）
- `503 Service Unavailable`: Redis 连接刷新失败

---

### 触发配置同步

> **权限**: SuperAdmin 专属

触发配置同步到所有集群节点，用于配置热更新。

#### 请求

```http
POST /api/v1/admin/config/sync
Content-Type: application/json

{
  "config_keys": ["system.name", "websocket.max_connections"]
}
```

#### 请求体

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| config_keys | array | 否 | 要同步的配置键列表，不传则同步所有配置 |

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "synced": true,
    "nodes_count": 3,
    "synced_nodes": 3,
    "failed_nodes": 0,
    "message": "Configuration synced to all nodes"
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| synced | boolean | 是否同步成功 |
| nodes_count | integer | 集群节点总数 |
| synced_nodes | integer | 成功同步的节点数 |
| failed_nodes | integer | 同步失败的节点数 |
| message | string | 同步结果描述 |

**错误响应**

- `403 Forbidden`: 权限不足（需要 SuperAdmin）
- `503 Service Unavailable`: 部分节点同步失败

---

### 获取配置同步状态

获取配置同步的当前状态，包括上次同步时间和待处理变更。

#### 请求

```http
GET /api/v1/admin/config/sync/status
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "sync_enabled": true,
    "last_sync_at": "2026-04-26T10:30:00+00:00",
    "nodes_total": 3,
    "nodes_synced": 3,
    "pending_changes": 0,
    "sync_latency_ms": 15.5
  }
}
```

#### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| sync_enabled | boolean | 配置同步功能是否启用 |
| last_sync_at | string/null | 上次同步时间（ISO 8601格式） |
| nodes_total | integer | 集群节点总数 |
| nodes_synced | integer | 已同步的节点数 |
| pending_changes | integer | 待同步的配置变更数 |
| sync_latency_ms | number/null | 同步延迟（毫秒） |

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
