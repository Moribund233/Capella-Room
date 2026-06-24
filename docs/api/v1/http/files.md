# 文件接口文档

> **认证要求**: 所有接口均需要认证（需要携带 Access Token）
> 
> **文件上传限制**: 单文件最大 10MB（可通过配置调整）

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/files` | 获取文件列表 |
| GET | `/api/v1/files/:file_id` | 获取文件详情 |
| DELETE | `/api/v1/files/:file_id` | 删除文件 |
| POST | `/api/v1/upload` | 通用文件上传 |
| POST | `/api/v1/upload/image` | 上传图片 |
| POST | `/api/v1/upload/avatar` | 上传头像 |
| POST | `/api/v1/upload/chunked/init` | 初始化分片上传会话 |
| POST | `/api/v1/upload/chunked/:session_id/:chunk_index` | 上传单个分片 |
| GET | `/api/v1/upload/chunked/:session_id/status` | 查询分片上传状态 |
| POST | `/api/v1/upload/chunked/:session_id/complete` | 完成分片上传 |
| DELETE | `/api/v1/upload/chunked/:session_id` | 取消分片上传 |

---

## 文件分类

### 文件类型 (category)

| 类型 | 说明 | 存储目录 |
|------|------|---------|
| `image` | 图片文件 | `images/` |
| `document` | 文档文件 | `documents/` |
| `video` | 视频文件 | `videos/` |
| `audio` | 音频文件 | `audio/` |
| `other` | 其他文件 | `other/` |

### 文件用途 (usage_type)

| 用途 | 说明 | 使用场景 |
|------|------|---------|
| `avatar` | 头像 | 用户头像上传 |
| `message` | 消息附件 | 聊天消息中的图片/文件 |
| `room_cover` | 房间封面 | 聊天室封面图片 |
| `general` | 通用 | 其他用途 |

### 支持的 MIME 类型

#### 图片类型
- `image/jpeg` - JPEG 图片
- `image/png` - PNG 图片
- `image/gif` - GIF 图片
- `image/webp` - WebP 图片
- `image/svg+xml` - SVG 矢量图

#### 文档类型
- `application/pdf` - PDF 文档
- `application/msword` / `application/vnd.openxmlformats-officedocument.wordprocessingml.document` - Word 文档
- `application/vnd.ms-excel` / `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` - Excel 表格
- `application/vnd.ms-powerpoint` / `application/vnd.openxmlformats-officedocument.presentationml.presentation` - PPT 演示文稿
- `text/plain` - 纯文本
- `text/markdown` - Markdown 文档

#### 视频类型
- `video/mp4` - MP4 视频
- `video/webm` - WebM 视频
- `video/ogg` - Ogg 视频
- `video/quicktime` - QuickTime 视频

#### 音频类型
- `audio/mpeg` - MP3 音频
- `audio/ogg` - Ogg 音频
- `audio/wav` - WAV 音频
- `audio/webm` - WebM 音频
- `audio/aac` - AAC 音频

---

## 获取文件列表

获取当前用户上传的文件列表。

### 请求

```http
GET /api/v1/files?category={category}&usage_type={usage_type}&limit={limit}&offset={offset}
Authorization: Bearer {access_token}
```

### 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `category` | string | 否 | 文件分类筛选：`image` / `document` / `video` / `audio` / `other` |
| `usage_type` | string | 否 | 用途筛选：`avatar` / `message` / `room_cover` / `general` |
| `limit` | number | 否 | 每页数量，默认 20 |
| `offset` | number | 否 | 偏移量，默认 0 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "files": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "original_name": "screenshot.png",
        "file_url": "/uploads/images/550e8400-e29b-41d4-a716-446655440000.png",
        "file_size": 1024567,
        "mime_type": "image/png",
        "category": "image",
        "usage_type": "message",
        "uploader": {
          "id": "550e8400-e29b-41d4-a716-446655440001",
          "username": "user1",
          "avatar_url": null
        },
        "created_at": "2024-01-15T08:30:00Z"
      },
      {
        "id": "660e8400-e29b-41d4-a716-446655440002",
        "original_name": "document.pdf",
        "file_url": "/uploads/documents/660e8400-e29b-41d4-a716-446655440002.pdf",
        "file_size": 2048000,
        "mime_type": "application/pdf",
        "category": "document",
        "usage_type": "general",
        "uploader": null,
        "created_at": "2024-01-15T09:15:00Z"
      }
    ],
    "total": 2
  }
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `files` | array | 文件列表 |
| `total` | number | 总文件数量 |

#### 文件对象

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 文件唯一标识 |
| `original_name` | string | 原始文件名 |
| `file_url` | string | 文件访问 URL |
| `file_size` | number | 文件大小（字节） |
| `mime_type` | string | MIME 类型 |
| `category` | string | 文件分类 |
| `usage_type` | string | 文件用途 |
| `uploader` | object \| null | 上传者信息 |
| `created_at` | string (ISO 8601) | 上传时间 |

---

## 获取文件详情

获取指定文件的详细信息。

### 请求

```http
GET /api/v1/files/{file_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `file_id` | string (UUID) | 文件唯一标识 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "original_name": "screenshot.png",
    "file_url": "/uploads/images/550e8400-e29b-41d4-a716-446655440000.png",
    "file_size": 1024567,
    "mime_type": "image/png",
    "category": "image",
    "usage_type": "message",
    "uploader": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "username": "user1",
      "avatar_url": null
    },
    "created_at": "2024-01-15T08:30:00Z"
  }
}
```

**失败 - 文件不存在 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源未找到",
  "message": "未找到资源"
}
```

