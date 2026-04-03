# Seredeli Room 开发路线文档

## 项目概述

Seredeli Room 是一个基于 **Axum + WebSocket + PostgreSQL** 构建的实时聊天室应用。本文档规划了项目的开发阶段和里程碑。

### 阶段总览

| 阶段 | 名称 | 状态 | 说明 |
|------|------|------|------|
| 1 | 基础架构搭建 | ✅ 已完成 | 项目初始化、配置管理、数据库连接 |
| 2 | 用户认证系统 | ✅ 已完成 | 注册、登录、JWT Token、密码加密 |
| 3 | 聊天室管理 | ✅ 已完成 | 房间 CRUD、成员管理、权限控制 |
| 4 | WebSocket 通信 | ✅ 已完成 | 实时消息、心跳、重连、广播 |
| 5 | 消息系统 | ✅ 已完成 | 消息存储、历史记录、编辑删除 |
| 6 | 用户功能 | ✅ 已完成 | 用户资料、状态管理、用户列表 |
| 6.5 | 文件上传与资源管理 | ✅ 已完成 | 文件上传、分类存储、权限控制 |
| 7 | 测试与优化 | 🧪 待开始 | 单元测试、集成测试、性能优化 |
| 8 | 部署与运维 | 🚀 待开始 | 容器化、监控、文档 |

---

## 开发阶段

### 阶段一：基础架构搭建 ✅

**目标**：建立项目基础架构，配置开发环境，实现核心依赖的集成。

#### 任务清单

- [✅] **1.1 配置管理**
  - 实现 `config` 模块，支持从环境变量和配置文件加载配置
  - 配置 `.env` 文件和环境变量验证
  - 设置日志系统（tracing）

- [✅] **1.2 数据库连接**
  - 配置 PostgreSQL 连接池（sqlx）
  - 实现数据库迁移系统
  - 创建数据库迁移脚本

- [✅] **1.3 错误处理**
  - 完善 `AppError` 错误类型
  - 实现统一的错误响应格式
  - 添加错误日志记录

- [✅] **1.4 项目启动**
  - 完成 `main.rs` 启动逻辑
  - 实现优雅关闭（Graceful Shutdown）
  - 添加健康检查端点

#### 技术要点
- 使用 `config` crate 管理多环境配置
- 使用 `sqlx` 的编译时查询检查
- 使用 `tracing` 进行结构化日志记录

#### 验收标准
- [✅] 应用可以正常启动并连接数据库
- [✅] 数据库迁移可以自动执行
- [✅] 日志系统正常工作
- [✅] 健康检查端点返回 200

#### 完成情况
- ✅ 配置管理：支持多环境配置（`.env.development`、`.env.test`）
- ✅ 数据库连接：PostgreSQL 连接池配置完成，支持测试数据库隔离
- ✅ 错误处理：`AppError` 类型完善，统一错误响应格式实现
- ✅ 项目启动：优雅关闭机制实现，健康检查端点 `/health` 正常工作
- ✅ 测试覆盖：27 个阶段一功能测试全部通过（`tests/phase1_infrastructure_test.rs`）

---

### 阶段二：用户认证系统 ✅

**目标**：实现用户注册、登录和 JWT 认证机制。

#### 任务清单

- [✅] **2.1 用户模型**
  - 完善 `User` 数据模型
  - 实现用户注册请求验证（用户名、邮箱、密码强度）
  - 创建用户相关数据库查询

- [✅] **2.2 密码安全**
  - 集成 `argon2` 进行密码哈希
  - 实现密码强度验证（大写、小写、数字、长度）
  - 添加密码哈希验证

- [✅] **2.3 JWT 认证**
  - 实现 Token 生成和验证
  - 配置 Token 过期策略（Access Token: 24小时）
  - 实现 Token 刷新机制

- [✅] **2.4 认证接口**
  - 实现 `/api/auth/register` 注册接口
  - 实现 `/api/auth/login` 登录接口
  - 实现 `/api/auth/refresh` Token刷新接口

- [✅] **2.5 认证中间件**
  - 实现 JWT 认证中间件
  - 保护需要认证的接口
  - 处理认证错误

