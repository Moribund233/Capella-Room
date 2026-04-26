# 认证接口文档

> **API 前缀**: `/api/v1`
> **认证要求**: 所有接口均无需认证

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/auth/register` | 用户注册 |
| POST | `/api/v1/auth/login` | 用户登录 |
| POST | `/api/v1/auth/refresh` | 刷新 Token |

---

## 用户注册

### 请求

```http
POST /api/v1/auth/register
Content-Type: application/json
```

### 请求体

```json
{
  "username": "testuser",
  "email": "test@example.com",
  "password": "SecurePass123!"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `username` | string | 是 | 用户名，3-20 个字符，只能包含字母、数字、下划线 |
| `email` | string | 是 | 邮箱地址，需符合邮箱格式 |
| `password` | string | 是 | 密码，至少 8 位，需包含大小写字母和数字 |

### 响应

**成功 (201 Created)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "avatar_url": null,
    "status": "offline",
    "role": "user",
    "created_at": "2024-01-15T08:30:00Z"
  }
}
```

**失败 - 邮箱已存在 (409 Conflict)**

```json
{
  "success": false,
  "code": "CONFLICT",
  "error": "资源已存在",
  "message": "资源已存在: 邮箱已被注册"
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
  "message": "验证失败: username: 用户名格式不正确"
}
```

### 响应字段说明

#### 成功响应

| 字段 | 类型 | 说明 |
|------|------|------|
| `success` | boolean | 操作是否成功，固定为 `true` |
| `data` | object | 用户数据对象 |
| `data.id` | string (UUID) | 用户唯一标识 |
| `data.username` | string | 用户名 |
| `data.email` | string | 邮箱地址 |
| `data.avatar_url` | string \| null | 头像 URL |
| `data.status` | string | 用户状态：`online` / `offline` / `away` / `disabled` |
| `data.role` | string | 用户角色：`user` / `admin` / `super_admin` |
| `data.created_at` | string (ISO 8601) | 创建时间 |

#### 错误响应

| 字段 | 类型 | 说明 |
|------|------|------|
| `success` | boolean | 操作是否成功，固定为 `false` |
| `code` | string | 错误代码，用于程序判断错误类型 |
| `error` | string | 错误类型描述 |
| `message` | string | 详细的错误信息，用于展示给用户 |

### 说明

- 注册成功后，用户状态默认为 `offline`
- 用户角色默认为 `user`
- 注册时会自动记录审计日志
- 密码会使用 Argon2 算法进行哈希存储

---

## 用户登录

### 请求

```http
POST /api/v1/auth/login
Content-Type: application/json
```

### 请求体

```json
{
  "email": "test@example.com",
  "password": "SecurePass123!"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `email` | string | 是 | 注册时的邮箱地址 |
| `password` | string | 是 | 用户密码 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 900,
    "token_type": "Bearer",
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "testuser",
      "email": "test@example.com",
      "avatar_url": null,
      "status": "online",
      "role": "user",
      "created_at": "2024-01-15T08:30:00Z"
    }
  }
}
```

**失败 - 邮箱或密码错误 (401 Unauthorized)**

```json
{
  "success": false,
  "code": "AUTH_ERROR",
  "error": "认证失败",
  "message": "认证失败: 邮箱或密码错误"
}
```

**失败 - 账号被禁用 (401 Unauthorized)**

```json
{
  "success": false,
  "code": "AUTH_ERROR",
  "error": "认证失败",
  "message": "认证失败: 账号已被禁用，请联系管理员"
}
```

### 响应字段说明

#### Token 信息

| 字段 | 类型 | 说明 |
|------|------|------|
| `access_token` | string | 访问令牌，用于 API 认证 |
| `refresh_token` | string | 刷新令牌，用于获取新的 Access Token |
| `expires_in` | number | Access Token 有效期（秒），默认 900 秒（15 分钟） |
| `token_type` | string | Token 类型，固定为 `Bearer` |

#### 用户信息

| 字段 | 类型 | 说明 |
|------|------|------|
| `user.id` | string (UUID) | 用户唯一标识 |
| `user.username` | string | 用户名 |
| `user.email` | string | 邮箱地址 |
| `user.avatar_url` | string \| null | 头像 URL |
| `user.status` | string | 登录后状态为 `online` |
| `user.role` | string | 用户角色 |
| `user.created_at` | string (ISO 8601) | 创建时间 |

### 说明

- 登录成功后，用户状态会自动更新为 `online`
- 登录失败会记录审计日志（包含失败原因）
- 登录成功也会记录审计日志
- Access Token 有效期为 15 分钟，Refresh Token 有效期为 7 天

---

## 刷新 Token

### 请求

```http
POST /api/v1/auth/refresh
Content-Type: application/json
```

### 请求体

```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `refresh_token` | string | 是 | 登录时获取的 Refresh Token |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 900,
    "token_type": "Bearer"
  }
}
```

**失败 - Token 无效 (401 Unauthorized)**

```json
{
  "success": false,
  "code": "AUTH_ERROR",
  "error": "认证失败",
  "message": "认证失败: Token验证失败"
}
```

**失败 - Token 已过期 (401 Unauthorized)**

```json
{
  "success": false,
  "code": "AUTH_ERROR",
  "error": "认证失败",
  "message": "认证失败: Token已过期"
}
```

**失败 - 用户不存在 (401 Unauthorized)**

```json
{
  "success": false,
  "code": "AUTH_ERROR",
  "error": "认证失败",
  "message": "认证失败: 用户不存在"
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `access_token` | string | 新的访问令牌 |
| `refresh_token` | string | 新的刷新令牌（刷新轮换策略） |
| `expires_in` | number | Access Token 有效期（秒） |
| `token_type` | string | Token 类型，固定为 `Bearer` |

### 说明

- 刷新 Token 会同时生成新的 Access Token 和新的 Refresh Token
- 旧的 Refresh Token 会立即失效（刷新轮换策略）
- 如果 Refresh Token 过期，需要重新登录
- 建议在 Access Token 过期前 1-2 分钟调用此接口刷新

---

## 使用示例

### cURL 示例

```bash
# 用户注册
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'

# 用户登录
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'

# 刷新 Token
curl -X POST http://localhost:3000/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }'
```

### JavaScript 示例

```javascript
// 用户注册
async function register(username, email, password) {
  const response = await fetch('http://localhost:3000/api/v1/auth/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, email, password })
  });
  
  const data = await response.json();
  
  if (data.success) {
    console.log('注册成功:', data.data);
    return data.data;
  } else {
    console.error('注册失败:', data.message);
    throw new Error(data.message);
  }
}

// 用户登录
async function login(email, password) {
  const response = await fetch('http://localhost:3000/api/v1/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password })
  });
  
  const data = await response.json();
  
  if (data.success) {
    // 保存 Token
    localStorage.setItem('access_token', data.data.access_token);
    localStorage.setItem('refresh_token', data.data.refresh_token);
    localStorage.setItem('expires_at', Date.now() + data.data.expires_in * 1000);
    
    console.log('登录成功:', data.data.user);
    return data.data;
  } else {
    console.error('登录失败:', data.message);
    throw new Error(data.message);
  }
}

// 刷新 Token
async function refreshToken() {
  const refreshToken = localStorage.getItem('refresh_token');
  
  if (!refreshToken) {
    throw new Error('No refresh token available');
  }
  
  const response = await fetch('http://localhost:3000/api/v1/auth/refresh', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refresh_token: refreshToken })
  });
  
  const data = await response.json();
  
  if (data.success) {
    // 更新 Token
    localStorage.setItem('access_token', data.data.access_token);
    localStorage.setItem('refresh_token', data.data.refresh_token);
    localStorage.setItem('expires_at', Date.now() + data.data.expires_in * 1000);
    
    return data.data;
  } else {
    // 刷新失败，需要重新登录
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    throw new Error(data.message);
  }
}

// 自动刷新 Token 的包装函数
async function fetchWithAuth(url, options = {}) {
  const expiresAt = localStorage.getItem('expires_at');
  
  // 如果 Token 将在 60 秒内过期，先刷新
  if (expiresAt && Date.now() > expiresAt - 60000) {
    await refreshToken();
  }
  
  const token = localStorage.getItem('access_token');
  
  options.headers = {
    ...options.headers,
    'Authorization': `Bearer ${token}`
  };
  
  return fetch(url, options);
}
```

---

## 认证流程图

```
┌─────────────┐     注册      ┌─────────────┐
│   新用户    │ ────────────> │  /register  │
└─────────────┘               └─────────────┘
                                    │
                                    ▼
                            ┌─────────────┐
                            │  返回用户信息  │
                            └─────────────┘

┌─────────────┐     登录      ┌─────────────┐
│   已注册    │ ────────────> │   /login    │
└─────────────┘               └─────────────┘
                                    │
                                    ▼
                    ┌───────────────────────────┐
                    │  返回 Access + Refresh    │
                    │  Token 和用户信息         │
                    └───────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
            ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
            │ 访问受保护API  │  │ Token过期   │  │  退出登录   │
            │ (带Access)   │  │             │  │             │
            └─────────────┘  └─────────────┘  └─────────────┘
                                    │
                                    ▼
                           ┌─────────────┐
                           │   /refresh   │
                           └─────────────┘
                                    │
                                    ▼
                    ┌───────────────────────────┐
                    │  返回新的 Token 对        │
                    └───────────────────────────┘
```

---

## 安全建议

1. **Token 存储**
   - Access Token: 可以存储在内存中或 sessionStorage
   - Refresh Token: 建议存储在 httpOnly cookie 或 secure storage

2. **Token 刷新时机**
   - 在 Access Token 过期前 1-2 分钟刷新
   - 收到 401 响应时尝试刷新
   - 刷新失败时引导用户重新登录

3. **密码安全**
   - 前端不要限制密码复杂度提示（由后端验证）
   - 使用 HTTPS 传输密码
   - 不要在日志中记录密码

4. **并发刷新**
   - 防止多个请求同时触发刷新
   - 使用队列或锁机制确保只有一个刷新请求

---

## 错误码汇总

### HTTP 状态码

| HTTP 状态码 | 错误场景 | 说明 |
|------------|---------|------|
| 200 | 请求成功 | 注册/登录/刷新成功 |
| 400 | 请求参数错误 | 参数验证失败、JSON 解析错误 |
| 401 | 认证失败 | 邮箱/密码错误、Token 无效或过期 |
| 409 | 资源冲突 | 邮箱或用户名已存在 |
| 500 | 服务器错误 | 内部服务器错误 |

### 业务错误码 (code)

| 错误码 | HTTP 状态码 | 说明 | 处理建议 |
|--------|------------|------|---------|
| `VALIDATION_ERROR` | 400 | 参数验证失败 | 检查请求参数是否符合要求 |
| `AUTH_ERROR` | 401 | 认证失败 | 检查邮箱密码是否正确，或重新登录 |
| `CONFLICT` | 409 | 资源冲突 | 更换邮箱或用户名后重试 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 | 稍后重试或联系管理员 |

### 常见错误处理示例

```javascript
async function handleAuthError(response) {
  const data = await response.json();
  
  if (!data.success) {
    switch (data.code) {
      case 'VALIDATION_ERROR':
        // 显示验证错误信息
        console.error('输入错误:', data.message);
        // 可以解析 message 中的具体字段错误
        // 如: "验证失败: username: 用户名格式不正确"
        break;
        
      case 'AUTH_ERROR':
        // 认证失败，清除本地 Token 并跳转登录页
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        window.location.href = '/login';
        break;
        
      case 'CONFLICT':
        // 资源冲突，提示用户更换信息
        if (data.message.includes('邮箱')) {
          console.error('该邮箱已被注册');
        } else if (data.message.includes('用户名')) {
          console.error('该用户名已被使用');
        }
        break;
        
      default:
        console.error('未知错误:', data.message);
    }
    
    throw new Error(data.message);
  }
}
```

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