---

## 删除文件

删除指定的文件。

### 请求

```http
DELETE /api/v1/files/{file_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `file_id` | string (UUID) | 文件唯一标识 |

### 响应

**成功 (204 No Content)**

无响应体。

**失败 - 无权限 (403 Forbidden)**

```json
{
  "success": false,
  "code": "FORBIDDEN",
  "error": "权限不足",
  "message": "权限不足: 您没有权限执行此操作"
}
```

**失败 - 文件不存在 (404 Not Found)**

```json
{
  "success": false,
  "code": "NOT_FOUND",
  "error": "资源未找到",
  "message": "未找到资源"
}
```

### 说明

- 只能删除自己上传的文件
- 管理员可以删除任何文件
- 删除是软删除（标记删除状态）
- 删除后文件 URL 将返回 404

---

## 通用文件上传

上传任意类型的文件。

### 请求

```http
POST /api/v1/upload
Authorization: Bearer {access_token}
Content-Type: multipart/form-data
```

### 请求体 (multipart/form-data)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file` | file | 是 | 要上传的文件 |
| `usage_type` | string | 否 | 文件用途：`avatar` / `message` / `room_cover` / `general`，默认 `general` |
| `room_id` | string (UUID) | 否 | 关联的聊天室 ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "original_name": "document.pdf",
    "file_url": "/uploads/documents/550e8400-e29b-41d4-a716-446655440000.pdf",
    "file_size": 2048000,
    "mime_type": "application/pdf",
    "category": "document",
    "usage_type": "general"
  }
}
```

**失败 - 文件类型不允许 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "不支持的文件类型"
}
```

**失败 - 文件过大 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "文件大小超过限制"
}
```

**失败 - 缺少文件 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "缺少文件数据"
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string (UUID) | 文件唯一标识 |
| `original_name` | string | 原始文件名 |
| `file_url` | string | 文件访问 URL |
| `file_size` | number | 文件大小（字节） |
| `mime_type` | string | MIME 类型 |
| `category` | string | 文件分类 |
| `usage_type` | string | 文件用途 |

---

## 上传图片

专门用于上传图片文件，会自动验证文件类型为图片。

### 请求

```http
POST /api/v1/upload/image
Authorization: Bearer {access_token}
Content-Type: multipart/form-data
```

### 请求体 (multipart/form-data)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file` | file | 是 | 要上传的图片文件 |
| `room_id` | string (UUID) | 否 | 关联的聊天室 ID |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "original_name": "photo.jpg",
    "file_url": "/uploads/images/550e8400-e29b-41d4-a716-446655440000.jpg",
    "file_size": 512000,
    "mime_type": "image/jpeg",
    "category": "image",
    "usage_type": "message"
  }
}
```

**失败 - 非图片文件 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "只允许上传图片文件"
}
```

