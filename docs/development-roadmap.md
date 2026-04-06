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
| 7 | 测试与优化 | ✅ 已完成 | 单元测试、集成测试、性能优化 |
| 8 | 配置化与运维管理 | ✅ 已完成 | 配置体系✅、管理员系统✅、运维 API✅ |
| 8.4 | 安全审计系统 | ✅ 已完成 | 审计日志、安全告警、合规追溯 |
| 9 | 实际应用场景测试与细节修复 | ⏸️ 规划中 | 端到端测试、边界场景、细节优化 |
| 10 | 生产部署与文档 | ⏸️ 规划中 | 容器化部署、运维文档、用户手册 |

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

- [✅] **4.5 心跳机制**
  - 实现客户端心跳检测
  - 实现服务端心跳响应
  - 处理超时断开

- [✅] **4.6 消息通知系统**
  - **私信通知**：实现用户间私信推送（`PrivateMessage`）
  - **@提及通知**：检测消息中的@符号并推送通知（`Mentioned`）
  - **房间邀请通知**：私有房间邀请推送（`RoomInvitation`）
  - **系统通知**：系统广播和重要通知（`SystemNotification`）
  - **文件上传通知**：大文件上传完成通知（`FileUploadComplete`）
  - **通知历史**：存储未读通知到数据库，支持离线同步

#### 技术要点
- 使用 `dashmap` 管理并发连接
- 使用 `tokio::sync::mpsc` 进行消息通道通信
- 使用 `axum::extract::ws` 处理 WebSocket
- 使用正则表达式检测消息中的 @提及
- 使用通知服务层统一管理各类通知

#### 验收标准
- [✅] 客户端可以建立 WebSocket 连接
- [✅] 用户可以加入/离开房间
- [✅] 消息可以实时广播到房间所有成员
- [✅] 私信通知可以正确推送给目标用户
- [✅] @提及通知能够检测并推送
- [✅] 房间邀请通知可以发送给被邀请用户
- [✅] 系统通知可以广播给所有用户
- [✅] 文件上传完成后可以发送通知
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
- ✅ 消息通知系统：实现私信通知、@提及通知、房间邀请通知、系统通知、文件上传通知
  - `NotificationService` 服务层统一管理通知发送和存储
  - 支持在线推送和离线存储（`notifications` 表）
  - @提及检测使用正则表达式 `@[a-zA-Z0-9_]{3,20}`
  - 32个通知系统测试全部通过（`tests/phase4_notification_system_test.rs`）

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

- [✅] **消息回复功能（引用消息）**
  - 支持回复特定消息
  - 显示引用消息的上下文
  - 验证被回复消息的存在性和有效性
  - 测试：`tests/phase6_reply_message_test.rs`

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
| 单元测试 | `src/` 内嵌测试 | 53 | ✅ 通过 |
| 阶段一测试 | `tests/phase1_infrastructure_test.rs` | 27 | ✅ 通过 |
| 阶段二测试 | `tests/phase2_authentication_test.rs` | 26 | ✅ 通过 |
| 阶段三测试 | `tests/phase3_room_management_test.rs` | 27 | ✅ 通过 |
| 阶段四测试 | `tests/phase4_websocket_test.rs` | 20 | ✅ 通过 |
| 阶段四.6 测试 | `tests/phase4_notification_system_test.rs` | 32 | ✅ 通过 |
| 阶段五测试 | `tests/phase5_messaging_test.rs` | 14 | ✅ 通过 |
| 阶段六测试 | `tests/phase6_user_features_test.rs` | 17 | ✅ 通过 |
| 阶段六扩展 | `tests/phase6_extra_features_test.rs` | 16 | ✅ 通过 |
| 阶段 6.5 测试 | `tests/phase6_5_file_upload_test.rs` | 13 | ✅ 通过 |
| 阶段八测试 | `tests/phase8_admin_system_test.rs` | 13 | ✅ 通过 |
| 配置系统测试 | `tests/config_system_test.rs` | 14 | ✅ 通过 |
| 阶段 8.4 告警系统 | `tests/phase8_4_alert_system_test.rs` | 9 | ✅ 通过 |
| 阶段 8.4 审计系统 | `tests/phase8_4_audit_system_test.rs` | 16 | ✅ 通过 |
| 阶段 8.4 配置性能 | `tests/phase8_4_config_performance_test.rs` | 13 | ✅ 通过 |
| 端到端集成测试 | `tests/integration_test.rs` | 9 | ✅ 通过 |
| WebSocket 场景测试 | `tests/websocket_test.rs` | 17 | ✅ 通过 |
| **总计** | - | **336** | **✅ 全部通过** |

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

