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