#### 技术要点
- 使用 `argon2` 进行安全的密码哈希
- 使用 `jsonwebtoken` 处理 JWT
- 使用 `validator` 进行请求验证
- 自定义验证函数（用户名格式、密码强度、邮箱格式）

#### 验收标准
- [✅] 用户可以正常注册账号
- [✅] 用户可以使用邮箱和密码登录
- [✅] JWT Token 可以正确生成和验证
- [✅] 受保护的接口需要有效 Token
- [✅] Token 过期后可以刷新

#### 完成情况
- ✅ 用户模型：完善 User 数据模型，实现用户名、邮箱、密码验证
- ✅ 密码安全：集成 argon2 密码哈希，实现密码强度验证
- ✅ JWT 认证：实现 Token 生成、验证、刷新机制
- ✅ 认证接口：注册、登录、刷新 Token 接口完整实现
- ✅ 认证中间件：JWT 认证中间件实现，保护受保护接口
- ✅ 测试覆盖：26 个阶段二功能测试全部通过（`tests/phase2_authentication_test.rs`）

---

### 阶段三：聊天室管理 ✅ 已完成

**目标**：实现聊天室的创建、管理和成员功能。

#### 任务清单

- [✅] **3.1 聊天室模型**
  - 完善 `Room` 和 `RoomMember` 模型
  - 实现成员角色系统（Owner/Admin/Member）
  - 创建聊天室相关数据库查询

- [✅] **3.2 聊天室接口**
  - 实现创建聊天室接口
  - 实现获取聊天室列表接口（支持分页、搜索）
  - 实现获取聊天室详情接口

- [✅] **3.3 成员管理**
  - 实现加入聊天室接口
  - 实现离开聊天室接口
  - 实现获取成员列表接口
  - 实现成员角色管理（踢出、设置管理员）

- [✅] **3.4 权限控制**
  - 实现聊天室权限检查
  - 只有 Owner/Admin 可以管理成员
  - 私有房间需要邀请才能加入

#### 技术要点
- 使用数据库事务处理成员操作
- 使用 PostgreSQL 枚举类型定义角色
- 实现软删除机制

#### 验收标准
- [✅] 用户可以创建聊天室
- [✅] 用户可以浏览公开聊天室列表
- [✅] 用户可以加入/离开聊天室
- [✅] 聊天室成员角色系统正常工作
- [✅] 权限控制正确生效

#### 完成情况
- ✅ 聊天室模型：`Room` 和 `RoomMember` 模型完善，成员角色系统实现（Owner/Admin/Member）
- ✅ 聊天室接口：创建、列表（分页/搜索）、详情、更新、删除接口完整实现
- ✅ 成员管理：加入、离开、获取成员列表、踢出成员、设置角色接口实现
- ✅ 权限控制：Owner/Admin/Member 三级权限系统，私有房间访问控制
- ✅ 测试覆盖：7 个模型测试通过（`tests/phase3_room_management_test.rs`）

---

### 阶段四：WebSocket 实时通信 ✅ 已完成

**目标**：实现 WebSocket 连接管理和实时消息传输。

#### 任务清单

- [✅] **4.1 WebSocket 管理器**
  - 完善 `WebSocketManager` 连接管理
  - 实现用户连接注册和断开处理
  - 管理房间订阅关系

- [✅] **4.2 WebSocket 处理器**
  - 实现 WebSocket 升级处理
  - 实现消息接收循环
  - 实现消息发送循环
  - 处理连接断开清理

- [✅] **4.3 消息协议**
  - 定义 WebSocket 消息格式
  - 实现消息序列化/反序列化
  - 处理各种消息类型（加入房间、发送消息、心跳等）

- [✅] **4.4 房间广播**
  - 实现房间消息广播
  - 实现单播消息（私信）
  - 处理离线用户消息（可选：消息持久化）

- [✅] **4.5 心跳机制**
  - 实现客户端心跳检测
  - 实现服务端心跳响应
  - 处理超时断开

#### 技术要点
- 使用 `dashmap` 管理并发连接
- 使用 `tokio::sync::mpsc` 进行消息通道通信
- 使用 `axum::extract::ws` 处理 WebSocket