### 说明

- 自动验证文件类型为图片（`image/*`）
- `usage_type` 自动设置为 `message`
- 支持压缩和优化（根据配置）

---

## 上传头像

专门用于上传用户头像，上传成功后会自动更新用户头像。

### 请求

```http
POST /api/v1/upload/avatar
Authorization: Bearer {access_token}
Content-Type: multipart/form-data
```

### 请求体 (multipart/form-data)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file` | file | 是 | 要上传的头像图片 |

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "file_url": "/uploads/images/550e8400-e29b-41d4-a716-446655440000.png",
    "message": "头像上传成功"
  }
}
```

**失败 - 非图片文件 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "头像必须是图片文件"
}
```

### 说明

- 自动验证文件类型为图片
- 上传成功后自动更新用户 `avatar_url`
- `usage_type` 自动设置为 `avatar`
- 建议头像尺寸：200x200 像素
- 会自动压缩和裁剪（根据配置）

---

## 分片上传

> **说明**: 分片上传（Chunked Upload）用于大文件上传场景。前端将文件切分为多个分片，逐个上传，最后合并。
>
> **存储**: 分片临时存储在 `{UPLOAD_DIR}/.chunks/{session_id}/` 目录下，合并后自动清理。
>
> **有效期**: 上传会话默认 24 小时后过期（可通过 `upload.session_ttl_hours` 配置）。

### 工作流程

```
1. POST /upload/chunked/init    → 初始化，获取 session_id
2. POST /upload/chunked/:sid/:index  → 逐个上传分片（可并发）
3. GET  /upload/chunked/:sid/status  → 查询已接收分片
4. POST /upload/chunked/:sid/complete → 所有分片完成，服务器合并
5. DELETE /upload/chunked/:sid        → (可选) 取消/清理
```

---

## 初始化分片上传

开始一个新的分片上传会话。

### 请求

```http
POST /api/v1/upload/chunked/init
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "file_name": "large_video.mp4",
  "file_size": 104857600,
  "mime_type": "video/mp4",
  "usage_type": "message",
  "total_chunks": 20
}
```

### 参数说明

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file_name` | string | 是 | 原始文件名 |
| `file_size` | number | 是 | 文件总大小（字节） |
| `mime_type` | string | 是 | MIME 类型 |
| `usage_type` | string | 否 | 文件用途，默认 `general` |
| `total_chunks` | number | 是 | 总分片数 |

### 响应

**成功 (200 OK)**

```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "chunk_size": 5242880,
  "total_chunks": 20
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `session_id` | string (UUID) | 上传会话唯一标识 |
| `chunk_size` | number | 建议分片大小（字节） |
| `total_chunks` | number | 总分片数 |

**失败 - 分片上传未启用 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "分片上传未启用"
}
```

---

## 上传分片

上传单个文件分片。

### 请求

```http
POST /api/v1/upload/chunked/{session_id}/{chunk_index}
Authorization: Bearer {access_token}
Content-Type: multipart/form-data
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `session_id` | string (UUID) | 上传会话 ID |
| `chunk_index` | number | 分片索引（从 0 开始） |

### 请求体 (multipart/form-data)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `chunk` | file | 是 | 分片二进制数据 |

### 响应

**成功 (200 OK)**

```json
{
  "received": 5,
  "total": 20
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `received` | number | 已接收的分片数 |
| `total` | number | 总分片数 |

### 说明

- 支持**幂等上传**：同一分片重复上传不会报错，返回当前进度
- 支持**并发上传**：多个分片可以同时上传
- 分片索引越界返回 400

---

## 查询上传状态

获取当前上传会话的详细信息，包括已接收和缺失的分片。

### 请求

```http
GET /api/v1/upload/chunked/{session_id}/status
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `session_id` | string (UUID) | 上传会话 ID |

### 响应

**成功 (200 OK)**