### 阶段八：配置化与运维管理 ✅ 已完成

**目标**：构建三层配置管理体系和管理员系统，实现运维管理功能。

**状态**：✅ 已完成

#### 任务清单

- [✅] **8.1 三层配置管理体系**
  - **环境变量层**：敏感配置和基础设施配置（如 `DATABASE_URL`, `JWT_SECRET`, `UPLOAD_DIR`）
  - **配置文件层**：创建 `config.toml` 作为项目实体配置文件，统一分散的配置项，支持配置重置操作
  - **数据库配置层**：创建 `system_configs` 表存储运行时配置，支持热更新
  - 实现配置优先级：环境变量 > 数据库配置 > 配置文件
  - 实现配置热重载机制（部分配置支持运行时更新）
  - 重构现有配置模块，支持从三层配置源读取
  - 移除代码中硬编码的默认值，确保配置由配置文件和环境变量管理

- [✅] **8.2 管理员系统**
  - 扩展用户模型，添加 `role` 字段（`SuperAdmin`, `Admin`, `User`）
  - 创建数据库迁移：添加 `user_role` 枚举类型和用户角色字段
  - 实现管理员认证中间件
  - 初始化超级管理员账号（通过环境变量或配置文件）
  - 实现权限检查工具函数

- [✅] **8.3 运维管理 API**
  - **8.3.1 用户管理接口**：
    - `GET /api/v1/admin/users` - 查看所有用户列表（支持分页、搜索）
    - `PUT /api/v1/admin/users/:id/status` - 禁用/启用用户
    - `DELETE /api/v1/admin/users/:id` - 删除用户
    - `PUT /api/v1/admin/users/:id/role` - 修改用户角色
  - **8.3.2 配置管理接口**：
    - `GET /api/v1/admin/configs` - 查看所有配置项
    - `GET /api/v1/admin/configs/:key` - 查看指定配置
    - `PUT /api/v1/admin/configs/:key` - 修改配置项（持久化到数据库）
    - `POST /api/v1/admin/configs/reset` - 重置配置到默认值（依据 config.toml）

- [✅] **8.3.3 房间管理接口**
  - `GET /api/v1/admin/rooms` - 查看所有房间列表
  - `DELETE /api/v1/admin/rooms/:id` - 强制删除房间
  - `GET /api/v1/admin/rooms/:id/messages` - 查看房间消息记录

- [✅] **8.3.4 消息审核接口**
  - `GET /api/v1/admin/messages` - 查看所有消息（支持关键词搜索）
  - `DELETE /api/v1/admin/messages/:id` - 删除违规消息

- [✅] **8.3.5 系统统计接口**
  - `GET /api/v1/admin/stats` - 系统统计（用户数、房间数、消息数、在线用户数）
  - `GET /api/v1/admin/stats/activity` - 活跃度统计（日活、周活、月活）

- [✅] **8.3.6 日志查看接口**
  - `GET /api/v1/admin/logs` - 查看应用运行日志（支持级别过滤、时间范围）
  - `GET /api/v1/admin/logs/download` - 下载日志文件

- [✅] **8.3.7 健康检查与状态**
  - 扩展健康检查端点，返回数据库连接状态、WebSocket 连接数等
  - 添加就绪检查（Readiness Probe）端点
  - 添加存活检查（Liveness Probe）端点

