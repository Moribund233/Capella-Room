# 消息接口文档

> **认证要求**: 所有接口均需要认证（需要携带 Access Token）
> 
> **注意**: 发送消息通过 WebSocket 实时通信，HTTP API 仅用于消息管理（搜索、编辑、删除、获取历史）

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/rooms/:room_id/messages` | 获取房间消息历史 |
| GET | `/api/v1/messages/search` | 搜索消息 |
| PUT | `/api/v1/messages/:message_id` | 编辑消息 |
| DELETE | `/api/v1/messages/:message_id` | 删除消息 |
| GET | `/api/v1/messages/:message_id/history` | 获取消息编辑历史 |
| POST | `/api/v1/messages/:message_id/reactions` | 添加表情反应 |
| DELETE | `/api/v1/messages/:message_id/reactions?emoji=xxx` | 移除表情反应 |
| GET | `/api/v1/messages/:message_id/reactions` | 获取消息的反应列表 |
| POST | `/api/v1/messages/:message_id/pin` | 置顶消息 |
| DELETE | `/api/v1/messages/:message_id/pin` | 取消置顶消息 |
| GET | `/api/v1/rooms/:room_id/pinned-messages` | 获取房间置顶消息列表 |

---

## 获取房间消息历史

获取指定聊天室的消息历史记录，支持游标分页。

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
| `before` | string (UUID) | 否 | 游标，获取此消息 ID 之前的历史消息 |

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
        "reply_to_message": null,
        "is_deleted": false,
        "created_at": "2024-01-15T08:30:00Z",
        "edit_count": 0,
        "edited_at": null,
        "reactions": null
      },
      {
        "id": "660e8400-e29b-41d4-a716-446655440003",
        "room_id": "550e8400-e29b-41d4-a716-446655440001",
        "sender": {
          "id": "550e8400-e29b-41d4-a716-446655440004",
          "username": "user2",
          "avatar_url": "https://example.com/avatar.jpg"
        },
        "content": "欢迎新人！",
        "message_type": "text",
        "reply_to": "550e8400-e29b-41d4-a716-446655440000",
        "reply_to_message": {
          "id": "550e8400-e29b-41d4-a716-446655440000",
          "sender": {
            "id": "550e8400-e29b-41d4-a716-446655440002",
            "username": "user1",
            "avatar_url": null
          },
          "content": "大家好！",
          "created_at": "2024-01-15T08:30:00Z"
        },
        "is_deleted": false,
        "created_at": "2024-01-15T08:31:00Z",
        "edit_count": 1,
        "edited_at": "2024-01-15T08:35:00Z",
        "reactions": [
          {
            "emoji": "👍",
            "count": 2,
            "users": ["550e8400-...", "660e8400-..."]
          }
        ]
      }
    ],
    "total": 2,
    "has_more": true
  }
}
```

### 响应字段说明

#### 分页信息

| 字段 | 类型 | 说明 |
|------|------|------|
| `messages` | array | 消息列表，按时间倒序排列（最新的在前） |
| `total` | number | 本次返回的消息数量 |
| `has_more` | boolean | 是否还有更多历史消息 |

#### 消息对象

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 消息唯一标识 |
| `room_id` | string (UUID) | 所属聊天室 ID |
| `sender` | object | 发送者信息 |
| `sender.id` | string (UUID) | 发送者用户 ID |
| `sender.username` | string | 发送者用户名 |
| `sender.avatar_url` | string \| null | 发送者头像 URL |
| `content` | string | 消息内容 |
| `message_type` | string | 消息类型：`text` / `image` / `file` / `system` |
| `reply_to` | string \| null | 回复的消息 ID |
| `reply_to_message` | object \| null | 被回复消息的详细信息 |
| `is_deleted` | boolean | 是否已删除 |
| `created_at` | string (ISO 8601) | 发送时间 |
| `edit_count` | number | 编辑次数 |
| `edited_at` | string \| null | 最后编辑时间 |
| `reactions` | array \| null | 表情反应汇总（`{emoji, count, users}`） |

#### 被回复消息对象 (reply_to_message)

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 被回复消息 ID |
| `sender` | object | 原消息发送者信息 |
| `content` | string | 原消息内容 |
| `created_at` | string (ISO 8601) | 原消息发送时间 |

### 说明

- 消息按时间倒序排列，最新的消息排在最前面
- 使用游标分页（`before` 参数）避免消息重复或遗漏
- 只返回未删除的消息（`is_deleted: false`）
- 回复消息会包含被回复消息的完整信息

### 分页加载示例

