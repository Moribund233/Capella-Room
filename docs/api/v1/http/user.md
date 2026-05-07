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
| **GET** | **`/api/v1/users/search`** | **搜索用户** |
| GET | `/api/v1/users/:user_id` | 获取指定用户信息 |
| **GET** | **`/api/v1/users/friends`** | **获取好友列表** |
| **POST** | **`/api/v1/users/friends/requests`** | **发送好友请求** |
| **GET** | **`/api/v1/users/friends/requests/received`** | **获取收到的好友请求** |
| **GET** | **`/api/v1/users/friends/requests/sent`** | **获取发送的好友请求** |
| **POST** | **`/api/v1/users/friends/requests/handle`** | **处理好友请求** |
| **DELETE** | **`/api/v1/users/friends/requests/:id`** | **取消好友请求** |
| **DELETE** | **`/api/v1/users/friends/:id`** | **删除好友** |

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

# 用户设置接口

> **API 前缀**: `/api/v1`
> **认证要求**: 所有接口均需要认证

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/users/me/settings` | 获取用户设置 |
| PATCH | `/api/v1/users/me/settings` | 部分更新用户设置 |

---

## 获取用户设置

### 请求

```http
GET /api/v1/users/me/settings
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "notification": {
      "private_message": true,
      "mentioned": true,
      "room_invitation": true,
      "system_notification": true,
      "file_upload_complete": true,
      "sound_enabled": true,
      "desktop_notification": true,
      "do_not_disturb": false
    },
    "privacy": {
      "online_status_visibility": "everyone",
      "profile_visibility": "everyone",
      "allow_stranger_message": true,
      "allow_room_invitation": true,
      "single_device_login": false
    },
    "message": {
      "message_preview": true,
      "read_receipt": true,
      "typing_indicator": true,
      "do_not_disturb": false
    },
    "language": {
      "language": "zh-CN",
      "timezone": "Asia/Shanghai",
      "time_format": "24h",
      "date_format": "YYYY-MM-DD",
      "first_day_of_week": "monday"
    },
    "accessibility": {
      "font_size": "medium",
      "reduce_motion": false,
      "high_contrast": false,
      "dense_mode": false
    },
    "media": {
      "auto_download_media": true,
      "save_media_gallery": false,
      "image_quality": "high",
      "auto_play_video": true,
      "auto_play_audio": false
    }
  }
}
```

### 响应字段说明

#### 通知设置 (notification)

| 字段 | 类型 | 说明 |
|------|------|------|
| `private_message` | boolean | 私信通知 |
| `mentioned` | boolean | @提及通知 |
| `room_invitation` | boolean | 房间邀请通知 |
| `system_notification` | boolean | 系统通知 |
| `file_upload_complete` | boolean | 文件上传完成通知 |
| `sound_enabled` | boolean | 声音提醒 |
| `desktop_notification` | boolean | 桌面通知 |
| `do_not_disturb` | boolean | 勿扰模式 |

#### 隐私设置 (privacy)

| 字段 | 类型 | 说明 |
|------|------|------|
| `online_status_visibility` | string | 在线状态可见性: `everyone`/`friends`/`nobody` |
| `profile_visibility` | string | 个人资料可见性: `everyone`/`friends`/`nobody` |
| `allow_stranger_message` | boolean | 允许陌生人私信 |
| `allow_room_invitation` | boolean | 允许房间邀请 |
| `single_device_login` | boolean | 单设备登录开关 |

#### 消息设置 (message)

| 字段 | 类型 | 说明 |
|------|------|------|
| `message_preview` | boolean | 消息预览 |
| `read_receipt` | boolean | 已读回执 |
| `typing_indicator` | boolean | 输入状态指示器 |
| `do_not_disturb` | boolean | 消息勿扰 |

#### 语言设置 (language)

| 字段 | 类型 | 说明 |
|------|------|------|
| `language` | string | 语言代码，如 `zh-CN` |
| `timezone` | string | 时区，如 `Asia/Shanghai` |
| `time_format` | string | 时间格式: `12h`/`24h` |
| `date_format` | string | 日期格式: `YYYY-MM-DD`/`DD/MM/YYYY`/`MM/DD/YYYY` |
| `first_day_of_week` | string | 周起始日: `monday`/`sunday` |

#### 无障碍设置 (accessibility)

| 字段 | 类型 | 说明 |
|------|------|------|
| `font_size` | string | 字体大小: `small`/`medium`/`large` |
| `reduce_motion` | boolean | 减少动画 |
| `high_contrast` | boolean | 高对比度 |
| `dense_mode` | boolean | 紧凑模式 |

#### 媒体设置 (media)

| 字段 | 类型 | 说明 |
|------|------|------|
| `auto_download_media` | boolean | 自动下载媒体 |
| `save_media_gallery` | boolean | 保存到相册 |
| `image_quality` | string | 图片质量: `original`/`high`/`medium`/`low` |
| `auto_play_video` | boolean | 自动播放视频 |
| `auto_play_audio` | boolean | 自动播放音频 |

---

## 部分更新用户设置

### 请求

```http
PATCH /api/v1/users/me/settings
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

