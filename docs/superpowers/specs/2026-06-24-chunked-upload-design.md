# 分片上传设计

## 背景

当前文件上传为单次 `multipart/form-data` 提交，最大 10MB。对于大文件场景（视频、压缩包等），需要分片上传来支持断点续传和弱网环境。路线图 9.5 标注为待实施。

## 设计原则

- 分片存储使用磁盘临时目录，不占 Redis/DB 内存
- 会话元数据使用 Redis（带 TTL），完成后归档到 DB
- 合并后复用现有 `FileService::upload_file()` 完成去重和记录
- 不引入新依赖

## 架构

```
客户端                      服务端
 │                           │
 ├── POST /upload/init ─────→│  Redis 创建会话（TTL 24h）
 │  ← {session_id, chunk_size}│
 │                           │
 ├── POST /upload/{sid}/chunk│  写入磁盘 .chunks/{sid}/{index}
 │  × N ───────────────────→│  更新 Redis 会话状态
 │  ← {received, total}     │
 │                           │
 ├── GET /upload/{sid} ─────→│  查询缺失分片（断点续传）
 │  ← missing_chunks[]      │
 │                           │
 ├── POST /upload/{sid}/complete│  合并分片 → FileService::upload_file()
 │  ───────────────────────→│  → 清理临时目录 + 删除 Redis key
 │  ← FileUploadResponse    │
 │                           │
 └── DELETE /upload/{sid} ──→│  清理临时分片 + 删除 Redis key
```

## API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/v1/upload/init` | POST | 创建上传会话 |
| `/api/v1/upload/{session_id}/chunk` | POST | 上传单个分片 |
| `/api/v1/upload/{session_id}` | GET | 查询会话状态（含缺失分片） |
| `/api/v1/upload/{session_id}/complete` | POST | 合并分片完成上传 |
| `/api/v1/upload/{session_id}` | DELETE | 取消上传 |

### Request/Response

**POST /upload/init**
```json
// Request
{ "file_name": "video.mp4", "file_size": 104857600, "mime_type": "video/mp4",
  "usage_type": "message", "total_chunks": 20 }
// Response
{ "session_id": "uuid", "chunk_size": 5242880, "total_chunks": 20 }
```

**POST /upload/{session_id}/chunk**
```
// Multipart: chunk_index (text) + file (binary)
// Response
{ "received": 3, "total": 20 }
```

**GET /upload/{session_id}**
```json
// Response
{ "session_id": "uuid", "file_name": "video.mp4", "file_size": 104857600,
  "status": "active", "total_chunks": 20, "received_chunks": [0,1,2],
  "missing_chunks": [3,4,...,19], "created_at": "2026-06-24T..." }
```

**POST /upload/{session_id}/complete**
```json
// Response (same as existing FileUploadResponse)
{ "id": "uuid", "original_name": "video.mp4", "file_url": "/uploads/...",
  "file_size": 104857600, "mime_type": "video/mp4", "category": "video" }
```

**DELETE /upload/{session_id}**
```json
// Response
{ "success": true }
```

## 数据结构

### Redis 会话（TTL 24h）

```rust
struct UploadSessionMeta {
    session_id: Uuid,
    user_id: Uuid,
    file_name: String,
    file_size: u64,
    mime_type: String,
    usage_type: FileUsageType,
    total_chunks: u32,
    received_chunks: Vec<u32>,
    chunk_size: u32,
    status: UploadStatus,  // Active | Completed | Cancelled
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}
```

### 分片存储

`{upload_dir}/.chunks/{session_id}/{chunk_index}`

## 文件变更

| 文件 | 改动 |
|------|------|
| `src/services/upload_session.rs` | **新增**：UploadSessionManager（Redis 会话 CRUD + TTL） |
| `src/handlers/file.rs` | 新增 5 个 handler：init / chunk / status / complete / cancel |
| `src/services/file_service.rs` | 新增 `init_session`、`save_chunk`、`get_session`、`complete_session`、`cancel_session`、`merge_chunks`、`cleanup_expired_sessions` |
| `src/models/file.rs` | 新增 UploadSessionResponse、UploadChunkResponse、UploadStatusResponse DTO |
| `src/config/mod.rs` | UploadConfig 增加 `chunked_upload_enabled`、`default_chunk_size`、`session_ttl_hours` |
| `config.toml` | 增加分片上传默认值 |
| `src/state/mod.rs` | 初始化 UploadSessionManager，传给 AppState |
| `src/main.rs` | 创建 UploadSessionManager |
| `src/routes/mod.rs` | 挂载新端点 |
| `docs/api/v1/http/files.md` | 新增分片上传 API 文档节 |

## 配置

```toml
[upload]
max_file_size = 10485760
base_url = "/uploads"
chunked_upload_enabled = true
default_chunk_size = 5242880    # 5MB
session_ttl_hours = 24
```

## 错误处理

| 场景 | HTTP | message |
|------|------|---------|
| 分片索引重复 | 200 | 幂等覆盖，返回当前状态 |
| 分片索引越界 | 400 | `chunk_index out of range` |
| 会话已过期 | 404 | `session not found or expired` |
| 合并时缺分片 | 400 | `missing chunks: [2, 5]` |
| Redis 不可用 | 500 | `upload service unavailable` |
| 文件去重 | 200 | 自动处理，走现有逻辑 |

## 测试

- `test_upload_init_creates_session` — 初始化会话
- `test_upload_chunk_saves_to_disk` — 分片写入
- `test_upload_chunk_duplicate_idempotent` — 重复分片幂等
- `test_upload_complete_merges_chunks` — 合并成功
- `test_upload_complete_missing_chunks` — 缺分片拒绝
- `test_upload_cancel_cleans_up` — 取消清理
- `test_upload_session_ttl` — 会话过期
