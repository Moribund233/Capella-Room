# 用户接口文档

> **API 前缀**: `/api/v1`
> **认证要求**: 所有接口均需要认证（需要携带 Access Token）

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/users/me` | 获取当前用户信息 |
| PUT | `/api/v1/users/me` | 更新当前用户信息 |
| PUT | `/api/v1/users/me/password` | 修改密码 |
| GET | `/api/v1/users/me/rooms` | 获取我的聊天室列表 |
| POST | `/api/v1/users/logout` | 登出 |
| GET | `/api/v1/users` | 获取用户列表 |
| GET | `/api/v1/users/:user_id` | 获取指定用户信息 |

---

## 获取当前用户信息

### 请求

```http
GET /api/v1/users/me
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "avatar_url": null,
    "status": "online",
    "is_active": true,
    "role": "user",
    "created_at": "2024-01-15T08:30:00Z"
  }
}
```

**失败 - Token 无效 (401 Unauthorized)**

```json
{
  "success": false,
  "code": "AUTH_ERROR",
  "error": "认证失败",
  "message": "认证失败: Token已过期"
}
```

**失败 - 用户不存在 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源不存在",
  "message": "资源不存在: 用户不存在"
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 用户唯一标识 |
| `username` | string | 用户名 |
| `email` | string | 邮箱地址 |
| `avatar_url` | string \| null | 头像 URL |
| `status` | string | 在线状态：`online` / `offline` / `away` |
| `is_active` | boolean | 账号状态：`true` 启用 / `false` 禁用 |
| `role` | string | 用户角色：`user` / `admin` / `super_admin` |
| `created_at` | string (ISO 8601) | 创建时间 |

---

## 更新当前用户信息

### 请求

```http
PUT /api/v1/users/me
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "username": "newusername",
  "avatar_url": "https://example.com/avatar.jpg"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `username` | string | 否 | 新用户名，3-20 个字符 |
| `avatar_url` | string | 否 | 新头像 URL |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "newusername",
    "email": "test@example.com",
    "avatar_url": "https://example.com/avatar.jpg",
    "status": "online",
    "is_active": true,
    "role": "user",
    "created_at": "2024-01-15T08:30:00Z"
  }
}
```

**失败 - 用户名已存在 (409 Conflict)**

```json
{
  "success": false,
  "code": "CONFLICT",
  "error": "资源已存在",
  "message": "资源已存在: 用户名已被使用"
}
```

**失败 - 参数验证失败 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "验证失败: username: 长度必须在 3-20 之间"
}
```

### 说明

- 可以只更新部分字段，未提供的字段保持不变
- 用户名修改后需确保唯一性
- 头像 URL 可以是任意有效的 HTTP/HTTPS URL

---

## 修改密码

### 请求

```http
PUT /api/v1/users/me/password
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "old_password": "OldPass123!",
  "new_password": "NewPass456!"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `old_password` | string | 是 | 当前密码 |
| `new_password` | string | 是 | 新密码，至少 8 位，需包含大小写字母和数字 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "密码修改成功"
}
```

**失败 - 原密码错误 (401 Unauthorized)**

```json
{
  "success": false,
  "code": "AUTH_ERROR",
  "error": "认证失败",
  "message": "认证失败: 原密码错误"
}
```

**失败 - 新密码与原密码相同 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "新密码不能与原密码相同"
}
```

**失败 - 新密码强度不足 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "验证失败: new_password: 密码强度不足"
}
```

### 说明

- 修改密码后会记录审计日志
- 建议在修改密码后要求用户重新登录
- 新密码不能与旧密码相同

---

## 获取我的聊天室列表

### 请求

```http
GET /api/v1/users/me/rooms
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
[
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
    "max_members": 50,
    "member_count": 5,
    "created_at": "2024-01-15T08:30:00Z",
    "updated_at": "2024-01-15T08:30:00Z"
  }
]
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
| `max_members` | number | 最大成员数 |
| `member_count` | number | 当前成员数量 |
| `created_at` | string (ISO 8601) | 创建时间 |
| `updated_at` | string (ISO 8601) | 更新时间 |

### 说明

- 返回当前用户加入的所有聊天室
- 包括用户自己创建的聊天室
- 按加入时间倒序排列

---

## 用户登出

### 请求

```http
POST /api/v1/users/logout
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "登出成功"
}
```

### 说明

- 登出后会将用户状态更新为 `offline`
- 会记录登出审计日志
- 客户端需要清除本地存储的 Token

---

## 获取用户统计信息

### 请求

```http
GET /api/v1/users/me/stats
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "joined_rooms": 5,
    "total_messages": 128,
    "online_hours": 168
  }
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `joined_rooms` | number | 加入的聊天室数量 |
| `total_messages` | number | 发送的消息总数 |
| `online_hours` | number | 在线时长（小时） |

### 说明

- 统计数据实时计算
- `online_hours` 基于用户注册时间估算

---

## 获取用户列表

### 请求