```javascript
// 首次加载
const firstPage = await getRoomMessages(roomId, 50);

// 加载更多（使用最后一条消息的 ID 作为游标）
if (firstPage.has_more) {
  const lastMessageId = firstPage.messages[firstPage.messages.length - 1].id;
  const nextPage = await getRoomMessages(roomId, 50, lastMessageId);
}
```

---

## 搜索消息

在指定聊天室或全局搜索消息内容。

### 请求

```http
GET /api/v1/messages/search?q={keyword}&room_id={room_id}&limit={limit}
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `q` | string | 是 | 搜索关键词，1-100 个字符 |
| `room_id` | string (UUID) | 否 | 限定在某个聊天室搜索，不传则全局搜索 |
| `limit` | number | 否 | 结果数量限制，默认 50，最大 100 |

### 响应

**成功 (200 OK)**

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "sender": {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "username": "user1",
      "avatar_url": null
    },
    "content": "大家好，欢迎来到技术交流群！",
    "message_type": "text",
    "reply_to": null,
    "reply_to_message": null,
    "is_deleted": false,
    "created_at": "2024-01-15T08:30:00Z",
    "edit_count": 0,
    "edited_at": null,
    "reactions": null
  }
]
```

**失败 - 搜索关键词为空 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "验证失败: q: 长度必须在 1-100 之间"
}
```

### 说明

- 支持模糊搜索，不区分大小写
- 只搜索未删除的消息
- 搜索结果按时间倒序排列
- 使用 PostgreSQL 全文搜索优化性能

---

## 编辑消息

编辑已发送的消息内容，会记录编辑历史。

### 请求

```http
PUT /api/v1/messages/{message_id}
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `message_id` | string (UUID) | 消息唯一标识 |

### 请求体

```json
{
  "content": "编辑后的消息内容"
}
```

### 请求字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `content` | string | 是 | 新的消息内容，1-2000 个字符 |

### 响应

**成功 (200 OK)**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "room_id": "550e8400-e29b-41d4-a716-446655440001",
  "sender": {
    "id": "550e8400-e29b-41d4-a716-446655440002",
    "username": "user1",
    "avatar_url": null
  },
  "content": "编辑后的消息内容",
  "message_type": "text",
  "reply_to": null,
  "reply_to_message": null,
  "is_deleted": false,
  "created_at": "2024-01-15T08:30:00Z",
  "edit_count": 1,
  "edited_at": "2024-01-15T08:35:00Z"
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

**失败 - 系统消息不能编辑 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "系统消息不能编辑"
}
```

**失败 - 消息不存在 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源未找到",
  "message": "未找到资源"
}
```

### 说明

- **只能编辑自己发送的消息**，即使管理员也不能通过此接口编辑他人消息
- 如需编辑他人消息，管理员应使用 `DELETE /api/v1/admin/messages/:message_id` 删除后重新发送
- 系统消息（`message_type: system`）不能编辑
- 每次编辑会增加 `edit_count` 计数
- 会记录编辑历史，可通过 `GET /api/v1/messages/{message_id}/history` 查看
- 编辑后会通过 WebSocket 广播 `MessageEdited` 事件

---

## 删除消息

删除已发送的消息（软删除）。

### 请求

```http
DELETE /api/v1/messages/{message_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `message_id` | string (UUID) | 消息唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "message": "消息已删除"
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

**失败 - 消息不存在 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源未找到",
  "message": "未找到资源"
}
```

### 说明

- **只能删除自己发送的消息**，管理员不能通过此接口删除他人消息
- 管理员如需删除他人消息，应使用管理员接口 `DELETE /api/v1/admin/messages/:message_id`
- 删除是软删除（`is_deleted` 标记为 `true`）
- 删除后会通过 WebSocket 广播 `MessageDeleted` 事件
- 已删除的消息在历史记录中显示为"[已删除]"

---

## 获取消息编辑历史

获取指定消息的编辑历史记录。

### 请求