#### 技术要点
- 三层配置管理：环境变量 + config.toml + 数据库配置表
- 管理员角色系统：SuperAdmin / Admin / User
- 配置热更新：部分配置支持运行时修改立即生效
- 日志分级：ERROR、WARN、INFO、DEBUG，支持动态调整日志级别

#### 数据库表设计

**system_configs 表**：
```sql
CREATE TABLE system_configs (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    category VARCHAR(50), -- 配置分类：general、security、upload、rate_limit
    is_editable BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

#### 配置文件结构（config.toml）

```toml
# =============================================================================
# Seredeli Room 配置文件
# =============================================================================
# 此文件作为项目实体配置文件，用于：
# 1. 统一存储项目中分散的硬编码配置
# 2. 作为配置重置操作的基准
# 3. 提供配置默认值
# =============================================================================

[server]
# 服务器监听地址
host = "0.0.0.0"
# 服务器端口
port = 3000

[database]
# 数据库连接池大小
max_connections = 10

[jwt]
# JWT Token 过期时间（小时）
expiration_hours = 24

[upload]
# 最大文件大小（字节），默认 10MB
max_file_size = 10485760
# 文件访问基础URL路径
base_url = "/uploads"

[rate_limit]
# 是否启用速率限制
enabled = true
# 默认限制：时间窗口内的最大请求数
default_requests = 100
# 默认限制：时间窗口（秒）
default_window_secs = 60
# 认证接口限制（登录、注册等）
auth_requests = 5
# 认证接口时间窗口（秒）
auth_window_secs = 60
# 消息接口限制
message_requests = 30
# 消息接口时间窗口（秒）
message_window_secs = 60
# 房间接口限制
room_requests = 20
# 房间接口时间窗口（秒）
room_window_secs = 60

[websocket]
# WebSocket 心跳超时时间（秒）
heartbeat_timeout_secs = 90
# WebSocket 认证超时时间（秒）
auth_timeout_secs = 30
# WebSocket 消息缓冲区大小
message_buffer_size = 100

[logging]
# 日志级别：error, warn, info, debug, trace
level = "info"
# 是否启用结构化日志（JSON格式）
structured = true
# 日志文件路径（可选，默认输出到控制台）
# file_path = "./logs/app.log"
# 日志轮转配置
# max_file_size = "100MB"
# max_files = 7

[cors]
# 允许的源（* 表示允许所有）
allowed_origins = ["*"]
# 允许的方法
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
# 允许的请求头
allowed_headers = ["*"]
# 是否允许凭证
allow_credentials = false
# 预检请求缓存时间（秒）
max_age = 3600

[admin]
# 初始超级管理员配置（仅在首次启动时生效）
[admin.initial]
# 是否自动创建初始超级管理员
enabled = true
username = "admin"
email = "admin@example.com"
# 初始密码通过环境变量 ADMIN_INITIAL_PASSWORD 配置