只传递需要修改的分组，未传递的分组保持不变：

```json
{
  "privacy": {
    "single_device_login": true
  }
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `notification` | object | 否 | 通知设置对象 |
| `privacy` | object | 否 | 隐私设置对象 |
| `message` | object | 否 | 消息设置对象 |
| `language` | object | 否 | 语言设置对象 |
| `accessibility` | object | 否 | 无障碍设置对象 |
| `media` | object | 否 | 媒体设置对象 |

### 响应

**成功 (200 OK)**

返回更新后的完整设置（同获取设置接口）。

---

# 账号安全接口

> **API 前缀**: `/api/v1`
> **认证要求**: 所有接口均需要认证

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/users/me/security/overview` | 获取账号安全概览 |
| GET | `/api/v1/users/me/devices` | 获取登录设备列表 |
| DELETE | `/api/v1/users/me/devices/:device_id` | 登出指定设备 |
| POST | `/api/v1/users/me/devices/:device_id/block` | 禁用指定设备 |
| POST | `/api/v1/users/me/devices/:device_id/unblock` | 启用被禁用的设备 |
| POST | `/api/v1/users/me/devices/terminate-others` | 登出所有其他设备 |
| GET | `/api/v1/users/me/login-history` | 获取登录历史 |
| GET | `/api/v1/users/me/login-history/suspicious` | 获取可疑登录记录 |

---

## 获取账号安全概览

### 请求

```http
GET /api/v1/users/me/security/overview
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "active_devices_count": 2,
    "recent_logins": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "ip_address": "192.168.1.100",
        "device_name": "Chrome on Windows",
        "device_type": "desktop",
        "location": "Beijing, China",
        "login_status": "success",
        "risk_level": "low",
        "is_suspicious": false,
        "created_at": "2024-01-15T08:30:00Z"
      }
    ],
    "has_suspicious_activity": false,
    "abnormal_login_alert": true
  }
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `active_devices_count` | number | 活跃设备数量 |
| `recent_logins` | array | 最近登录记录列表 |
| `has_suspicious_activity` | boolean | 是否存在可疑活动 |
| `abnormal_login_alert` | boolean | 是否开启异常登录提醒 |

---

## 获取登录设备列表

### 请求

```http
GET /api/v1/users/me/devices
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "device_name": "Chrome on Windows",
      "device_type": "desktop",
      "ip_address": "192.168.1.100/32",
      "location": "Beijing, China",
      "is_current": true,
      "is_active": true,
      "is_blocked": false,
      "last_active_at": "2024-01-15T08:30:00Z",
      "created_at": "2024-01-15T08:00:00Z"
    }
  ]
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 设备/会话唯一标识 |
| `device_name` | string | 设备名称 |
| `device_type` | string | 设备类型: `mobile`/`tablet`/`desktop`/`unknown` |
| `ip_address` | string | IP地址（CIDR格式） |
| `location` | string \| null | 地理位置 |
| `is_current` | boolean | 是否为当前设备 |
| `is_active` | boolean | 是否活跃 |
| `is_blocked` | boolean | 是否被禁用 |
| `last_active_at` | string (ISO 8601) | 最后活跃时间 |
| `created_at` | string (ISO 8601) | 创建时间 |

---

## 登出指定设备

远程登出其他设备，当前设备的 Token 将失效。

### 请求

```http
DELETE /api/v1/users/me/devices/{device_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `device_id` | string (UUID) | 设备/会话 ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "message": "设备已成功登出"
  }
}
```

**失败 - 不能登出当前设备 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "不能终止当前会话，请使用登出功能"
}
```

---

## 禁用指定设备

被禁用的设备无法使用旧 Token 登录，需要重新登录。

### 请求

```http
POST /api/v1/users/me/devices/{device_id}/block
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `device_id` | string (UUID) | 设备/会话 ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "message": "设备已禁用，该设备无法再使用旧 Token 登录"
  }
}
```

**失败 - 不能禁用当前设备 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "不能禁用当前设备"
}
```

---

## 启用被禁用的设备

将设备从禁用状态恢复，但用户需要重新登录。

### 请求