```http
GET /api/v1/messages/{message_id}/history?limit={limit}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `message_id` | string (UUID) | 消息唯一标识 |

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `limit` | number | 否 | 结果数量限制，默认 20，最大 50 |

### 响应

**成功 (200 OK)**

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440005",
    "message_id": "550e8400-e29b-41d4-a716-446655440000",
    "editor": {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "username": "user1",
      "avatar_url": null
    },
    "old_content": "原始消息内容",
    "new_content": "第一次编辑后的内容",
    "created_at": "2024-01-15T08:35:00Z"
  },
  {
    "id": "660e8400-e29b-41d4-a716-446655440006",
    "message_id": "550e8400-e29b-41d4-a716-446655440000",
    "editor": {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "username": "user1",
      "avatar_url": null
    },
    "old_content": "第一次编辑后的内容",
    "new_content": "第二次编辑后的内容",
    "created_at": "2024-01-15T08:40:00Z"
  }
]
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 编辑记录唯一标识 |
| `message_id` | string (UUID) | 所属消息 ID |
| `editor` | object | 编辑者信息 |
| `editor.id` | string (UUID) | 编辑者用户 ID |
| `editor.username` | string | 编辑者用户名 |
| `editor.avatar_url` | string \| null | 编辑者头像 URL |
| `old_content` | string | 编辑前的内容 |
| `new_content` | string | 编辑后的内容 |
| `created_at` | string (ISO 8601) | 编辑时间 |

### 说明

- 编辑历史按时间正序排列（最早的编辑在前）
- 只有消息发送者和管理员可以查看编辑历史
- 每次编辑都会生成一条历史记录

---

## 使用示例

### cURL 示例

```bash
# 获取房间消息历史
curl -X GET "http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440001/messages?limit=50" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 加载更多历史消息（使用游标）
curl -X GET "http://localhost:3000/api/v1/rooms/550e8400-e29b-41d4-a716-446655440001/messages?limit=50&before=550e8400-e29b-41d4-a716-446655440000" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 搜索消息
curl -X GET "http://localhost:3000/api/v1/messages/search?q=欢迎&limit=20" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 在指定房间搜索消息
curl -X GET "http://localhost:3000/api/v1/messages/search?q=技术&room_id=550e8400-e29b-41d4-a716-446655440001&limit=20" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 编辑消息
curl -X PUT http://localhost:3000/api/v1/messages/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "content": "编辑后的消息内容"
  }'

# 删除消息
curl -X DELETE http://localhost:3000/api/v1/messages/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 获取消息编辑历史
curl -X GET "http://localhost:3000/api/v1/messages/550e8400-e29b-41d4-a716-446655440000/history?limit=10" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### JavaScript 示例

```javascript
// 获取房间消息历史
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

// 搜索消息
async function searchMessages(keyword, roomId = null, limit = 50) {
  const params = new URLSearchParams();
  params.append('q', keyword);
  params.append('limit', limit);
  if (roomId) params.append('room_id', roomId);
  
  const response = await fetch(
    `http://localhost:3000/api/v1/messages/search?${params}`,
    {
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  
  return await response.json();
}

// 编辑消息
async function editMessage(messageId, newContent) {
  const response = await fetch(
    `http://localhost:3000/api/v1/messages/${messageId}`,
    {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ content: newContent })
    }
  );
  
  return await response.json();
}