```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "file_name": "large_video.mp4",
  "file_size": 104857600,
  "mime_type": "video/mp4",
  "status": "active",
  "total_chunks": 20,
  "received_chunks": [0, 1, 2, 3, 4],
  "missing_chunks": [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
  "created_at": "2026-06-24T10:30:00+00:00"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `session_id` | string (UUID) | 上传会话 ID |
| `file_name` | string | 原始文件名 |
| `file_size` | number | 文件总大小 |
| `mime_type` | string | MIME 类型 |
| `status` | string | 会话状态：`active` / `completed` / `cancelled` |
| `total_chunks` | number | 总分片数 |
| `received_chunks` | array[number] | 已接收的分片索引列表 |
| `missing_chunks` | array[number] | 缺失的分片索引列表 |
| `created_at` | string (ISO 8601) | 会话创建时间 |

---

## 完成分片上传

所有分片上传完成后，调用此接口合并分片并保存文件。

### 请求

```http
POST /api/v1/upload/chunked/{session_id}/complete
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `session_id` | string (UUID) | 上传会话 ID |

### 响应

**成功 (200 OK)**

```json
{
  "file": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "original_name": "large_video.mp4",
    "file_url": "/uploads/videos/660e8400-e29b-41d4-a716-446655440001.mp4",
    "file_size": 104857600,
    "mime_type": "video/mp4",
    "category": "video",
    "usage_type": "message"
  },
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `file` | object | 合并后的文件信息（同通用上传响应） |
| `session_id` | string | 原会话 ID |

**失败 - 分片未上传完成 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "分片未上传完成: 5/20"
}
```

### 说明

- 只有所有分片都上传完成后才能合并
- 合并后会清理临时分片文件
- 返回的文件对象与 `POST /upload` 一致，可继续使用文件管理 API

---

## 取消分片上传

取消并清理上传会话。

### 请求

```http
DELETE /api/v1/upload/chunked/{session_id}
Authorization: Bearer {access_token}
```

### 路径参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `session_id` | string (UUID) | 上传会话 ID |

### 响应

**成功 (204 No Content)**

无响应体。

### 说明

- 会删除该会话的所有临时分片文件
- 取消后无法恢复，需要重新初始化

---

## 使用示例

### cURL 示例