```http
POST /api/v1/users/me/devices/{device_id}/unblock
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `device_id` | string (UUID) | 设备/会话 ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "message": "设备已启用"
  }
}
```

---

## 登出所有其他设备

一键登出除当前设备外的所有活跃设备。

### 请求

```http
POST /api/v1/users/me/devices/terminate-others
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "message": "其他设备已成功登出",
    "terminated_count": 3
  }
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `message` | string | 操作结果消息 |
| `terminated_count` | number | 被登出的设备数量 |

---

## 获取登录历史

### 请求

```http
GET /api/v1/users/me/login-history?limit=20&offset=0
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
      "ip_address": "192.168.1.100",
      "device_name": "Chrome on Windows",
      "device_type": "desktop",
      "location": "Beijing, China",
      "login_status": "success",
      "risk_level": "low",
      "is_suspicious": false,
      "failure_reason": null,
      "created_at": "2024-01-15T08:30:00Z"
    }
  ],
  "pagination": {
    "total": 50,
    "limit": 20,
    "offset": 0
  }
}
```

### 响应字段说明

#### 登录记录

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 记录唯一标识 |
| `ip_address` | string | IP地址 |
| `device_name` | string | 设备名称 |
| `device_type` | string | 设备类型 |
| `location` | string \| null | 地理位置 |
| `login_status` | string | 登录状态: `success`/`failed`/`blocked` |
| `risk_level` | string | 风险等级: `low`/`medium`/`high` |
| `is_suspicious` | boolean | 是否可疑 |
| `failure_reason` | string \| null | 失败原因 |
| `created_at` | string (ISO 8601) | 登录时间 |

#### 分页信息

| 字段 | 类型 | 说明 |
|------|------|------|
| `total` | number | 总记录数 |
| `limit` | number | 每页数量 |
| `offset` | number | 当前偏移量 |

---

## 获取可疑登录记录

获取被标记为可疑的登录记录。

### 请求

```http
GET /api/v1/users/me/login-history/suspicious?limit=20&offset=0
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `limit` | number | 否 | 每页数量，默认 20，最大 100 |
| `offset` | number | 否 | 偏移量，默认 0 |

### 响应

同登录历史接口，但只返回 `is_suspicious` 为 `true` 的记录。

---

## 使用示例

### cURL 示例

```bash
# 获取用户设置
curl -X GET http://localhost:3000/api/v1/users/me/settings \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 更新隐私设置（开启单设备登录）
curl -X PATCH http://localhost:3000/api/v1/users/me/settings \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "privacy": {
      "single_device_login": true
    }
  }'

# 获取账号安全概览
curl -X GET http://localhost:3000/api/v1/users/me/security/overview \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 获取登录设备列表
curl -X GET http://localhost:3000/api/v1/users/me/devices \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 登出指定设备
curl -X DELETE http://localhost:3000/api/v1/users/me/devices/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 禁用指定设备
curl -X POST http://localhost:3000/api/v1/users/me/devices/550e8400-e29b-41d4-a716-446655440000/block \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 启用被禁用的设备
curl -X POST http://localhost:3000/api/v1/users/me/devices/550e8400-e29b-41d4-a716-446655440000/unblock \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 登出所有其他设备
curl -X POST http://localhost:3000/api/v1/users/me/devices/terminate-others \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 获取登录历史
curl -X GET "http://localhost:3000/api/v1/users/me/login-history?limit=10&offset=0" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 获取可疑登录记录
curl -X GET "http://localhost:3000/api/v1/users/me/login-history/suspicious?limit=10" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### JavaScript 示例

```javascript
// 获取用户设置
async function getUserSettings() {
  const response = await fetch('http://localhost:3000/api/v1/users/me/settings', {
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`
    }
  });
  const data = await response.json();
  return data.success ? data.data : null;
}

// 更新单设备登录设置
async function updateSingleDeviceLogin(enabled) {
  const response = await fetch('http://localhost:3000/api/v1/users/me/settings', {
    method: 'PATCH',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      privacy: { single_device_login: enabled }
    })
  });
  const data = await response.json();
  return data.success;
}

// 获取登录设备列表
async function getDevices() {
  const response = await fetch('http://localhost:3000/api/v1/users/me/devices', {
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`
    }
  });
  const data = await response.json();
  return data.success ? data.data : [];
}

