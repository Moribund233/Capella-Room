# 后端细节修复清单

> 本文档记录后端 API 响应结构中存在的用户体验问题，供阶段9优化时参考。

---

##  0001-后端多处 API 响应只返回 `user_id`（UUID 格式），前端无法直接显示友好的用户名，影响用户体验。

---

## 问题列表

### 1.  ✅ RoomResponse 缺少房主名称 [P1-高优先级] - 已修复

**位置**: `src/models/room.rs` - `RoomResponse` 结构体

**修复方案**:
- 采用统一 UserInfo 结构体方案
- 将 `owner_id: Uuid` 改为 `owner: UserInfo`
- SQL 查询使用 JOIN 获取用户信息

**修复后代码**:
```rust
pub struct RoomResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner: UserInfo,  // ✅ 使用 UserInfo 替代 owner_id
    pub is_private: bool,
    pub max_members: i32,
    pub member_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**影响文件**:
- `src/models/room.rs` - RoomResponse 定义
- `src/models/user.rs` - UserInfo 结构体
- `src/services/room_service.rs` - SQL 查询修改（6处）

**前端同步修改**:
- `client/debug/src/types/api.ts` - Room 接口 owner_id → owner
- `client/debug/src/views/RoomManager.vue` - 显示用户名和头像

---

### 2.  ✅ AuditAlert 缺少用户名称信息 [P2-中优先级] - 已修复

**位置**: `src/models/audit.rs` - `AuditAlertResponse` 结构体

**修复方案**:
- 创建新的 `AuditAlertResponse` DTO 结构体
- 使用 `UserInfo` 替代裸 UUID
- 原 `AuditAlert` 保持数据库模型不变

**修复后代码**:
```rust
pub struct AuditAlertResponse {
    pub id: Uuid,
    pub rule_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: AuditSeverity,
    pub title: String,
    pub description: String,
    pub affected_user: Option<UserInfo>,  // ✅ 使用 UserInfo
    pub status: AlertStatus,
    pub acknowledged_by: Option<UserInfo>,  // ✅ 使用 UserInfo
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<UserInfo>,  // ✅ 使用 UserInfo
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**影响文件**:
- `src/models/audit.rs` - 新增 AuditAlertResponse
- `src/services/audit_service.rs` - SQL 查询修改

**前端同步修改**:
- `client/debug/src/types/api.ts` - 新增 AuditAlert 接口

---

### 3.  ✅ FileResponse 缺少上传者信息 [P2-中优先级] - 已修复

**位置**: `src/models/file.rs` - `FileResponse` 结构体

**修复方案**:
- 添加 `uploader: Option<UserInfo>` 字段
- SQL 查询 LEFT JOIN users 表

**修复后代码**:
```rust
pub struct FileResponse {
    pub id: Uuid,
    pub original_name: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: String,
    pub category: FileCategory,
    pub usage_type: FileUsageType,
    pub uploader: Option<UserInfo>,  // ✅ 新增上传者信息
    pub created_at: DateTime<Utc>,
}
```

**影响文件**:
- `src/models/file.rs` - FileResponse 定义
- `src/services/file_service.rs` - SQL 查询修改

**前端同步修改**:
- `client/debug/src/types/api.ts` - 新增 FileResource 接口

---

### 4.  ✅ 统一 UserInfo 结构体 [P3-低优先级/架构优化] - 已修复

**修复方案**:
创建统一的 `UserInfo` 结构体，并在多处复用：

```rust
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
}
```

**应用范围**:
- ✅ `RoomResponse.owner` - 房主信息
- ✅ `AuditAlertResponse.affected_user` - 受影响用户
- ✅ `AuditAlertResponse.acknowledged_by` - 确认者
- ✅ `AuditAlertResponse.resolved_by` - 解决者
- ✅ `FileResponse.uploader` - 上传者
- ✅ `MessageResponse.sender` - 使用独立的 `SenderInfo`（保持独立，不强制复用）

**设计决策**:
- `SenderInfo` 保持独立结构，不强制复用 `UserInfo`，避免增加代码理解难度
- 两者结构相同但语义不同，便于后续独立演进

**影响文件**:
- `src/models/user.rs` - UserInfo 定义
- `src/models/room.rs` - RoomResponse 使用 UserInfo
- `src/models/audit.rs` - AuditAlertResponse 使用 UserInfo
- `src/models/file.rs` - FileResponse 使用 UserInfo
- `src/models/message.rs` - SenderInfo（独立但结构相同）

---

## 修复工作量评估（实际）

| 问题 | 影响文件数 | SQL查询修改 | 实际工作量 | 状态 |
|------|-----------|------------|-----------|------|
| RoomResponse 添加 owner | 3 | 6处 | 中等 | ✅ 完成 |
| AuditAlert 添加用户名称 | 2 | 2处 | 小 | ✅ 完成 |
| FileResponse 添加上传者 | 2 | 2处 | 小 | ✅ 完成 |
| 统一 UserInfo 结构体 | 5+ | 多处 | 较大 | ✅ 完成 |
| 前端类型同步修改 | 5 | - | 中等 | ✅ 完成 |

---

## 临时解决方案

在修复完成前，前端可采用以下方式处理：
1. UUID 截断显示：只显示前8位，如 `55bea4ba...`
2. 悬停提示：鼠标悬停时显示完整UUID
3. 异步加载：点击后单独查询用户信息

**状态**: 已修复，前端现在可以直接显示友好的用户名和头像。

---

## 相关文件清单

### 后端
- `src/models/user.rs` - UserInfo 定义
- `src/models/room.rs` - RoomResponse 定义
- `src/models/audit.rs` - AuditAlertResponse 定义
- `src/models/file.rs` - FileResponse 定义
- `src/models/message.rs` - SenderInfo 定义
- `src/services/room_service.rs` - 房间查询逻辑
- `src/services/audit_service.rs` - 审计查询逻辑
- `src/services/file_service.rs` - 文件查询逻辑

### 前端
- `client/debug/src/types/api.ts` - API 类型定义
- `client/debug/src/api/message.ts` - Message 类型
- `client/debug/src/api/index.ts` - 类型导出
- `client/debug/src/views/RoomManager.vue` - 房间管理组件
- `client/debug/src/views/MessageTest.vue` - 消息测试组件

---

## 0002-WebSocket 心跳机制双向不匹配导致周期性断连 [P0-紧急] - ✅ 已修复并验证

**位置**: `src/websocket/handler.rs` - `handle_message` 函数

**问题描述**:
后端设计意图为"服务端主导心跳"（服务端每30秒发Ping，客户端回复Pong），但实际实现中：
1. 服务端发送 Ping 后，客户端未回复 Pong
2. 客户端发送 Ping 后，服务端未回复 Pong（仅记录警告）
3. 双方都在发送 Ping，但都不回复对方的 Ping
4. 前端心跳超时更短（60秒），导致每60秒周期性断连

**修复方案**（混合模式 - 双向心跳 + 配置同步）：

### 1. 后端修复
在 `handle_message` 中添加处理客户端 Ping 的逻辑：

```rust
// 心跳请求 - 回复 Pong（支持客户端主导的心跳）
WebSocketMessage::Ping => {
    let pong = WebSocketMessage::Pong;
    if let Ok(json) = pong.to_json() {
        let _ = tx.send(json).await;
    }
    debug!("Received ping from user: {}, sent pong", user_id);
}

// 心跳响应
WebSocketMessage::Pong => {
    if let Ok(mut last) = last_pong.lock() {
        *last = Instant::now();
    }
    debug!("Received pong from user: {}", user_id);
}
```

### 2. 配置同步方案
为避免前后端心跳配置不一致导致的问题，实现了公开配置端点：

**后端**: `GET /api/config/client` - 返回客户端需要的配置
```rust
pub struct ClientConfig {
    pub websocket: ClientWebSocketConfig,
    pub reconnect: ClientReconnectConfig,
    pub upload: ClientUploadConfig,
    pub system: ClientSystemConfig,
}
```

**前端**: 页面加载时自动获取服务端配置
```javascript
const response = await fetch('http://localhost:8080/api/config/client');
const serverConfig = response.data;
// 使用服务端配置的心跳间隔
const interval = serverConfig.websocket.heartbeat_interval_secs * 1000;
```

### 3. 验证结果
终端日志分析（连接时长 > 7分钟，无断连记录）：
```
08:02:14 - WebSocket 连接建立，用户认证成功
08:02:14 - 收到客户端 Pong
08:02:44 - 收到客户端 Ping，回复 Pong
08:02:44 - 收到客户端 Pong
08:03:14 - 收到客户端 Ping，回复 Pong
08:03:14 - 收到客户端 Pong
...（持续 7+ 分钟，每 30 秒双向心跳正常）
08:09:14 - 仍在正常心跳交互，无断连
```

**状态**: ✅ 已修复并验证 - 混合心跳模式工作正常，连接稳定

**验证方式**: 
- HTML+JS 测试页面 (`temp/test/websocket-heartbeat-test.html`)
- Chrome DevTools 自动化测试
- 终端日志确认无断连

**影响文件**:
- `src/websocket/handler.rs` - 添加 Ping 处理分支
- `src/config/mod.rs` - ClientConfig 定义
- `src/handlers/config.rs` - 公开配置端点
- `src/routes/mod.rs` - 路由配置
- `temp/test/websocket-heartbeat-test.html` - 测试页面

**关联配置**:
- `websocket.heartbeat_interval_secs` - 30秒（服务端发Ping间隔）
- `websocket.heartbeat_timeout_secs` - 90秒（服务端超时）
- 前端通过 `/api/config/client` 动态获取配置

---

*文档创建时间: 2026-04-10*
*关联任务: 阶段9 - 后端细节优化*
*更新时间: 2026-04-12 - 添加 0002 问题修复验证记录和配置同步方案*