#### 验收标准
- [✅] 客户端可以建立 WebSocket 连接
- [✅] 用户可以加入/离开房间
- [✅] 消息可以实时广播到房间所有成员
#### 完成情况
- ✅ WebSocket 管理器：`WebSocketManager` 完善，支持用户连接注册、断开处理、房间订阅管理
- ✅ WebSocket 处理器：`ws_handler` 完整实现，支持连接升级、消息收发循环、认证、断开清理
- ✅ 消息协议：`WebSocketMessage` 协议定义完整，支持认证、心跳、房间管理、消息通信、系统消息等多种类型
- ✅ 房间广播：支持房间消息广播、单播、在线用户列表同步
- ✅ 心跳机制：服务端每30秒发送Ping，客户端回复Pong，支持超时检测（90秒超时）
- ✅ 消息处理：支持加入/离开房间、发送消息、正在输入、消息已读、编辑、删除等功能
- ✅ 测试覆盖：20个 WebSocket 测试全部通过（11个单元测试 + 9个集成测试）
- ✅ 心跳超时清理：超过90秒未响应Pong的连接会被自动断开
- ✅ 断线重连机制：支持 `Reconnect` 消息重连，自动恢复房间订阅，支持获取离线消息

---

### 阶段五：消息系统 ✅

**目标**：实现消息的存储、查询和管理功能。

#### 任务清单

- [✅] **5.1 消息模型**
  - 完善 `Message` 模型
  - 支持多种消息类型（文本、图片、文件、系统消息）
  - 实现消息回复功能

- [✅] **5.2 消息存储**
  - 实现消息持久化到数据库
  - 处理 WebSocket 消息存储
  - 实现消息软删除

- [✅] **5.3 消息查询**
  - 实现获取聊天室消息历史接口
  - 实现游标分页（支持无限滚动）
  - 实现消息搜索功能

- [✅] **5.4 消息接口**
  - 实现获取历史消息接口
  - 实现搜索消息接口
  - 实现删除消息接口

#### 技术要点
- 使用游标分页优化大数据量查询（使用 created_at 时间戳而非 UUID 比较）
- 使用 PostgreSQL ILIKE 进行模糊搜索
- 消息内容大小限制和验证（1-2000 字符）

#### 验收标准
- [✅] 消息可以正确存储到数据库
- [✅] 可以获取聊天室历史消息
- [✅] 消息分页加载正常工作
- [✅] 可以搜索消息内容
- [✅] 消息可以软删除

#### 完成情况
- ✅ 消息模型：`Message` 模型完善，支持 Text/Image/File/System 类型，支持 reply_to 回复功能
- ✅ 消息存储：实现消息持久化到数据库，WebSocket 消息自动存储，软删除机制实现
- ✅ 消息查询：`get_room_messages` 支持游标分页，`search_messages` 支持模糊搜索，`get_missed_messages` 支持离线消息获取
- ✅ 消息接口：HTTP API 接口完整实现（`GET /api/messages/:room_id/messages`、`GET /api/messages/search`、`DELETE /api/messages/:message_id`）
- ✅ 权限控制：只有消息发送者才能删除自己的消息
- ✅ 测试覆盖：14 个阶段五功能测试全部通过（`tests/phase5_messaging_test.rs`）

---

### 阶段六：用户功能完善 ✅ 已完成

**目标**：完善用户相关功能，提升用户体验。

#### 任务清单

- [✅] **6.1 用户资料**
  - 实现获取当前用户信息接口 `GET /api/v1/users/me`
  - 实现更新用户信息接口 `PUT /api/v1/users/me`
  - 实现获取指定用户信息接口 `GET /api/v1/users/:user_id`
  - 支持头像URL更新

- [✅] **6.2 用户状态**
  - 实现在线状态管理（Online/Offline/Away）
  - WebSocket 实时更新用户状态 `UpdateStatus` 消息
  - 状态变更广播 `UserStatusChanged` 消息
  - 获取在线用户列表 `GetOnlineUsers` 消息

- [✅] **6.3 用户列表**
  - 实现获取用户列表接口 `GET /api/v1/users`
  - 支持搜索（用户名/邮箱模糊搜索）
  - 支持分页（limit/offset）
  - 显示用户在线状态

#### 技术要点
- 使用 WebSocket 广播用户状态变更
- 用户服务层添加搜索、统计、按状态查询等方法
- HTTP API 支持查询参数和分页响应