// 禁用设备
async function blockDevice(deviceId) {
  const response = await fetch(
    `http://localhost:3000/api/v1/users/me/devices/${deviceId}/block`,
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  const data = await response.json();
  return data.success;
}

// 登出所有其他设备
async function terminateOtherDevices() {
  const response = await fetch(
    'http://localhost:3000/api/v1/users/me/devices/terminate-others',
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  const data = await response.json();
  return data.success ? data.data.terminated_count : 0;
}
```

---

## 错误码汇总

### 账号安全相关错误码

| 错误码 | HTTP 状态码 | 说明 | 处理建议 |
|--------|------------|------|---------|
| `DEVICE_NOT_FOUND` | 404 | 设备不存在 | 检查设备 ID 是否正确 |
| `DEVICE_BLOCKED` | 403 | 设备已被禁用 | 先启用设备或联系管理员 |
| `CANNOT_BLOCK_CURRENT_DEVICE` | 400 | 不能禁用当前设备 | 使用其他设备操作 |
| `CANNOT_TERMINATE_CURRENT_SESSION` | 400 | 不能终止当前会话 | 使用登出功能 |

---

## 搜索用户

### 请求

```http
GET /api/v1/users/search?keyword={keyword}&limit={limit}&offset={offset}
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `keyword` | string | 是 | 搜索关键词（匹配用户名） |
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
        "avatar_url": null,
        "status": "online"
      }
    ],
    "total": 1
  }
}
```

### 说明

- 搜索结果受目标用户隐私设置影响
- 如果目标用户的 `profile_visibility` 设置为 `Friends`，则只有好友可见
- 如果设置为 `Nobody`，则不可搜索到

---

## 获取好友列表

### 请求

```http
GET /api/v1/users/friends
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "username": "friend_user",
      "avatar_url": null,
      "status": "online",
      "is_friend": true
    }
  ]
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 好友用户ID |
| `username` | string | 好友用户名 |
| `avatar_url` | string \| null | 好友头像URL |
| `status` | string | 在线状态：online/offline/away |
| `is_friend` | boolean | 是否为好友 |

---

## 发送好友请求

### 请求

```http
POST /api/v1/users/friends/requests
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "receiver_id": "550e8400-e29b-41d4-a716-446655440002",
  "message": "你好，我想加你为好友"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `receiver_id` | string (UUID) | 是 | 接收者用户ID |
| `message` | string | 否 | 附加消息（最多200字符） |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440030",
    "sender_id": "550e8400-e29b-41d4-a716-446655440001",
    "receiver_id": "550e8400-e29b-41d4-a716-446655440002",
    "message": "你好，我想加你为好友",
    "status": "pending",
    "created_at": "2024-01-20T10:00:00Z"
  }
}
```

**失败 - 已经是好友 (409 Conflict)**

```json
{
  "success": false,
  "code": "CONFLICT",
  "error": "资源已存在",
  "message": "已经是好友"
}
```

**失败 - 请求已存在 (409 Conflict)**

```json
{
  "success": false,
  "code": "CONFLICT",
  "error": "资源已存在",
  "message": "已存在待处理的好友请求"
}
```

---

## 获取收到的好友请求

### 请求

```http
GET /api/v1/users/friends/requests/received
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440030",
      "sender": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "username": "requester",
        "avatar_url": null
      },
      "message": "你好，我想加你为好友",
      "status": "pending",
      "created_at": "2024-01-20T10:00:00Z"
    }
  ]
}
```

---

## 获取发送的好友请求

### 请求

```http
GET /api/v1/users/friends/requests/sent
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440030",
      "receiver": {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "username": "target_user",
        "avatar_url": null
      },
      "message": "你好，我想加你为好友",
      "status": "pending",
      "created_at": "2024-01-20T10:00:00Z"
    }
  ]
}
```

---

## 处理好友请求

### 请求

```http
POST /api/v1/users/friends/requests/handle
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440030",
  "action": "accept"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `request_id` | string (UUID) | 是 | 好友请求ID |
| `action` | string | 是 | 操作：`accept`（接受）或 `reject`（拒绝） |

### 响应

**成功 - 接受 (200 OK)**

```json
{
  "success": true,
  "data": "好友请求已接受"
}
```

**成功 - 拒绝 (200 OK)**

```json
{
  "success": true,
  "data": "好友请求已拒绝"
}
```

**失败 - 请求不存在 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源不存在",
  "message": "好友请求不存在"
}
```

---

## 取消好友请求

### 请求

```http
DELETE /api/v1/users/friends/requests/{request_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `request_id` | string (UUID) | 好友请求ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "好友请求已取消"
}
```

### 说明

- 只能取消自己发送的待处理请求
- 已接受或已拒绝的请求不能取消

---

## 删除好友

### 请求

```http
DELETE /api/v1/users/friends/{user_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `user_id` | string (UUID) | 好友用户ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "好友已删除"
}
```

**失败 - 不是好友 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源不存在",
  "message": "不是好友关系"
}
```

---

*文档版本: 1.2.0*  
*最后更新: 2026-05-07*
