# 后端细节修复清单

> 本文档记录后端 API 响应结构中存在的用户体验问题，供阶段9优化时参考。

---

##  0001-后端多处 API 响应只返回 `user_id`（UUID 格式），前端无法直接显示友好的用户名，影响用户体验。

---

## 问题列表

### 1.  [ ]  RoomResponse 缺少房主名称 [P1-高优先级] 

**位置**: `src/models/room.rs` - `RoomResponse` 结构体

**当前状态**:
```rust
pub struct RoomResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,  // ❌ 只有ID，没有名称
    pub is_private: bool,
    pub max_members: i32,
    pub member_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**影响接口**:
- `GET /api/v1/rooms` - 房间列表
- `GET /api/v1/rooms/recent` - 最近房间
- `GET /api/v1/rooms/:id` - 房间详情
- `GET /api/v1/users/me/rooms` - 我的房间
- `GET /api/v1/users/me/rooms/search` - 搜索我的房间

**修复建议**:
- 添加 `owner_name: String` 字段
- 修改 SQL 查询，JOIN users 表获取用户名
- 影响文件：`src/models/room.rs`, `src/services/room_service.rs` (多处 SQL)

---

### 2.  [ ]  AuditAlert 缺少用户名称信息 [P2-中优先级]

**位置**: `src/models/audit.rs` - `AuditAlert` 结构体

**当前状态**:
```rust
pub struct AuditAlert {
    pub id: Uuid,
    pub rule_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: AuditSeverity,
    pub title: String,
    pub description: String,
    pub related_logs: Option<Vec<Uuid>>,
    pub source_ip: Option<String>,
    pub affected_user_id: Option<Uuid>,  // ❌ 只有ID
    pub status: AlertStatus,
    pub acknowledged_by: Option<Uuid>,   // ❌ 只有ID
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,       // ❌ 只有ID
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**影响接口**:
- `GET /api/v1/admin/audit/alerts` - 告警列表
- `GET /api/v1/admin/audit/alerts/:id` - 告警详情

**修复建议**:
- 添加 `affected_user_name: Option<String>`
- 添加 `acknowledged_by_name: Option<String>`
- 添加 `resolved_by_name: Option<String>`
- 或使用嵌套结构 `affected_user: Option<UserInfo>`

---

### 3.  [ ]  FileResponse 缺少上传者信息 [P2-中优先级]

**位置**: `src/models/file.rs` - `FileResponse` 结构体

**当前状态**:
```rust
pub struct FileResponse {
    pub id: Uuid,
    pub original_name: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: String,
    pub category: FileCategory,
    pub usage_type: FileUsageType,
    pub created_at: DateTime<Utc>,
    // ❌ 缺少 uploader_id 和 uploader_name
}
```

**数据库模型** (`FileResource`):
```rust
pub struct FileResource {
    pub id: Uuid,
    pub uploader_id: Option<Uuid>,  // 有ID但未在响应中暴露
    // ...
}
```

**影响接口**:
- `GET /api/v1/files` - 文件列表
- `GET /api/v1/files/:id` - 文件详情
- 上传文件相关接口

**修复建议**:
- 在 `FileResponse` 中添加 `uploader: Option<UserInfo>` 或 `uploader_name: Option<String>`

---

### 4.  [ ]  统一 UserInfo 结构体 [P3-低优先级/架构优化]

**问题描述**:
后端多个地方需要返回用户信息，但实现方式不统一：
- `MessageResponse` 使用 `SenderInfo { id, username, avatar_url }`
- `RoomMemberWithUser` 直接内联用户字段
- 其他响应使用裸 `user_id: Uuid`

**建议方案**:
创建一个统一的 `UserInfo` 结构体：
```rust
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
}
```

**应用范围**:
- 替换 `RoomResponse` 中的 `owner_id` -> `owner: UserInfo`
- 替换 `AuditAlert` 中的用户ID字段
- 替换 `FileResponse` 中的上传者信息
- 统一 `RoomMemberWithUser` 的结构

---

## 修复工作量评估

| 问题 | 影响文件数 | SQL查询修改 | 预估工作量 |
|------|-----------|------------|-----------|
| RoomResponse 添加 owner_name | 2 | 6-8处 | 中等 |
| AuditAlert 添加用户名称 | 2 | 2-3处 | 小 |
| FileResponse 添加上传者 | 2 | 2-3处 | 小 |
| 统一 UserInfo 结构体 | 5+ | 多处 | 较大 |

---

## 临时解决方案

在修复完成前，前端可采用以下方式处理：
1. UUID 截断显示：只显示前8位，如 `55bea4ba...`
2. 悬停提示：鼠标悬停时显示完整UUID
3. 异步加载：点击后单独查询用户信息

---

## 相关文件清单

- `src/models/room.rs` - RoomResponse 定义
- `src/models/audit.rs` - AuditAlert 定义
- `src/models/file.rs` - FileResponse 定义
- `src/models/message.rs` - SenderInfo 参考实现
- `src/services/room_service.rs` - 房间查询逻辑
- `src/services/audit_service.rs` - 审计查询逻辑
- `src/services/file_service.rs` - 文件查询逻辑

---

*文档创建时间: 2026-04-10*
*关联任务: 阶段9 - 后端细节优化*