```http
GET /api/v1/users?search={keyword}&limit={limit}&offset={offset}
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `search` | string | 否 | 搜索关键词（匹配用户名或邮箱） |
| `limit` | number | 否 | 每页数量，默认 20，最大 100 |
| `offset` | number | 否 | 偏移量，默认 0 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "testuser",
        "email": "test@example.com",
        "avatar_url": null,
        "status": "online",
        "role": "user",
        "created_at": "2024-01-15T08:30:00Z"
      }
    ],
    "total": 100,
    "limit": 20,
    "offset": 0
  }
}
```

### 响应字段说明

#### 分页信息

| 字段 | 类型 | 说明 |
|------|------|------|
| `users` | array | 用户列表 |
| `total` | number | 总用户数 |
| `limit` | number | 每页数量 |
| `offset` | number | 当前偏移量 |

#### 用户对象

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 用户唯一标识 |
| `username` | string | 用户名 |
| `email` | string | 邮箱地址 |
| `avatar_url` | string \| null | 头像 URL |
| `status` | string | 在线状态：`online` / `offline` / `away` |
| `is_active` | boolean | 账号状态：`true` 启用 / `false` 禁用 |
| `role` | string | 用户角色 |
| `created_at` | string (ISO 8601) | 创建时间 |

### 说明

- 支持分页查询，默认每页 20 条
- 支持按用户名或邮箱搜索
- 搜索时会忽略大小写

---

## 获取指定用户信息

### 请求

```http
GET /api/v1/users/{user_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `user_id` | string (UUID) | 用户唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "avatar_url": null,
    "status": "online",
    "is_active": true,
    "role": "user",
    "created_at": "2024-01-15T08:30:00Z"
  }
}

**失败 - 用户不存在 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源不存在",
  "message": "资源不存在: 用户不存在"
}
```

### 说明

- 可用于查看其他用户的公开信息
- 返回的信息与获取当前用户相同

---

## 使用示例

### cURL 示例

```bash
# 获取当前用户信息
curl -X GET http://localhost:3000/api/v1/users/me \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 更新用户信息
curl -X PUT http://localhost:3000/api/v1/users/me \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newname",
    "avatar_url": "https://example.com/avatar.jpg"
  }'

# 修改密码
curl -X PUT http://localhost:3000/api/v1/users/me/password \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "old_password": "OldPass123!",
    "new_password": "NewPass456!"
  }'

# 获取我的聊天室列表
curl -X GET http://localhost:3000/api/v1/users/me/rooms \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 登出
curl -X POST http://localhost:3000/api/v1/users/logout \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 获取用户列表（分页）
curl -X GET "http://localhost:3000/api/v1/users?limit=10&offset=0" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 搜索用户
curl -X GET "http://localhost:3000/api/v1/users?search=test" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 获取指定用户信息
curl -X GET http://localhost:3000/api/v1/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### JavaScript 示例

```javascript
// 获取当前用户信息
async function getCurrentUser() {
  const response = await fetch('http://localhost:3000/api/v1/users/me', {
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

// 更新用户信息
async function updateUser(username, avatarUrl) {
  const response = await fetch('http://localhost:3000/api/v1/users/me', {
    method: 'PUT',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ username, avatar_url: avatarUrl })
  });
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 修改密码
async function changePassword(oldPassword, newPassword) {
  const response = await fetch('http://localhost:3000/api/v1/users/me/password', {
    method: 'PUT',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      old_password: oldPassword,
      new_password: newPassword
    })
  });
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 获取用户列表（支持分页和搜索）
async function listUsers(options = {}) {
  const { search, limit = 20, offset = 0 } = options;
  
  const params = new URLSearchParams();
  if (search) params.append('search', search);
  params.append('limit', limit);
  params.append('offset', offset);
  
  const response = await fetch(`http://localhost:3000/api/v1/users?${params}`, {
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

// 登出
async function logout() {
  const response = await fetch('http://localhost:3000/api/v1/users/logout', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`
    }
  });
  
  const data = await response.json();
  
  if (data.success) {
    // 清除本地存储的 Token
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    localStorage.removeItem('expires_at');
    return data.data;
  } else {
    throw new Error(data.message);
  }
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
| 404 | 资源不存在 | 用户不存在 |
| 409 | 资源冲突 | 用户名已被使用 |
| 500 | 服务器错误 | 内部服务器错误 |

### 业务错误码 (code)

| 错误码 | HTTP 状态码 | 说明 | 处理建议 |
|--------|------------|------|---------|
| `VALIDATION_ERROR` | 400 | 参数验证失败 | 检查请求参数是否符合要求 |
| `AUTH_ERROR` | 401 | 认证失败 | 检查 Token 是否过期，尝试刷新或重新登录 |
| `NOT_FOUND` | 404 | 资源不存在 | 检查用户 ID 是否正确 |
| `CONFLICT` | 409 | 资源冲突 | 更换用户名后重试 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 | 稍后重试或联系管理员 |

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