#### 验收标准
- [✅] 用户可以查看和修改个人资料
- [✅] 用户在线状态实时更新
- [✅] 可以浏览其他用户信息

#### 完成情况
- ✅ 用户资料接口：`/api/v1/users/me` 获取和更新当前用户信息，`/api/v1/users/:user_id` 获取指定用户信息
- ✅ 用户列表接口：`/api/v1/users` 支持搜索和分页，返回用户列表和总数
- ✅ 用户状态管理：WebSocket 支持 `UpdateStatus`、`GetOnlineUsers` 消息，状态变更自动广播
- ✅ 用户服务增强：添加 `search_users`、`count_users`、`get_online_users`、`get_users_by_status` 等方法
- ✅ 测试覆盖：17 个阶段六功能测试全部通过（`tests/phase6_user_features_test.rs`）

#### 阶段 6 扩展功能（额外开发）📎 规划中

在阶段 6 开发过程中，根据实际需求补充了以下功能：

**已完成：**

- [✅] **房间响应增强**：`RoomResponse` 添加 `updated_at` 字段，显示房间最后更新时间
  - 测试：`tests/phase6_extra_features_test.rs` - `test_room_response_has_updated_at`
  
- [✅] **最近房间列表**：新增 `GET /api/v1/rooms/recent` 接口，按 `updated_at` 降序返回最近活跃的房间
  - 支持分页（limit/offset）
  - 尊重房间隐私设置（公开/私有）
  - 支持匿名用户访问（仅公开房间）
  - 测试：`tests/phase6_extra_features_test.rs` - `test_list_recent_rooms`, `test_list_recent_rooms_pagination`, `test_recent_rooms_respects_privacy`, `test_recent_rooms_anonymous_user`

**已完成：**

- [✅] **速率限制中间件**
  - 实现基于 IP 的请求速率限制（使用 DashMap 存储请求计数）
  - 实现基于用户的请求速率限制（更严格的限制策略）
  - 针对不同接口设置不同限制策略：
    - 认证接口：5 请求/分钟
    - 消息接口：30 请求/分钟
    - 房间接口：20 请求/分钟
    - 默认接口：100 请求/分钟
  - 返回 429 Too Many Requests 状态码和 Retry-After 头部
  - 文件：`src/middleware/rate_limit.rs`
  - 测试：`tests/phase6_extra_features_test.rs` - `test_rate_limiter_ip_limit`, `test_rate_limiter_user_limit`, `test_rate_limiter_different_paths`

- [✅] **消息编辑功能**
  - 数据库表：`message_edits` 存储编辑历史
  - 消息模型添加 `edit_count` 和 `edited_at` 字段
  - API：`PUT /api/v1/messages/:message_id` 编辑消息
  - API：`GET /api/v1/messages/:message_id/history` 获取编辑历史
  - 权限控制：只有消息发送者可以编辑
  - 系统消息禁止编辑
  - 测试：`tests/phase6_extra_features_test.rs` - `test_edit_message`, `test_edit_message_permission`, `test_edit_message_history`, `test_edit_system_message_forbidden`, `test_edit_message_multiple_times`

- [✅] **消息搜索优化（全文搜索）**
  - 数据库添加 `search_vector` 字段（tsvector 类型）
  - 添加 GIN 索引优化搜索性能
  - 实现 `search_messages_fulltext` 方法
  - 支持按房间过滤和分页
  - 测试：`tests/phase6_extra_features_test.rs` - `test_search_messages_fulltext`

**待开发：**

- [ ] **消息回复功能（引用消息）**
  - 支持回复特定消息
  - 显示引用消息的上下文

---

### 阶段 6.5：文件上传与资源管理 ✅ 已完成

**目标**：实现文件上传和资源管理系统，支持图片、文档等多种文件类型的上传、存储和管理。

#### 任务清单

- [✅] **数据库设计**
  - 创建 `file_resources` 表存储文件元数据
  - 使用枚举类型 `file_category` 和 `file_usage_type` 进行分类管理
  - 支持文件与消息、用户、聊天室的关联

- [✅] **存储架构**
  - 文件按分类存储：`images/`, `documents/`, `videos/`, `audio/`, `others/`
  - 每个分类下按 `年/月` 目录结构组织
  - 生成唯一文件名避免冲突
  - 使用 SHA256 哈希进行文件去重