[system]
# 系统名称
name = "Seredeli Room"
# 系统描述
description = "Real-time chat room application"
# 系统版本
version = "1.0.0"
# 维护模式
maintenance_mode = false
# 维护模式提示信息
maintenance_message = "System is under maintenance, please try again later."
```

#### 配置项分类说明

| 分类 | 配置项 | 环境变量 | 数据库配置 | 说明 |
|------|--------|----------|------------|------|
| **基础设施** | `DATABASE_URL` | ✅ | ❌ | 仅环境变量，敏感信息 |
| | `JWT_SECRET` | ✅ | ❌ | 仅环境变量，敏感信息 |
| | `UPLOAD_DIR` | ✅ | ❌ | 仅环境变量，系统路径 |
| **服务器** | `server.host` | ✅ | ✅ | 可从数据库热更新 |
| | `server.port` | ✅ | ❌ | 需重启生效 |
| **数据库** | `database.max_connections` | ✅ | ❌ | 需重启生效 |
| **JWT** | `jwt.expiration_hours` | ✅ | ✅ | 可从数据库热更新 |
| **文件上传** | `upload.max_file_size` | ✅ | ✅ | 可从数据库热更新 |
| | `upload.base_url` | ✅ | ✅ | 可从数据库热更新 |
| **速率限制** | `rate_limit.enabled` | ✅ | ✅ | 可从数据库热更新 |
| | `rate_limit.*_requests` | ✅ | ✅ | 可从数据库热更新 |
| **WebSocket** | `websocket.heartbeat_timeout_secs` | ✅ | ✅ | 可从数据库热更新 |
| | `websocket.auth_timeout_secs` | ✅ | ✅ | 可从数据库热更新 |
| **日志** | `logging.level` | ✅ | ✅ | 可从数据库热更新（动态调整） |
| **CORS** | `cors.*` | ✅ | ✅ | 可从数据库热更新 |
| **系统** | `system.maintenance_mode` | ✅ | ✅ | 可从数据库热更新 |

#### 验收标准
- [✅] 三层配置管理体系正常工作，支持配置热更新
- [✅] 管理员系统完整，支持 SuperAdmin 和 Admin 角色
- [✅] 所有运维管理 API 可用并通过测试
- [✅] 日志系统支持分级和动态调整

#### 完成情况

**三层配置管理体系**：
- ✅ 环境变量层：`DATABASE_URL`、`JWT_SECRET`、`UPLOAD_DIR` 等敏感配置
- ✅ 配置文件层：`config.toml` 统一管理项目配置，支持配置重置
- ✅ 数据库配置层：`system_configs` 表存储运行时配置，支持热更新
- ✅ 配置优先级：环境变量 > 数据库配置 > 配置文件

**管理员系统**：
- ✅ 用户角色：`SuperAdmin`、`Admin`、`User` 三级角色
- ✅ 权限控制：SuperAdmin 可管理 Admin，Admin 可管理普通用户
- ✅ 系统配置修改权限仅限 SuperAdmin
- ✅ 超级管理员初始化：通过 `config.toml` 配置自动创建

**运维管理 API**：
- ✅ 用户管理：列表、禁用/启用、删除、修改角色
- ✅ 配置管理：查看、修改、重置配置
- ✅ 房间管理：列表、强制删除、查看消息
- ✅ 消息审核：搜索、删除违规消息
- ✅ 系统统计：用户数、房间数、消息数、在线用户数
- ✅ 日志查看：级别过滤、时间范围、下载

**配置系统优化详情**：

1. **配置统一化完成**
   - 全面扫描并移除了所有硬编码配置值
   - WebSocket 处理器：心跳间隔、超时时间、缓冲区大小改为配置驱动
   - 速率限制器：请求限制、时间窗口、清理间隔改为配置驱动
   - 数据库连接：连接超时、空闲超时改为配置驱动
   - 所有配置项在 `config.toml` 中提供显式默认值

2. **热重载机制实现**
   - 配置变更事件系统：使用 `tokio::sync::broadcast` 实现事件广播
   - 事件类型：`ConfigUpdated`、`CategoryUpdated`、`ConfigReloaded`
   - 组件监听器：
     - `WebSocketConfigListener` - 监听 WebSocket 配置变更
     - `RateLimitConfigListener` - 监听速率限制配置变更
     - `LoggingConfigListener` - 监听日志级别变更
   - 配置修改后自动触发组件重新加载，无需重启应用

3. **线程安全改进**
   - 速率限制器使用 `tokio::sync::RwLock` 包装，支持并发读写
   - WebSocket 管理器配置使用原子操作和异步锁保护
   - 配置读取使用读锁，更新使用写锁，确保线程安全

4. **测试覆盖**
   - 新增配置系统集成测试：`tests/config_system_test.rs`
   - 14 个配置相关测试用例：
     - 配置加载测试（4 个）：最小配置、完整配置、错误处理
     - 热重载机制测试（2 个）：WebSocket 配置更新、速率限制配置更新
     - 配置变更事件测试（3 个）：事件类型、克隆、广播通道
     - 组件默认值测试（2 个）：管理器、限制器默认值
     - 速率限制功能测试（3 个）：IP 限制、用户限制、配置生效

**测试覆盖**：
- ✅ 13 个阶段八功能测试全部通过（`tests/phase8_admin_system_test.rs`）
- ✅ 14 个配置系统集成测试全部通过（`tests/config_system_test.rs`）
- ✅ 38 个阶段 8.4 审计系统测试全部通过（`tests/phase8_4_*_test.rs`）
- ✅ 201 个完整测试套件全部通过

**代码质量**：
- ✅ `cargo clippy --all-targets --all-features` 无警告
- ✅ 代码已格式化（`cargo fmt`）

---

### 阶段 8.4：安全审计系统 ✅ 已完成

**目标**：实现安全审计系统，记录关键操作日志，支持安全事件追溯、异常行为检测和合规审计，与现有管理员系统和通知系统协同工作。

**状态**：✅ 已完成

#### 任务清单

- [✅] **8.4.1 数据库设计**
  - 创建 `audit_logs` 表存储审计日志
  - 创建 `audit_events` 枚举定义审计事件类型
  - 创建 `audit_severity` 枚举定义事件严重级别
  - 创建 `audit_alert_rules` 表存储告警规则
  - 创建 `audit_alerts` 表存储安全告警

- [✅] **8.4.2 审计事件定义**
  - 定义用户相关审计事件（登录、登出、注册、密码修改、权限变更）
  - 定义房间相关审计事件（创建、删除、成员管理、权限变更）
  - 定义消息相关审计事件（发送、编辑、删除、举报）
  - 定义管理员操作事件（用户禁用、配置修改、强制删除）
  - 定义系统安全事件（异常登录、暴力破解、越权访问）

- [✅] **8.4.3 审计服务层**
  - 实现 `AuditService` 服务层
  - 实现审计日志记录方法（异步、批量写入）
  - 实现审计日志查询方法（支持过滤、分页、排序）
  - 实现审计日志导出功能（JSON/CSV格式）
  - 实现审计日志自动归档和清理机制

- [✅] **8.4.4 审计中间件**
  - 实现 `audit_middleware` 自动记录HTTP请求审计日志
  - 实现敏感操作标记（管理员操作、数据修改）
  - 支持从请求上下文提取审计信息（用户ID、IP地址、User-Agent）
  - 支持排除特定路径的审计记录（健康检查、静态资源）

- [✅] **8.4.5 安全告警系统**
  - 实现告警规则引擎（基于阈值、频率、模式匹配）
  - 实现异常登录检测（异地登录、频繁失败）
  - 实现暴力破解检测（短时间内大量失败请求）
  - 实现越权访问检测（尝试访问无权限资源）
  - 实现敏感操作监控（管理员操作、批量删除）

- [✅] **8.4.6 告警通知集成**
  - 集成通知服务发送安全告警
  - 支持多种告警级别（Info、Warning、Critical）
  - 支持告警聚合（相同类型告警合并）
  - 支持告警抑制（防止告警风暴）
  - 管理员实时接收Critical级别告警

- [✅] **8.4.7 管理员审计API**
  - `GET /api/v1/admin/audit/logs` - 查询审计日志（支持过滤、分页）
  - `GET /api/v1/admin/audit/logs/:id` - 获取单条审计日志详情
  - `GET /api/v1/admin/audit/stats` - 获取审计统计信息
  - `GET /api/v1/admin/audit/export` - 导出审计日志
  - `GET /api/v1/admin/audit/alerts` - 获取安全告警列表
  - `PUT /api/v1/admin/audit/alerts/:id/resolve` - 处理安全告警
  - `GET /api/v1/admin/audit/alerts/rules` - 获取告警规则
  - `PUT /api/v1/admin/audit/alerts/rules/:id` - 修改告警规则（SuperAdmin）

- [✅] **8.4.8 审计配置管理**
  - 支持配置审计日志保留时间
  - 支持配置审计事件类型开关
  - 支持配置告警规则参数
  - 支持配置审计日志存储策略（本地/远程）

#### 技术要点
- 审计日志采用异步批量写入，避免影响主业务流程性能
- 使用PostgreSQL分区表存储审计日志，支持按时间分区
- 审计中间件使用Axum的Layer机制，透明记录请求
- 告警规则引擎支持DSL配置，可动态调整检测策略
- 审计日志支持结构化存储（JSONB），便于灵活查询

#### 数据模型设计

**审计日志表 (`audit_logs`)**：
```sql
- id: UUID PRIMARY KEY
- event_type: audit_event_type (登录、登出、创建房间等)
- severity: audit_severity (Info、Warning、Error、Critical)
- actor_id: UUID (操作者用户ID，匿名操作为NULL)
- actor_role: user_role (操作者角色)
- target_type: VARCHAR (操作对象类型：user、room、message等)
- target_id: UUID (操作对象ID)
- action: VARCHAR (具体操作：create、update、delete等)
- description: TEXT (操作描述)
- metadata: JSONB (额外上下文信息：IP、User-Agent、请求参数等)
- status: VARCHAR (操作结果：success、failure)
- error_message: TEXT (失败时的错误信息)
- created_at: TIMESTAMP
```

**安全告警表 (`audit_alerts`)**：
```sql
- id: UUID PRIMARY KEY
- rule_id: UUID (关联的告警规则)
- alert_type: VARCHAR (告警类型)
- severity: audit_severity
- title: VARCHAR (告警标题)
- description: TEXT (告警描述)
- related_logs: UUID[] (关联的审计日志ID数组)
- source_ip: INET (触发告警的IP地址)
- affected_user_id: UUID (受影响的用户)
- status: alert_status (new、acknowledged、resolved、ignored)
- acknowledged_by: UUID (确认告警的管理员)
- acknowledged_at: TIMESTAMP
- resolved_by: UUID (解决告警的管理员)
- resolved_at: TIMESTAMP
- created_at: TIMESTAMP
```

**告警规则表 (`audit_alert_rules`)**：
```sql
- id: UUID PRIMARY KEY
- name: VARCHAR (规则名称)
- description: TEXT (规则描述)
- event_type: audit_event_type (监控的事件类型)
- condition: JSONB (触发条件：阈值、时间窗口、频率等)
- severity: audit_severity (触发时的严重级别)
- enabled: BOOLEAN (是否启用)
- cooldown_minutes: INT (告警冷却时间，防止重复告警)
- notify_admins: BOOLEAN (是否通知管理员)
- created_at: TIMESTAMP
- updated_at: TIMESTAMP
```

#### 工作流程

**1. 审计日志记录流程**：
```
用户操作 → 审计中间件拦截 → 提取审计信息 → 异步写入audit_logs表
                ↓
        敏感操作标记 → 触发实时分析 → 匹配告警规则
                ↓
        规则匹配成功 → 创建audit_alerts记录 → 发送通知给管理员