// 删除消息
async function deleteMessage(messageId) {
  const response = await fetch(
    `http://localhost:3000/api/v1/messages/${messageId}`,
    {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  
  const data = await response.json();
  
  if (data.success) {
    return data;
  } else {
    throw new Error(data.message);
  }
}

// 获取消息编辑历史
async function getMessageEditHistory(messageId, limit = 20) {
  const response = await fetch(
    `http://localhost:3000/api/v1/messages/${messageId}/history?limit=${limit}`,
    {
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  
  return await response.json();
}

// 无限滚动加载消息示例
class MessageLoader {
  constructor(roomId) {
    this.roomId = roomId;
    this.messages = [];
    this.hasMore = true;
    this.loading = false;
  }
  
  async loadMore() {
    if (this.loading || !this.hasMore) return;
    
    this.loading = true;
    
    try {
      const before = this.messages.length > 0 
        ? this.messages[this.messages.length - 1].id 
        : null;
      
      const result = await getRoomMessages(this.roomId, 50, before);
      
      this.messages.push(...result.messages);
      this.hasMore = result.has_more;
      
      return result.messages;
    } finally {
      this.loading = false;
    }
  }
}
```

---

## 消息类型说明

### 消息类型 (message_type)

| 类型 | 说明 | 内容格式 |
|------|------|---------|
| `text` | 文本消息 | 纯文本内容 |
| `image` | 图片消息 | 图片 URL |
| `file` | 文件消息 | 文件 URL 和元数据 |
| `system` | 系统消息 | 系统通知内容 |

### 系统消息场景

系统消息由系统自动生成，包括：

- 用户加入/离开房间
- 用户被踢出房间
- 房间信息更新
- 成员角色变更

---

## 错误码汇总

### HTTP 状态码

| HTTP 状态码 | 错误场景 | 说明 |
|------------|---------|------|
| 200 | 请求成功 | 操作成功 |
| 400 | 请求参数错误 | 参数验证失败 |
| 401 | 认证失败 | Token 无效或过期 |
| 403 | 权限不足 | 不是消息发送者 |
| 404 | 资源不存在 | 消息不存在 |
| 500 | 服务器错误 | 内部服务器错误 |

### 业务错误码 (code)

| 错误码 | HTTP 状态码 | 说明 | 处理建议 |
|--------|------------|------|---------|
| `VALIDATION_ERROR` | 400 | 参数验证失败 | 检查请求参数是否符合要求 |
| `AUTH_ERROR` | 401 | 认证失败 | 检查 Token 是否过期 |
| `FORBIDDEN` | 403 | 权限不足 | 确认用户是否有操作权限 |
| `NOT_FOUND` | 404 | 资源不存在 | 检查消息 ID 是否正确 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 | 稍后重试或联系管理员 |

---

## 与 WebSocket 的关系

### HTTP API vs WebSocket

| 功能 | HTTP API | WebSocket |
|------|---------|-----------|
| **发送消息** | ❌ 不支持 | ✅ `ChatMessage` |
| **接收实时消息** | ❌ 不支持 | ✅ `NewMessage` |
| **获取历史消息** | ✅ `GET /rooms/{id}/messages` | ❌ 不支持 |
| **搜索消息** | ✅ `GET /messages/search` | ❌ 不支持 |
| **编辑消息** | ✅ `PUT /messages/{id}` | ✅ `EditMessage` |
| **删除消息** | ✅ `DELETE /messages/{id}` | ✅ `DeleteMessage` |
| **获取编辑历史** | ✅ `GET /messages/{id}/history` | ❌ 不支持 |
| **添加反应** | ✅ `POST /messages/{id}/reactions` | ✅ `AddReaction` |
| **移除反应** | ✅ `DELETE /messages/{id}/reactions` | ✅ `RemoveReaction` |
| **获取反应** | ✅ `GET /messages/{id}/reactions` | ❌ 不支持 |

### 推荐用法

1. **首次加载**: 使用 HTTP API 获取最近的消息历史
2. **实时通信**: 使用 WebSocket 接收新消息和发送消息
3. **加载更多**: 使用 HTTP API 分页加载历史消息
4. **搜索**: 使用 HTTP API 搜索消息
5. **管理操作**: 编辑、删除可通过 HTTP 或 WebSocket
6. **表情反应**: 可通过 HTTP 或 WebSocket 添加/移除

---

## 消息置顶

消息置顶功能允许将重要消息置顶到房间顶部，方便成员快速查看。

### 接口

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v1/messages/:message_id/pin` | 置顶消息 | 需要 |
| DELETE | `/api/v1/messages/:message_id/pin` | 取消置顶 | 需要 |
| GET | `/api/v1/rooms/:room_id/pinned-messages` | 获取置顶消息列表 | 需要 |

### 置顶消息

#### 请求

```http
POST /api/v1/messages/{message_id}/pin
Authorization: Bearer {access_token}
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "770e8400-e29b-41d4-a716-446655440003",
    "message_id": "660e8400-e29b-41d4-a716-446655440001",
    "room_id": "550e8400-e29b-41d4-a716-446655440000",
    "pinned_by": "44777268-d040-4ef5-81de-9aad6ea3ead3",
    "created_at": "2026-06-14T10:00:00.000Z"
  }
}
```

**失败 - 消息已被置顶 (409 Conflict)**

```json
{
  "success": false,
  "code": "CONFLICT",
  "error": "资源冲突",
  "message": "消息已被置顶"
}
```

### 取消置顶

#### 请求

```http
DELETE /api/v1/messages/{message_id}/pin
Authorization: Bearer {access_token}
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": "置顶已取消"
}
```

### 获取置顶消息列表

#### 请求

```http
GET /api/v1/rooms/{room_id}/pinned-messages
Authorization: Bearer {access_token}
```

#### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440003",
      "message_id": "660e8400-e29b-41d4-a716-446655440001",
      "room_id": "550e8400-e29b-41d4-a716-446655440000",
      "pinned_by": "44777268-d040-4ef5-81de-9aad6ea3ead3",
      "content": "这是重要通知",
      "sender_name": "user123",
      "created_at": "2026-06-14T10:00:00.000Z"
    }
  ]
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| id | string (UUID) | 置顶记录 ID |
| message_id | string (UUID) | 被置顶的消息 ID |
| room_id | string (UUID) | 房间 ID |
| pinned_by | string (UUID) | 置顶操作者 ID |
| content | string | 消息内容 |
| sender_name | string | 消息发送者名称 |
| created_at | string (ISO 8601) | 置顶时间 |

### 功能特性

- 已置顶的消息再次置顶会返回冲突（409）
- 取消置顶不存在的记录会返回 404
- 置顶消息列表按置顶时间倒序排列
- 已删除的消息不会出现在置顶列表中
- 置顶/取消置顶操作会通过 WebSocket 实时广播给房间所有成员（`MessagePinned` / `MessageUnpinned`）

---

*文档版本: 1.2.0*  
*最后更新: 2026-06-14*