- [✅] **配置管理**
  - `upload_dir` 从环境变量 `UPLOAD_DIR` 读取
  - `max_file_size` 和 `base_url` 通过配置管理
  - 支持 `.env` 文件和 config 文件双重配置

- [✅] **API 接口**
  - `POST /api/v1/files/upload` - 通用文件上传
  - `POST /api/v1/files/upload/image` - 图片上传
  - `POST /api/v1/files/upload/avatar` - 头像上传（自动更新用户头像）
  - `GET /api/v1/files/:id` - 获取文件信息
  - `GET /api/v1/files` - 获取当前用户文件列表
  - `DELETE /api/v1/files/:id` - 删除文件

- [✅] **安全特性**
  - JWT 认证保护所有接口
  - 文件类型白名单验证
  - 文件大小限制
  - 用户只能删除自己的文件
  - 安全的文件名处理

#### 技术要点
- 使用 `axum` 的 multipart 支持处理文件上传
- 使用 `sha2` 计算文件哈希实现去重
- 使用 `mime` 和 `mime_guess` 进行文件类型检测
- 文件大小限制和类型白名单
- 异步文件存储操作

#### 验收标准
- [✅] 支持图片和文件消息发送
- [✅] 文件分类存储结构清晰
- [✅] 文件访问有适当的权限控制
- [✅] 文件去重机制正常工作

#### 完成情况
- ✅ 数据库设计：`file_resources` 表完整实现，支持文件分类、用途、关联关系
- ✅ 存储架构：按 `uploads/{category}/{year}/{month}/{uuid}.{ext}` 结构存储
- ✅ 配置管理：`UPLOAD_DIR` 环境变量配置，支持灵活配置
- ✅ API 接口：完整的文件上传、查询、删除接口实现
- ✅ 安全特性：文件类型验证、大小限制、权限控制完善
- ✅ 测试覆盖：13 个阶段 6.5 功能测试全部通过（`tests/phase6_5_file_upload_test.rs`）

---

### 阶段七：测试与优化 ✅ 已完成

**目标**：完善测试覆盖，优化性能和代码质量。

**状态**：已完成

#### 任务清单

- [✅] **7.1 单元测试**
  - 为服务层编写单元测试
  - 为工具函数编写测试
  - 使用 `mockall` 进行 Mock 测试

- [✅] **7.2 集成测试**
  - 编写 API 集成测试
  - 编写数据库集成测试
  - 设置测试数据库

- [✅] **7.3 WebSocket 测试**
  - 编写 WebSocket 连接测试
  - 编写消息收发测试
  - 编写并发连接测试

- [✅] **7.4 性能优化**
  - 数据库查询优化（添加索引）
  - 连接池优化
  - WebSocket 连接数优化

- [✅] **7.5 代码质量**
  - 添加代码注释和文档
  - 运行 Clippy 检查
  - 格式化代码（rustfmt）

#### 技术要点
- 使用 `cargo test` 运行测试
- 使用 `cargo bench` 进行基准测试（可选）
- 使用 `tokio::test` 进行异步测试

#### 验收标准
- [✅] 核心功能测试覆盖率达到 80%+
- [✅] 所有测试通过
- [✅] Clippy 检查无警告
- [✅] 代码格式化完成

#### 完成情况

**测试覆盖统计**：

| 测试类别 | 测试文件 | 测试数量 | 状态 |
|---------|---------|---------|------|
| 单元测试 | `src/` 内嵌测试 | 45 | ✅ 通过 |
| 阶段一测试 | `tests/phase1_infrastructure_test.rs` | 27 | ✅ 通过 |
| 阶段二测试 | `tests/phase2_authentication_test.rs` | 26 | ✅ 通过 |
| 阶段三测试 | `tests/phase3_room_management_test.rs` | 27 | ✅ 通过 |
| 阶段四测试 | `tests/phase4_websocket_test.rs` | 20 | ✅ 通过 |
| 阶段五测试 | `tests/phase5_messaging_test.rs` | 14 | ✅ 通过 |
| 阶段六测试 | `tests/phase6_user_features_test.rs` | 17 | ✅ 通过 |
| 阶段六扩展 | `tests/phase6_extra_features_test.rs` | 16 | ✅ 通过 |
| 阶段 6.5 测试 | `tests/phase6_5_file_upload_test.rs` | 13 | ✅ 通过 |
| 端到端集成测试 | `tests/integration_test.rs` | 9 | ✅ 通过 |
| WebSocket 场景测试 | `tests/websocket_test.rs` | 17 | ✅ 通过 |
| **总计** | - | **231** | **✅ 全部通过** |