```

**2. 安全告警处理流程**：
```
审计服务检测异常 → 创建告警记录 → 根据severity决定通知方式
        ↓
    Critical: WebSocket实时推送 + 邮件通知
    Warning: WebSocket推送
    Info: 仅记录，不通知
        ↓
    管理员查看告警 → 确认/解决/忽略 → 更新告警状态
```

**3. 审计查询与分析流程**：
```
管理员访问审计页面 → 设置过滤条件（时间、用户、事件类型等）
        ↓
    分页查询audit_logs → 返回结构化数据
        ↓
    支持导出为JSON/CSV → 支持生成审计报告
```

#### 与现有系统集成

**与管理员系统集成**：
- 管理员操作自动记录审计日志
- 管理员可查看所有审计记录和安全告警
- SuperAdmin可配置告警规则
- 用户禁用/删除操作记录详细审计信息

**与通知系统集成**：
- Critical级别告警通过WebSocket实时推送给在线管理员
- 告警通知复用现有的`NotificationService`
- 支持告警聚合，避免通知风暴
- 告警状态变更通知相关管理员

**与配置系统集成**：
- 审计配置支持热更新
- 审计日志保留策略可配置
- 告警规则参数可动态调整
- 支持配置审计事件类型的开关

#### 完成情况

**测试覆盖统计**：

| 测试类别 | 测试文件 | 测试数量 | 状态 |
|---------|---------|---------|------|
| 告警系统测试 | `tests/phase8_4_alert_system_test.rs` | 9 | ✅ 通过 |
| 审计系统测试 | `tests/phase8_4_audit_system_test.rs` | 16 | ✅ 通过 |
| 配置性能测试 | `tests/phase8_4_config_performance_test.rs` | 13 | ✅ 通过 |
| **阶段 8.4 总计** | - | **38** | **✅ 全部通过** |

**实现的功能模块**：

- ✅ **数据库设计**：`audit_logs`、`audit_alerts`、`alert_rules` 表完整实现，支持 PostgreSQL 枚举类型
- ✅ **审计事件定义**：`AuditEventType` 枚举定义 20+ 种事件类型，`AuditSeverity` 四级严重级别
- ✅ **审计服务层**：`AuditService` 实现异步批量写入、查询过滤、JSON/CSV 导出、统计信息
- ✅ **审计中间件**：`audit_middleware` 自动记录 HTTP 请求，支持路径排除、敏感操作标记
- ✅ **安全告警系统**：`AlertEngine` 实现阈值、频率、模式匹配检测，内置 6 种预设规则
- ✅ **告警通知集成**：`AlertHandler` 集成通知服务，支持 Critical 级别实时推送
- ✅ **管理员审计 API**：完整的 REST API 实现，支持日志查询、导出、告警管理
- ✅ **审计配置管理**：`AuditConfig` 支持热更新，可配置保留时间、缓冲区大小、告警参数

**代码质量**：
- ✅ `cargo clippy --all-targets --all-features -- -D warnings` 无警告
- ✅ 代码已格式化（`cargo fmt`）
- ✅ 所有 38 个阶段 8.4 测试通过

#### 验收标准
- [✅] 所有关键操作都被记录到审计日志
- [✅] 审计日志支持按时间、用户、事件类型等多维度查询
- [✅] 安全告警能够及时检测异常行为并通知管理员
- [✅] 审计日志导出功能正常工作
- [✅] 审计日志自动归档和清理机制正常运行
- [✅] 审计系统不影响主业务流程性能（异步批量写入）

#### 性能考虑
- 审计日志写入采用异步批量处理，减少数据库压力
- 使用PostgreSQL分区表，按月份分区存储审计日志
- 审计中间件使用非阻塞方式记录日志
- 告警检测使用滑动窗口算法，避免全表扫描
- 支持审计日志归档到冷存储（对象存储）

---

### 阶段九：实际应用场景测试与细节修复 ⏸️ 规划中

**目标**：通过端到端测试和实际场景模拟，发现并修复细节问题，提升系统稳定性和用户体验。

**状态**：⏸️ 规划中

#### 任务清单

- [ ] **9.1 端到端场景测试**
  - 模拟完整用户流程：注册 → 登录 → 创建房间 → 邀请用户 → 发送消息 → 上传文件
  - 模拟多用户并发场景：多个用户同时在多个房间聊天
  - 模拟用户重连场景：网络中断后重连，验证消息不丢失
  - 模拟房间管理员操作：踢出成员、设置管理员、删除房间

- [ ] **9.2 边界场景测试**
  - 大量消息场景：单个房间 10 万 + 消息的性能和分页
  - 大量用户场景：单个房间 1000+ 用户的并发连接
  - 大量文件场景：用户上传大量文件的存储和查询
  - 长连接场景：WebSocket 连接保持数小时的稳定性

- [ ] **9.3 异常场景处理**
  - 数据库断开重连：数据库重启后应用自动恢复
  - 磁盘空间不足：文件上传失败处理
  - 恶意用户行为：频繁请求、大量消息轰炸
  - 非法数据输入：SQL 注入、XSS 攻击防护

- [ ] **9.4 用户体验优化**
  - 错误提示优化：友好的错误消息和提示
  - 加载状态优化：异步操作的加载指示器
  - 消息通知优化：新消息提醒、@提醒功能
  - 离线消息处理：离线消息推送和同步

- [ ] **9.5 性能细节优化**
  - 数据库查询优化：慢查询分析和索引优化
  - WebSocket 连接优化：连接池和内存管理
  - 文件上传优化：分片上传和断点续传
  - 缓存策略优化：热点数据缓存

#### 验收标准
- [ ] 端到端测试覆盖所有核心功能
- [ ] 边界场景测试通过，系统稳定
- [ ] 异常场景有适当的处理和提示
- [ ] 用户体验流畅，无明显卡顿
- [ ] 性能指标达到预期（响应时间、并发量）

---

### 阶段十：生产部署与文档 ⏸️ 规划中

**目标**：完成容器化部署，编写完整的运维文档和用户手册。

**状态**：⏸️ 规划中

#### 任务清单

- [ ] **10.1 容器化部署**
  - 编写 Dockerfile，支持多阶段构建优化镜像大小
  - 编写 docker-compose.yml，定义应用、数据库、Nginx 等服务
  - 配置 Docker 健康检查和日志收集
  - 支持环境变量配置容器参数

- [ ] **10.2 CI/CD 流程**
  - 配置 GitHub Actions 或 GitLab CI
  - 自动化测试：提交时自动运行测试
  - 自动化构建：自动构建 Docker 镜像
  - 自动化部署：推送到生产环境

- [ ] **10.3 监控与告警**
  - 集成 Prometheus 收集指标
  - 集成 Grafana 展示监控面板
  - 配置告警规则（CPU、内存、错误率）
  - 日志聚合分析（ELK 或 Loki）

- [ ] **10.4 API 文档**
  - 使用 OpenAPI/Swagger 规范编写 API 文档
  - 提供在线 API 文档和测试界面
  - 包含所有接口的请求/响应示例
  - 包含认证和授权说明

- [ ] **10.5 部署文档**
  - 编写生产环境部署指南
  - 编写配置文件说明文档
  - 编写环境变量说明文档
  - 编写数据库迁移指南

- [ ] **10.6 运维手册**
  - 编写管理员操作指南
  - 编写常见问题排查手册
  - 编写备份和恢复策略
  - 编写性能调优指南

- [ ] **10.7 用户手册**
  - 编写用户使用指南
  - 编写功能说明文档
  - 提供常见问题解答（FAQ）
  - 提供最佳实践建议

#### 技术要点
- 使用 Docker 进行容器化部署
- 使用 Docker Compose 编排多容器应用
- 使用 Nginx 作为反向代理
- 使用 Prometheus + Grafana 监控
- 使用 OpenAPI 规范编写 API 文档

#### 验收标准
- [ ] 应用可以通过 Docker Compose 一键启动
- [ ] CI/CD 流程自动化运行
- [ ] 监控面板正常展示系统指标
- [ ] API 文档完整且可在线访问
- [ ] 部署文档清晰易懂
- [ ] 运维手册覆盖日常运维场景

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
| 阶段八 | 2-3 天 | 配置管理和运维 API |
| 阶段九 | 3-5 天 | 场景测试和细节优化 |
| 阶段十 | 2-3 天 | 容器化部署和完整文档 |

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
*最后更新: 2026-04-04*