```bash
# 获取文件列表
curl -X GET "http://localhost:3000/api/v1/files?limit=20&category=image" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 获取文件详情
curl -X GET "http://localhost:3000/api/v1/files/550e8400-e29b-41d4-a716-446655440000" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 删除文件
curl -X DELETE "http://localhost:3000/api/v1/files/550e8400-e29b-41d4-a716-446655440000" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 通用文件上传
curl -X POST "http://localhost:3000/api/v1/upload" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -F "file=@/path/to/document.pdf" \
  -F "usage_type=general"

# 上传图片到指定房间
curl -X POST "http://localhost:3000/api/v1/upload/image" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -F "file=@/path/to/photo.jpg" \
  -F "room_id=550e8400-e29b-41d4-a716-446655440001"

# 上传头像
curl -X POST "http://localhost:3000/api/v1/upload/avatar" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -F "file=@/path/to/avatar.png"

# 分片上传示例（100MB 文件，20 个分片）
# 1. 初始化
SESSION_ID=$(curl -s -X POST "http://localhost:3000/api/v1/upload/chunked/init" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{"file_name":"video.mp4","file_size":104857600,"mime_type":"video/mp4","total_chunks":20}' \
  | jq -r '.session_id')

# 2. 上传每个分片
for i in $(seq 0 19); do
  curl -s -X POST "http://localhost:3000/api/v1/upload/chunked/$SESSION_ID/$i" \
    -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
    -F "chunk=@chunk_$i.dat"
done

# 3. 查询状态
curl -s "http://localhost:3000/api/v1/upload/chunked/$SESSION_ID/status" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 4. 完成
curl -s -X POST "http://localhost:3000/api/v1/upload/chunked/$SESSION_ID/complete" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### JavaScript 示例

```javascript
// 获取文件列表
async function getFiles(category = null, usageType = null, limit = 20, offset = 0) {
  const params = new URLSearchParams();
  if (category) params.append('category', category);
  if (usageType) params.append('usage_type', usageType);
  params.append('limit', limit);
  params.append('offset', offset);
  
  const response = await fetch(
    `http://localhost:3000/api/v1/files?${params}`,
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

// 获取文件详情
async function getFile(fileId) {
  const response = await fetch(
    `http://localhost:3000/api/v1/files/${fileId}`,
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

// 删除文件
async function deleteFile(fileId) {
  const response = await fetch(
    `http://localhost:3000/api/v1/files/${fileId}`,
    {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  
  if (response.status === 204) {
    return { success: true };
  } else {
    const data = await response.json();
    throw new Error(data.message);
  }
}

// 上传文件
async function uploadFile(file, usageType = 'general', roomId = null) {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('usage_type', usageType);
  if (roomId) formData.append('room_id', roomId);
  
  const response = await fetch(
    'http://localhost:3000/api/v1/upload',
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      },
      body: formData
    }
  );
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 上传图片
async function uploadImage(file, roomId = null) {
  const formData = new FormData();
  formData.append('file', file);
  if (roomId) formData.append('room_id', roomId);
  
  const response = await fetch(
    'http://localhost:3000/api/v1/upload/image',
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      },
      body: formData
    }
  );
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 上传头像
async function uploadAvatar(file) {
  const formData = new FormData();
  formData.append('file', file);
  
  const response = await fetch(
    'http://localhost:3000/api/v1/upload/avatar',
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      },
      body: formData
    }
  );
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 分片上传
async function uploadChunked(file, usageType = 'message') {
  const chunkSize = 5 * 1024 * 1024; // 5MB
  const totalChunks = Math.ceil(file.size / chunkSize);

  // 1. 初始化
  const initRes = await fetch('/api/v1/upload/chunked/init', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      file_name: file.name,
      file_size: file.size,
      mime_type: file.type,
      usage_type: usageType,
      total_chunks: totalChunks
    })
  });
  const { session_id } = await initRes.json();

  // 2. 并发上传所有分片
  const uploads = [];
  for (let i = 0; i < totalChunks; i++) {
    const start = i * chunkSize;
    const end = Math.min(start + chunkSize, file.size);
    const chunk = file.slice(start, end);
    const formData = new FormData();
    formData.append('chunk', chunk);

    uploads.push(
      fetch(`/api/v1/upload/chunked/${session_id}/${i}`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        },
        body: formData
      }).then(r => r.json())
    );
  }

  // 3. 等待所有分片上传完成
  const results = await Promise.all(uploads);
  const lastResult = results[results.length - 1];
  console.log(`Progress: ${lastResult.received}/${lastResult.total} chunks`);

  // 4. 完成合并
  const completeRes = await fetch(`/api/v1/upload/chunked/${session_id}/complete`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`
    }
  });
  const completeData = await completeRes.json();
  return completeData.file;
}

// 分片上传带进度回调
async function uploadChunkedWithProgress(file, onProgress, usageType = 'message') {
  const chunkSize = 5 * 1024 * 1024;
  const totalChunks = Math.ceil(file.size / chunkSize);

  const initRes = await fetch('/api/v1/upload/chunked/init', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      file_name: file.name,
      file_size: file.size,
      mime_type: file.type,
      usage_type: usageType,
      total_chunks: totalChunks
    })
  });
  const { session_id } = await initRes.json();

  let completed = 0;

  // 串行上传（避免服务端负载过高）
  for (let i = 0; i < totalChunks; i++) {
    const start = i * chunkSize;
    const end = Math.min(start + chunkSize, file.size);
    const chunk = file.slice(start, end);
    const formData = new FormData();
    formData.append('chunk', chunk);

    await fetch(`/api/v1/upload/chunked/${session_id}/${i}`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      },
      body: formData
    });

    completed++;
    if (onProgress) {
      onProgress({ completed, total: totalChunks, percent: Math.round(completed / totalChunks * 100) });
    }
  }

  const completeRes = await fetch(`/api/v1/upload/chunked/${session_id}/complete`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`
    }
  });
  const completeData = await completeRes.json();
  return completeData.file;
}

// 文件上传组件示例
class FileUploader {
  constructor() {
    this.chunkSize = 1024 * 1024; // 1MB 分片
  }
  