**集成测试发现的优化点**：

1. **AppState 参数完善** (`tests/integration_test.rs`, `tests/phase4_websocket_test.rs`, `tests/websocket_test.rs`)
   - 问题：`AppState::new` 调用缺少 `metrics_collector` 参数
   - 修复：添加 `Arc<MetricsCollector>` 参数，完善应用状态初始化

2. **WebSocket Sender 类型统一** (`tests/phase4_websocket_test.rs`)
   - 问题：测试中使用 `UnboundedSender`，但 `WebSocketManager::connect` 要求 `Sender<String>`
   - 修复：将 `mpsc::unbounded_channel()` 改为 `mpsc::channel(100)`

3. **异步发送操作完善** (`src/websocket/handler.rs`)
   - 问题：`tx.send(json)` 未等待，导致 `let_underscore_future` 警告
   - 修复：将所有 `let _ = tx.send(json);` 改为 `let _ = tx.send(json).await;`

4. **代码质量优化** (`src/utils/security.rs`)
   - 优化：使用 `strip_prefix` 替代手动字符串切片
   - 优化：简化冗余闭包 `|c| char::from_u32(c)` → `char::from_u32`

5. **测试代码优化** (`tests/websocket_test.rs`)
   - 优化：为未使用代码添加 `#[allow(dead_code)]`
   - 优化：简化冗余模式匹配 `while let Ok(_) = ...` → `while ...is_ok()`

**代码重构与功能增强**：

6. **健康检查端点增强** (`src/routes/mod.rs`)
   - 优化：将简单的 `"OK"` 文本响应改为结构化 JSON 响应
   - 新增：返回 `success`、`status`、`timestamp` 字段，符合统一响应格式

7. **API 版本信息标准化** (`src/routes/mod.rs`)
   - 优化：将版本信息包装在 `success` + `data` 结构中
   - 新增：统一响应格式，与其他 API 保持一致

8. **HTTP 方法语义化** (`src/routes/mod.rs`)
   - 优化：将离开房间的 HTTP 方法从 `POST` 改为 `DELETE`
   - 原因：`POST /:room_id/leave` → `DELETE /:room_id/leave` 更符合 RESTful 语义

9. **消息内容安全增强** (`src/utils/validation.rs`)
   - 新增：`validate_message_content` 函数增加 XSS 攻击检测
   - 新增：`sanitize_message_content` 函数提供安全过滤版本
   - 依赖：集成 `HtmlFilter` 进行 HTML/XSS 过滤

10. **工具模块组织优化** (`src/utils/mod.rs`)
    - 新增：添加 `reconnect` 模块（指数退避重连策略）
    - 优化：完善模块文档注释，说明各模块职责

11. **WebSocket 协议完善** (`src/websocket/protocol.rs`)
    - 新增：`UserStatus` 实现 `PartialEq`，便于测试比较
    - 优化：完善消息类型文档注释
    - 新增：支持断线重连相关消息类型（`Reconnect`, `ReconnectResult`, `GetMissedMessages`, `MissedMessages`, `SessionRestored`）

12. **主程序指标收集** (`src/main.rs`)
    - 新增：初始化 `MetricsCollector` 并传递给 `AppState`
    - 新增：启动周期性指标日志任务（每 60 秒）
    - 优化：完善启动日志输出

13. **速率限制器配置** (`src/middleware/rate_limit.rs`)
    - 优化：使用 `with_default_config()` 替代 `default()`，提供更明确的配置方式
    - 优化：测试用例中使用独立的 limiter 实例，避免状态污染

14. **模型模块导出优化** (`src/models/mod.rs`)
    - 优化：重新组织模块导出结构
    - 新增：完善文件资源模型 (`FileResource`) 的字段和关联关系

**代码质量检查**：
- ✅ `cargo clippy --all-targets --all-features` 无警告
- ✅ `cargo test --all-targets` 231 个测试全部通过
- ✅ 代码已格式化（`cargo fmt`）

---

### 阶段八：部署与运维 🚀

**目标**：准备生产环境部署，配置监控和日志。

#### 任务清单

- [ ] **8.1 容器化**
  - 编写 Dockerfile
  - 编写 docker-compose.yml
  - 配置多阶段构建

- [ ] **8.2 配置管理**
  - 配置生产环境变量
  - 配置日志收集
  - 配置健康检查

- [ ] **8.3 监控**
  - 集成应用性能监控（APM）
  - 配置指标收集（Prometheus）
  - 配置告警规则

- [ ] **8.4 文档**
  - 编写 API 文档（OpenAPI/Swagger）
  - 编写部署文档
  - 编写运维手册

#### 技术要点
- 使用 Docker 进行容器化部署
- 使用环境变量管理配置
- 使用结构化日志便于分析

#### 验收标准
- [ ] 应用可以 Docker 化部署
- [ ] 生产环境配置完整
- [ ] 监控和日志系统正常工作
- [ ] 部署文档完整

---

## 技术栈总结

| 类别 | 技术 |
|------|------|
| Web 框架 | Axum |
| 异步运行时 | Tokio |
| 数据库 | PostgreSQL + sqlx |
| WebSocket | axum::extract::ws |
| 认证 | JWT + argon2 |
| 配置 | config + dotenvy |
| 日志 | tracing |
| 验证 | validator |
| 序列化 | serde + serde_json |

---

## 项目结构

```
SeredeliRoom/
├── Cargo.toml              # 项目配置和依赖
├── .env.example            # 环境变量示例
├── src/
│   ├── main.rs             # 应用入口
│   ├── lib.rs              # 库模块导出
│   ├── config/             # 配置管理
│   ├── db/                 # 数据库连接
│   ├── error/              # 错误处理
│   ├── handlers/           # HTTP 请求处理器
│   ├── middleware/         # 中间件
│   ├── models/             # 数据模型
│   ├── routes/             # 路由配置
│   ├── services/           # 业务逻辑服务
│   ├── state/              # 应用状态
│   ├── utils/              # 工具函数
│   └── websocket/          # WebSocket 处理
├── migrations/             # 数据库迁移脚本
├── tests/                  # 集成测试
└── docs/                   # 文档
```

---

## 开发建议

1. **代码规范**
   - 遵循 Rust 命名规范
   - 使用 `cargo fmt` 格式化代码
   - 使用 `cargo clippy` 检查代码

2. **数据库开发**
   - 使用 `sqlx-cli` 管理迁移
   - 使用 `sqlx prepare` 进行离线编译检查
   - 编写迁移时考虑回滚方案

3. **测试策略**
   - 先写测试后写实现（TDD）
   - 使用 `cargo test` 运行测试
   - 使用 `cargo test -- --nocapture` 查看输出

4. **Git 工作流**
   - 使用 Feature Branch 工作流
   - 提交信息遵循 Conventional Commits
   - 每个阶段完成后打标签

---

## 里程碑

| 阶段 | 预计时间 | 关键交付物 |
|------|----------|------------|
| 阶段一 | 1-2 天 | 可启动的基础项目 |
| 阶段二 | 2-3 天 | 完整的认证系统 |
| 阶段三 | 2-3 天 | 聊天室管理功能 |
| 阶段四 | 3-4 天 | WebSocket 实时通信 |
| 阶段五 | 2-3 天 | 消息系统 |
| 阶段六 | 1-2 天 | 用户功能完善 |
| 阶段七 | 2-3 天 | 测试覆盖和优化 |
| 阶段八 | 2-3 天 | 可部署的生产版本 |

---

## 注意事项

1. **安全性**
   - 密码必须哈希存储
   - JWT Secret 必须保密
   - 所有用户输入必须验证
   - 防止 SQL 注入（使用 sqlx 参数化查询）

2. **性能**
   - 使用连接池管理数据库连接
   - WebSocket 连接使用 DashMap 管理
   - 消息查询使用分页避免大数据量

3. **可扩展性**
   - 服务层和处理器分离
   - 使用状态模式便于测试
   - 配置外部化

---

*文档版本: 1.0*
*最后更新: 2026-04-03*