  // 验证文件
  validateFile(file, allowedTypes = null, maxSize = 10 * 1024 * 1024) {
    // 检查文件大小
    if (file.size > maxSize) {
      throw new Error(`文件大小超过限制 (${maxSize / 1024 / 1024}MB)`);
    }
    
    // 检查文件类型
    if (allowedTypes && !allowedTypes.includes(file.type)) {
      throw new Error('不支持的文件类型');
    }
    
    return true;
  }
  
  // 格式化文件大小
  formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
  
  // 获取文件图标
  getFileIcon(category) {
    const icons = {
      image: '🖼️',
      document: '📄',
      video: '🎬',
      audio: '🎵',
      other: '📎'
    };
    return icons[category] || icons.other;
  }
}
```

---

## 文件访问

上传成功后，文件可以通过 `file_url` 访问。

### URL 构造

文件访问 URL 由以下部分组成：

```
{base_url}{file_url}
```

| 组成部分 | 说明 | 示例 |
|---------|------|------|
| `base_url` | 服务器基础 URL，由 `FILE_BASE_URL` 环境变量配置 | `http://localhost:3000` |
| `file_url` | API 返回的相对路径 | `/uploads/images/550e8400-e29b-41d4-a716-446655440000.png` |

**完整示例**:
```
GET http://localhost:3000/uploads/images/550e8400-e29b-41d4-a716-446655440000.png
```

### 配置说明

文件存储和访问 URL 通过以下环境变量配置：

| 环境变量 | 必填 | 默认值 | 说明 |
|---------|------|--------|------|
| `FILE_STORAGE_PATH` | 否 | `./uploads` | 文件本地存储路径 |
| `FILE_BASE_URL` | 否 | `http://localhost:3000` | 文件访问基础 URL |
| `FILE_MAX_SIZE` | 否 | `10485760` (10MB) | 单文件最大大小（字节） |
| `FILE_ALLOWED_TYPES` | 否 | 见上方 MIME 类型列表 | 允许的文件类型 |

> **注意**: `file_url` 返回的是相对路径，客户端需要拼接 `base_url` 得到完整 URL。如果配置了 CDN 或反向代理，应将 `FILE_BASE_URL` 设置为对应的域名。

### 访问权限

- **公开文件**: 无需认证即可访问
- **私有文件**: 需要携带 Token 或具有访问权限
- **已删除文件**: 返回 404

---

## 错误码汇总

### HTTP 状态码

| HTTP 状态码 | 错误场景 | 说明 |
|------------|---------|------|
| 200 | 请求成功 | 操作成功 |
| 204 | 删除成功 | 删除操作成功，无响应体 |
| 400 | 请求参数错误 | 文件类型不允许、文件过大等 |
| 401 | 认证失败 | Token 无效或过期 |
| 403 | 权限不足 | 不是文件上传者 |
| 404 | 资源不存在 | 文件不存在 |
| 413 | 文件过大 | 文件大小超过服务器限制 |
| 415 | 不支持的媒体类型 | MIME 类型不允许 |
| 500 | 服务器错误 | 内部服务器错误 |

### 业务错误码 (code)

| 错误码 | HTTP 状态码 | 说明 | 处理建议 |
|--------|------------|------|---------|
| `VALIDATION_ERROR` | 400 | 参数验证失败 | 检查文件类型和大小 |
| `AUTH_ERROR` | 401 | 认证失败 | 检查 Token 是否过期 |
| `FORBIDDEN` | 403 | 权限不足 | 确认用户是否有操作权限 |
| `NOT_FOUND` | 404 | 资源不存在 | 检查文件 ID 是否正确 |
| `FILE_TOO_LARGE` | 413 | 文件过大 | 压缩文件或分片上传 |
| `FILE_INVALID_TYPE` | 415 | 文件类型不允许 | 使用允许的文件类型 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 | 稍后重试或联系管理员 |

---

## 最佳实践

### 前端文件上传

1. **文件预览**: 上传前显示文件预览（图片）
2. **进度显示**: 显示上传进度条
3. **类型验证**: 客户端先验证文件类型
4. **大小验证**: 客户端先验证文件大小
5. **错误处理**: 友好的错误提示

### 文件管理

1. **定期清理**: 清理未引用的文件
2. **存储优化**: 压缩图片、转码视频
3. **CDN 加速**: 大文件使用 CDN
4. **备份策略**: 重要文件定期备份

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
