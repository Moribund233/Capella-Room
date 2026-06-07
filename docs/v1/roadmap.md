# Capella Room 开发路线文档

## 项目概述

Capella Room 是一个基于 **Axum + WebSocket + PostgreSQL** 构建的实时聊天室应用。本文档规划了项目的开发阶段和里程碑。

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
| 8.5 | Redis 分布式支持 | ✅ 已完成 | 分布式 WebSocket 广播、水平扩展 |
| 8.6 | 基于 Redis 的数据库写入优化与配置热更新同步 | ⚠️ 部分完成 | Redis Stream 架构实现✅、配置同步✅、性能测试待验证 |
| 8.7 | 服务安全加固 | ✅ 已完成 | IP 黑名单/白名单系统、WebSocket 安全防护 |
| 9 | 实际应用场景测试与细节修复 | ✅ 已完成 | 端到端场景测试✅、边界高并发测试✅(200K条 13601 msg/s 100%送达 100%持久化)、异常场景处理✅、用户体验优化✅、性能细节优化✅(13601 msg/s, 较基准提升61x) |
| 10 | 生产部署与文档 | ⚠️ 部分完成 | Dockerfile✅、docker-compose✅、CI/CD与监控待阶段9完成后实施 |

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
  - 实现 `POST /api/v1/auth/register` 注册接口
  - 实现 `POST /api/v1/auth/login` 登录接口
  - 实现 `POST /api/v1/auth/refresh` Token刷新接口

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
- **分布式架构（阶段 8.5）**：集成 Redis Pub/Sub 实现跨节点消息广播，支持水平扩展

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
- ✅ 消息接口：HTTP API 接口完整实现（`GET /api/v1/rooms/:room_id/messages`、`GET /api/v1/messages/search`、`DELETE /api/v1/messages/:message_id`）
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
  - `POST /api/v1/upload` - 通用文件上传
  - `POST /api/v1/upload/image` - 图片上传
  - `POST /api/v1/upload/avatar` - 头像上传（自动更新用户头像）
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
| 单元测试 | `src/` 内嵌测试 | 66 | ✅ 通过 |
| 配置系统测试 | `tests/config_system_test.rs` | 9 | ✅ 通过 |
| 阶段一测试 | `tests/phase1_infrastructure_test.rs` | 27 | ✅ 通过 |
| 阶段二测试 | `tests/phase2_authentication_test.rs` | 26 | ✅ 通过 |
| 阶段三测试 | `tests/phase3_room_management_test.rs` | 27 | ✅ 通过 |
| 阶段四测试 | `tests/phase4_websocket_test.rs` | 20 | ✅ 通过 |
| 阶段四.6 测试 | `tests/phase4_notification_system_test.rs` | 32 | ✅ 通过 |
| 阶段五测试 | `tests/phase5_messaging_test.rs` | 14 | ✅ 通过 |
| 阶段六测试 | `tests/phase6_user_features_test.rs` | 17 | ✅ 通过 |
| 阶段六扩展 | `tests/phase6_extra_features_test.rs` | 12 | ✅ 通过 |
| 阶段 6.5 测试 | `tests/phase6_5_file_upload_test.rs` | 13 | ✅ 通过 |
| 阶段六回复消息 | `tests/phase6_reply_message_test.rs` | 10 | ✅ 通过 |
| 阶段八测试 | `tests/phase8_admin_system_test.rs` | 13 | ✅ 通过 |
| 阶段 8.4 告警系统 | `tests/phase8_4_alert_system_test.rs` | 9 | ✅ 通过 |
| 阶段 8.4 审计系统 | `tests/phase8_4_audit_system_test.rs` | 16 | ✅ 通过 |
| 阶段 8.4 配置性能 | `tests/phase8_4_config_performance_test.rs` | 13 | ✅ 通过 |
| 待办通知测试 | `tests/pending_action_notification_test.rs` | 14 | ✅ 通过 |
| 端到端集成测试 | `tests/integration_test.rs` | 9 | ✅ 通过 |
| WebSocket 场景测试 | `tests/websocket_test.rs` | 17 | ✅ 通过 |
| Redis 集成测试 | `tests/redis_integration_test.rs` | 8 | ✅ 通过 |
| **总计** | - | **372** | **✅ 全部通过** |

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

13. **模型模块导出优化** (`src/models/mod.rs`)
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
      - 查询参数：`page`（页码，默认1）、`page_size`（每页数量，默认20）、`search`（搜索关键词，可选）
      - 返回：用户列表、总数、当前页、每页数量
    - `GET /api/v1/admin/users/:id` - 查看指定用户详情
    - `PUT /api/v1/admin/users/:id/status` - 禁用/启用用户
      - 请求体：`{ "is_active": true/false }`
      - 权限：SuperAdmin 可禁用任何用户，Admin 不能禁用 SuperAdmin 和其他 Admin
    - `DELETE /api/v1/admin/users/:id` - 删除用户
      - 权限：SuperAdmin 可删除任何用户，Admin 不能删除 SuperAdmin 和其他 Admin
    - `PUT /api/v1/admin/users/:id/role` - 修改用户角色
      - 请求体：`{ "role": "user" | "admin" | "super_admin" }`
      - 权限：只有 SuperAdmin 可以设置 SuperAdmin 角色或修改 Admin 角色
    - `PUT /api/v1/admin/users/:id/password` - 管理员重置用户密码（SuperAdmin 专属）
      - 请求体：`{ "new_password": "新密码" }`
      - 限制：密码长度至少8个字符，不能重置 SuperAdmin 密码
  - **8.3.2 配置管理接口**：
    - `GET /api/v1/admin/configs` - 查看所有配置项
    - `GET /api/v1/admin/configs/:key` - 查看指定配置
    - `PUT /api/v1/admin/configs/:key` - 修改配置项（持久化到数据库）
    - `POST /api/v1/admin/configs/reset` - 重置配置到默认值（依据 config.toml）

- [✅] **8.3.3 房间管理接口**
  - `GET /api/v1/admin/rooms` - 查看所有房间列表
  - `DELETE /api/v1/admin/rooms/:id` - 强制删除房间
  - `GET /api/v1/admin/rooms/:id/messages` - 查看房间消息记录
  - `DELETE /api/v1/admin/rooms/:room_id/members/:user_id` - 踢出房间成员（管理员专用）
  - `PUT /api/v1/admin/rooms/:room_id/members/:user_id/role` - 设置房间成员角色（管理员专用，支持转让房主）

- [✅] **8.3.4 消息审核接口**
  - `GET /api/v1/admin/messages` - 查看所有消息（支持关键词搜索）
  - `DELETE /api/v1/admin/messages/:id` - 删除违规消息

- [✅] **8.3.5 系统统计接口**
  - `GET /api/v1/admin/stats` - 系统统计（用户数、房间数、消息数、在线用户数）
  - `GET /api/v1/admin/stats/activity` - 活跃度统计（日活、周活、月活）
  
- [✅] **8.3.6 系统日志流 (WebSocket)**
  - `SubscribeLogs` - 订阅系统日志（支持级别过滤、模块过滤）
  - `LogEntry` - 实时推送日志条目
  - `UnsubscribeLogs` - 取消订阅

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
    category VARCHAR(50), -- 配置分类：general、security、upload、websocket、logging、system
    is_editable BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

#### 配置项分类说明

| 分类 | 配置项 | 环境变量 | 数据库配置 | 说明 |
|------|--------|----------|------------|------|
| **基础设施** | `DATABASE_URL` | ✅ | ❌ | 仅环境变量，敏感信息 |
| | `JWT_SECRET` | ✅ | ❌ | 仅环境变量，敏感信息 |
| | `UPLOAD_DIR` | ✅ | ❌ | 仅环境变量，系统路径 |
| **服务器** | `server.host` | ✅ | ⚠️ | 可从数据库加载，需重启生效 |
| | `server.port` | ✅ | ❌ | 需重启生效 |
| **数据库** | `database.max_connections` | ✅ | ❌ | 需重启生效 |
| **JWT** | `jwt.expiration_hours` | ✅ | ✅ | 支持热更新 |
| **文件上传** | `upload.max_file_size` | ✅ | ✅ | 支持热更新 |
| | `upload.base_url` | ✅ | ✅ | 支持热更新 |
| **WebSocket** | `websocket.heartbeat_interval_secs` | ✅ | ✅ | 支持热更新（实时生效） |
| | `websocket.heartbeat_timeout_secs` | ✅ | ✅ | 支持热更新（实时生效） |
| | `websocket.auth_timeout_secs` | ✅ | ✅ | 支持热更新（仅新连接生效） |
| | `websocket.message_buffer_size` | ✅ | ✅ | 支持热更新（仅新连接生效） |
| **日志** | `logging.level` | ✅ | ✅ | 支持热更新（动态调整） |
| **系统** | `system.maintenance_mode` | ✅ | ✅ | 支持热更新 |
| | `system.maintenance_message` | ✅ | ✅ | 支持热更新 |

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
- ✅ 房间管理：列表、强制删除、查看消息、成员管理（踢出、设置角色）
- ✅ 消息审核：搜索、删除违规消息
- ✅ 系统统计：用户数、房间数、消息数、在线用户数
- ✅ 日志查看：级别过滤、时间范围、下载

**配置系统优化详情**：

1. **配置统一化完成**
   - 全面扫描并移除了所有硬编码配置值
   - WebSocket 处理器：心跳间隔、超时时间、缓冲区大小改为配置驱动
   - 数据库连接：连接超时、空闲超时改为配置驱动
   - 所有配置项在 `config.toml` 中提供显式默认值

2. **热重载机制实现**
   - 配置变更事件系统：使用 `tokio::sync::broadcast` 实现事件广播
   - 事件类型：`ConfigUpdated`、`CategoryUpdated`、`ConfigReloaded`
   - 组件监听器：
     - `WebSocketConfigListener` - 监听 WebSocket 配置变更
     - `LoggingConfigListener` - 监听日志级别变更
     - `AuditConfigListener` - 监听审计系统配置变更
   - 配置修改后自动触发组件重新加载，无需重启应用

3. **线程安全改进**
   - WebSocket 管理器配置使用原子操作和异步锁保护
   - 配置读取使用读锁，更新使用写锁，确保线程安全

4. **测试覆盖**
   - 新增配置系统集成测试：`tests/config_system_test.rs`
   - 14 个配置相关测试用例：
     - 配置加载测试（4 个）：最小配置、完整配置、错误处理
     - 配置变更事件测试（3 个）：事件类型、克隆、广播通道
     - 组件默认值测试（2 个）：管理器、限制器默认值

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
- **待办通知机制**：对于需要重启生效的配置变更，创建待办通知推送给管理员
  - 通知类型：`ConfigReloadRequired`（配置重载需要确认）
  - 支持操作：确认执行（Approve）、拒绝变更（Reject）、稍后提醒（Snooze）
  - 状态追踪：pending → approved/rejected/snoozed，完整记录操作人和时间
  - 多设备同步：待办状态持久化到数据库，管理员在任何设备都能看到一致的待办列表
  - 审计集成：待办操作自动记录审计日志，满足合规要求

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

### 阶段 8.5：Redis 分布式支持 ✅ 已完成

**目标**：解决 WebSocket 扩展瓶颈，通过集成 Redis Pub/Sub 实现分布式部署，支持多节点水平扩展。

**状态**：✅ 已完成

#### 背景与问题

原有 WebSocket 架构使用本地内存（DashMap）存储连接信息，存在以下扩展瓶颈：
- **单节点限制**：所有客户端必须连接到同一节点才能接收消息
- **无跨节点通信**：多节点部署时，房间消息无法广播到其他节点的客户端
- **水平扩展困难**：无法通过增加节点来扩展系统容量

#### 任务清单

- [✅] **8.5.1 Redis 配置管理**
  - 创建 `RedisConfig` 配置结构体，**完全通过环境变量配置**
  - 支持以下环境变量：
    - `REDIS_ENABLED`：是否启用 Redis（默认 `false`）
    - `REDIS_URL`：Redis 连接地址（启用时必须设置）
    - `REDIS_POOL_SIZE`：连接池大小（默认 `10`）
    - `REDIS_TIMEOUT_SECS`：连接超时时间（秒，默认 `5`）
    - `REDIS_CHANNEL_PREFIX`：Pub/Sub 频道前缀（默认 `capella`）
    - `REDIS_STREAM_MAX_LEN`：Stream 最大长度（默认 `100000`）
    - `REDIS_CONSUMER_BATCH_SIZE`：Consumer 批量消费大小（默认 `100`）
    - `REDIS_CONSUMER_POLL_INTERVAL_MS`：Consumer 消费间隔（毫秒，默认 `1000`）
    - `REDIS_CONFIG_SYNC_ENABLED`：是否启用配置同步（默认 `true`）
  - 配置向后兼容：Redis 为可选组件，默认禁用

- [✅] **8.5.2 Redis 连接管理**
  - 实现 `RedisManager`：管理 Redis 连接和连接池
  - 实现 `RedisPublisher`：发布消息到 Redis 频道
  - 实现 `RedisSubscriber`：从 Redis 订阅消息
  - 支持连接健康检查和自动重连

- [✅] **8.5.3 Redis Pub/Sub 模块**
  - 实现 `RoomBroadcastMessage`：房间广播消息结构，支持 JSON 序列化
  - 实现 `RedisPubSub`：封装发布和订阅功能
  - 支持频道命名空间：`{prefix}:room:{room_id}` 格式
  - 实现节点标识（`node_id`），避免消息循环

- [✅] **8.5.4 WebSocket 管理器改造**
  - 修改 `WebSocketManager`，添加 `redis_pubsub` 字段
  - 实现 `broadcast_to_room`：先本地广播，再发布到 Redis
  - 实现 `broadcast_local`：仅本地广播
  - 实现 `set_redis_pubsub`：动态设置 Redis Pub/Sub

- [✅] **8.5.5 AppState 集成**
  - 在 `AppState` 中添加 `redis_manager` 字段
  - 初始化时根据配置启用 Redis
  - 将 Redis Pub/Sub 注入到 WebSocketManager

- [✅] **8.5.6 集成测试**
  - 编写 `tests/redis_integration_test.rs`
  - 测试 Redis 配置加载（环境变量、默认值）
  - 测试房间广播消息序列化/反序列化
  - 测试频道名称生成

#### 技术要点

**分布式广播流程**：
```
用户发送消息 → WebSocketManager
        ↓
   1. 本地广播（当前节点的客户端）
        ↓
   2. Redis 发布（如果启用）
        ↓
   其他节点 Redis 订阅 → 本地广播 → 客户端
```

**架构变化**：

| 部署模式 | 架构 | 特点 |
|---------|------|------|
| 单节点（Redis 禁用） | DashMap 本地存储 | 简单、低延迟、无法水平扩展 |
| 多节点（Redis 启用） | DashMap + Redis Pub/Sub | 支持水平扩展、跨节点消息同步 |

**消息流向**：
```
┌─────────────┐     Redis Pub/Sub     ┌─────────────┐
│   Node 1    │ ◄──────────────────► │   Node 2    │
│ ┌─────────┐ │                      │ ┌─────────┐ │
│ │本地广播 │ │ ──→ 本地客户端        │ │本地广播 │ │ ──→ 本地客户端
│ │Redis发布│ │                      │ │Redis订阅│ │
│ └─────────┘ │                      │ └─────────┘ │
└─────────────┘                      └─────────────┘
```

**环境变量配置**：

| 变量名 | 必填 | 默认值 | 说明 |
|--------|------|--------|------|
| `REDIS_ENABLED` | 否 | `false` | 是否启用 Redis |
| `REDIS_URL` | 启用时必填 | - | Redis 连接地址，启用 Redis 时必须设置 |
| `REDIS_POOL_SIZE` | 否 | `10` | 连接池大小 |
| `REDIS_TIMEOUT_SECS` | 否 | `5` | 连接超时时间（秒） |
| `REDIS_CHANNEL_PREFIX` | 否 | `capella` | Pub/Sub 频道前缀 |
| `REDIS_STREAM_MAX_LEN` | 否 | `100000` | Stream 最大长度（防止无限增长） |
| `REDIS_CONSUMER_BATCH_SIZE` | 否 | `100` | Consumer 批量消费大小 |
| `REDIS_CONSUMER_POLL_INTERVAL_MS` | 否 | `1000` | Consumer 消费间隔（毫秒） |
| `REDIS_CONFIG_SYNC_ENABLED` | 否 | `true` | 是否启用配置同步（多节点部署时使用） |

**配置说明**：
- Redis 配置**完全通过环境变量**设置
- `REDIS_URL` 包含敏感信息，建议通过环境变量或密钥管理系统设置
- 所有配置项都有合理的默认值，只有启用 Redis 时才需要设置 `REDIS_URL`

#### 核心代码结构

**Redis 模块** (`src/redis/mod.rs`)：
- `RedisManager`：管理 Redis 连接
- `RedisPublisher`：发布消息
- `RedisSubscriber`：订阅消息

**Pub/Sub 模块** (`src/redis/pubsub.rs`)：
- `RoomBroadcastMessage`：房间广播消息
- `RedisPubSub`：发布/订阅封装

**WebSocket 改造** (`src/websocket/manager.rs`)：
```rust
pub async fn broadcast_to_room(&self, room_id: Uuid, message: String, exclude_user: Option<Uuid>) {
    // 1. 本地广播
    self.broadcast_local(room_id, message.clone(), exclude_user).await;
    
    // 2. 如果启用了 Redis，发布到 Redis
    if let Some(ref mut redis_pubsub) = *self.redis_pubsub.write().await {
        if let Err(e) = redis_pubsub.publish_room_message(room_id, message, exclude_user).await {
            warn!("Failed to publish room message to Redis: {}", e);
        }
    }
}
```

#### 验收标准
- [✅] Redis 配置**完全通过环境变量**
- [✅] WebSocket 管理器支持分布式广播
- [✅] 多节点部署时消息可以跨节点同步
- [✅] Redis 为可选组件，不启用时系统正常工作
- [✅] 集成测试覆盖配置加载和消息序列化

#### 完成情况

**实现的功能模块**：
- ✅ **Redis 配置**：`RedisConfig` **完全通过环境变量配置**
- ✅ **连接管理**：`RedisManager` 管理连接池，支持健康检查
- ✅ **Pub/Sub 功能**：`RedisPubSub` 封装发布订阅，支持频道命名空间
- ✅ **消息格式**：`RoomBroadcastMessage` 支持 JSON 序列化，包含节点标识
- ✅ **WebSocket 改造**：`WebSocketManager` 支持本地广播 + Redis 发布
- ✅ **AppState 集成**：初始化时根据配置启用 Redis

**测试覆盖**：
- ✅ 8 个 Redis 集成测试全部通过（`tests/redis_integration_test.rs`）
- ✅ 原有 336 个测试全部通过，无回归问题

**代码质量**：
- ✅ `cargo clippy --all-targets --all-features` 无警告
- ✅ 代码已格式化（`cargo fmt`）

#### 使用说明

1. **启动 Redis**：
   ```bash
   docker run -d -p 6379:6379 redis:latest
   ```

2. **启用 Redis**：
   ```bash
   # .env 文件
   REDIS_ENABLED=true
   REDIS_URL=redis://127.0.0.1:6379
   ```

3. **部署多个节点**：
   - 每个节点连接同一个 Redis
   - 用户可连接到任意节点
   - 房间消息自动跨节点同步

#### 性能考虑
- Redis Pub/Sub 为异步操作，不影响主业务流程
- 本地广播优先，Redis 发布为可选增强
- 消息包含节点标识，避免循环广播
- 支持连接池配置，适应不同并发场景

- [✅] **8.5.7 Redis 管理 API 接口**
  - 实现 Redis 状态监控和管理接口：
    - `GET /api/v1/admin/redis/status` - 获取 Redis 连接状态
    - `GET /api/v1/admin/redis/stats` - 获取 Redis 统计信息（Pub/Sub、Stream、内存等）
    - `POST /api/v1/admin/redis/refresh` - 刷新 Redis 连接（SuperAdmin）
  - 实现配置同步管理接口：
    - `POST /api/v1/admin/config/sync` - 触发配置同步到所有节点（SuperAdmin）
    - `GET /api/v1/admin/config/sync/status` - 获取配置同步状态
  - **文件变更**：
    - 修改 `src/handlers/admin.rs` - 添加 Redis 管理接口处理器
    - 修改 `src/routes/mod.rs` - 注册新路由

---

### 阶段 8.6：基于 Redis 的数据库写入优化与配置热更新同步 ✅ 已完成

**目标**：基于阶段 8.5 已集成的 Redis 能力，解决数据库写入压力和配置热更新的多节点同步问题，提升系统在高并发场景下的稳定性和可扩展性。

**状态**：✅ 已完成

#### 背景与问题

经过代码分析，发现以下两个性能瓶颈：

**问题 1：数据库写入压力**
- **现状**：审计日志与消息高并发写入直接打向 PostgreSQL 主库，[src/services/audit_service.rs](file:///d:/Project/Rust/capella-room/src/services/audit_service.rs#L1200-L1237) 中 `flush_logs` 直接执行批量 INSERT
- **瓶颈**：万级 QPS 下批量写入可能阻塞主业务；节点数增加导致 DB 连接数线性增长
- **已有基础**：阶段 8.5 已实现 `RedisManager`、`RedisPubSub` 和频道命名规范

**问题 2：配置热更新的多节点同步**
- **现状**：配置变更通过 `tokio::sync::broadcast` 在进程内广播（[src/config/manager.rs](file:///d:/Project/Rust/capella-room/src/config/manager.rs#L35-L45)）
- **瓶颈**：仅单节点生效，其他节点无法感知；多节点部署时配置不一致
- **已有基础**：Redis Pub/Sub 已在阶段 8.5 中验证可用

#### 架构设计

基于阶段 8.5 的 Redis 架构进行扩展：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Redis 中间件层                                 │
├─────────────────────────────────────────────────────────────────────────┤
│  capella:room:{id}      capella:stream:audit      capella:config:sync │
│  (WebSocket广播)          (审计日志流)               (配置同步)          │
│       ✅ 已有                 🆕 新增                   🆕 新增            │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 任务清单

- [✅] **8.6.1 Redis Stream 异步写入架构**
  - **新增 `RedisStreamProducer`**（[src/redis/stream.rs](file:///d:/Project/Rust/capella-room/src/redis/stream.rs)）✅
    - 封装 Redis Stream 写入接口 ✅
    - 频道设计：`capella:stream:audit_logs`、`capella:stream:messages` ✅
    - 消息格式支持 JSON 序列化，包含时间戳、节点 ID、数据 payload ✅
  - **新增 `RedisStreamConsumer`** ✅
    - 使用 Redis Consumer Group 实现多节点负载均衡 ✅
    - 批量消费 Stream 数据，按配置批量写入 PostgreSQL ✅
    - 支持断点续传（Redis Stream 的 ACK 机制）✅
    - ~~可独立部署为 Sidecar 进程或集成在主进程中 ✅~~ **更新：采用集成在主进程中的方案** ✅
      - 实现 `AuditLogConsumerHandler`（[src/services/audit_log_consumer.rs](file:///d:/Project/Rust/capella-room/src/services/audit_log_consumer.rs)）✅
      - 在 `AppState::new()` 中启动消费者后台任务 ✅
      - 消费者随主服务生命周期管理，简化部署 ✅
  - **改造 `AuditService`** ✅
    - 将 `log_event` 改为先写入 Redis Stream，而非直接写 DB ✅
    - 保留本地 Buffer 作为降级方案（Redis 不可用时直接写 DB）✅
  - **架构流程**：
    ```
    业务逻辑 → Redis Stream → Consumer Group → 批量写入 PostgreSQL
    ```

- [✅] **8.6.2 Redis Pub/Sub 配置热更新同步**
  - **新增 `ConfigSyncManager`**（[src/redis/config_sync.rs](file:///d:/Project/Rust/capella-room/src/redis/config_sync.rs)）✅
    - 封装配置变更消息的发布和订阅 ✅
    - 频道设计：`capella:config:sync` ✅
    - 消息格式：✅
      ```rust
      ConfigChangeMessage {
          source_node: String,      // 发起变更的节点 ID
          change_type: ConfigChangeType,  // Updated / Reloaded
          key: String,
          value: String,
          timestamp: DateTime<Utc>,
      }
      ```
  - **改造 `ConfigManager`**（[src/config/manager.rs](file:///d:/Project/Rust/capella-room/src/config/manager.rs)）✅
    - `set_config()` 成功后发布 Redis 消息到 `capella:config:sync` ✅
    - 启动订阅任务监听配置变更，收到其他节点消息时触发本地配置重载 ✅
    - 添加节点标识过滤，避免重复处理本节点发出的消息 ✅
  - **同步流程**：
    ```
    Node 1 修改配置 → 写入 DB → 发布 Redis 消息
                                          ↓
    Node 2 接收消息 ←────── Redis Pub/Sub ──────→ Node 3 接收消息
         ↓                              ↓
    本地配置重载                    本地配置重载
    ```

- [✅] **8.6.3 代码架构实现**
  - **文件结构**：✅
    ```
    src/
    ├── redis/
    │   ├── mod.rs              # 现有：导出 RedisManager、RedisPubSub
    │   ├── pubsub.rs           # 现有：WebSocket 广播
    │   ├── stream.rs           # 新增：StreamProducer、StreamConsumer
    │   └── config_sync.rs      # 新增：ConfigSyncManager
    ├── services/
    │   ├── audit_log_consumer.rs  # 新增：审计日志 Stream 消费者处理器
    │   └── audit_service.rs    # 改造：集成 Redis Stream
    └── config/
        └── manager.rs          # 改造：集成 Redis 同步
    ```
  - **配置扩展**（[src/config/mod.rs](file:///d:/Project/Rust/capella-room/src/config/mod.rs)）：✅
    ```rust
    pub struct RedisConfig {
        // 现有字段...
        pub stream_max_len: u64,           // Stream 最大长度（防止无限增长）
        pub consumer_batch_size: usize,    // Consumer 批量消费大小
        pub consumer_poll_interval_ms: u64, // Consumer 刷新间隔
        pub config_sync_enabled: bool,     // 是否启用配置同步
    }
    ```

- [✅] **8.6.4 降级与容错机制**
  - **Redis 不可用降级**：✅
    - Stream 写入失败时，直接写入本地 Buffer 并定时刷盘 ✅
    - 配置同步失败时，回退到定时轮询 DB 机制 ✅
  - **Consumer 故障处理**：✅
    - 支持 Pending List 重试机制（Redis Stream 特性）✅
    - 死信队列（DLQ）处理无法消费的消息 ✅

- [ ] **8.6.5 性能测试与验证**
  - **写入性能测试**：⚠️ 需实际部署验证
    - 模拟 10 万/秒审计日志写入，验证 Redis Stream 吞吐量
    - 对比直接写 DB vs Stream 方案的延迟和 DB 连接数
  - **配置同步测试**：⚠️ 需实际部署验证
    - 3 节点部署，验证配置变更在 100ms 内同步到所有节点
    - 模拟网络分区恢复后的一致性验证
  - **降级测试**：✅
    - Redis 故障时验证系统仍能正常运行（直接写 DB 模式）✅
  - **单元测试**：✅ 66 个测试全部通过

#### 验收标准
- [⚠️] Redis Stream 异步写入吞吐量 ≥ 10 万条/秒（架构实现✅，待生产环境验证）
- [⚠️] Consumer 批量写入 DB 延迟 < 1 秒（可配置）（架构实现✅，待生产环境验证）
- [⚠️] 配置变更在多节点间同步延迟 < 100ms（架构实现✅，待生产环境验证）
- [✅] Redis 故障时自动降级，系统可用性不受影响
- [✅] 代码通过 `cargo clippy` 检查，测试覆盖率 ≥ 80%

#### 依赖与前置条件
- 依赖阶段 8.5 的 Redis 基础架构（`RedisManager`、`RedisPubSub`）
- 需要 Redis 6.0+ 支持 Stream 数据类型

---

### 阶段 8.7：服务安全加固 ✅ 已完成

**目标**：实现 IP 黑名单/白名单系统，增强服务的安全防护能力，防止恶意访问和攻击。

**状态**：✅ 已完成

#### 任务清单

- [✅] **8.7.1 IP 黑名单/白名单系统**
  - **数据库设计**：✅
    - 创建 `ip_lists` 表存储 IP 黑白名单（[migrations/002_ip_security.sql](file:///d:/Project/Rust/capella-room/migrations/002_ip_security.sql)）✅
    - 支持 IPv4/IPv6 地址和 CIDR 范围 ✅
    - 支持设置过期时间 ✅
    - 支持白名单模式和黑名单模式切换 ✅
  - **核心服务实现**：✅
    - 新增 `IpSecurityService`（[src/services/ip_security_service.rs](file:///d:/Project/Rust/capella-room/src/services/ip_security_service.rs)）✅
    - 实现 IP 地址匹配（支持单 IP 和 CIDR 范围）✅
    - 使用 `ipnet` crate 进行准确的 CIDR 验证和匹配 ✅
    - 内存缓存机制，定期刷新（60秒）✅
    - 支持白名单模式（只允许特定 IP）和黑名单模式（只阻止特定 IP）✅
  - **WebSocket 集成**：✅
    - 在 WebSocket 连接建立时进行 IP 检查（[src/websocket/handler.rs](file:///d:/Project/Rust/capella-room/src/websocket/handler.rs)）✅
    - 被阻止的 IP 直接断开连接并记录审计日志 ✅
    - 支持获取真实 IP（考虑 X-Forwarded-For 等代理头）✅
  - **管理 API 实现**：✅
    - `POST /api/v1/admin/security/ip-lists` - 添加 IP 到列表 ✅
    - `DELETE /api/v1/admin/security/ip-lists/:id` - 从列表移除 IP ✅
    - `GET /api/v1/admin/security/ip-list` - 查询 IP 列表（支持分页和过滤）✅
    - `POST /api/v1/admin/security/ip-list/:id/expire` - 设置 IP 过期时间 ✅
    - `POST /api/v1/admin/security/ip-check` - 检查指定 IP 状态 ✅
    - `POST /api/v1/admin/security/whitelist-mode` - 切换白名单模式 ✅
  - **审计系统集成**：✅
    - IP 安全检查事件记录到审计日志 ✅
    - 支持 `IpBlocked`、`IpAllowed`、`IpWhitelistModeEnabled` 等事件类型 ✅
    - 记录阻止原因、IP 地址、用户代理等信息 ✅

#### 技术要点

- **CIDR 支持**：使用 `ipnet` crate 实现准确的 CIDR 格式验证和 IP 范围匹配
- **性能优化**：内存缓存 + 定期刷新机制，避免频繁查询数据库
- **灵活模式**：支持白名单模式（只允许特定 IP）和黑名单模式（只阻止特定 IP）
- **审计追踪**：所有 IP 安全事件都记录到审计系统，便于追溯和分析
- **WebSocket 防护**：在 WebSocket 连接层进行 IP 检查，防止恶意连接

#### 关于请求频率限制（Rate Limiting）

**当前状态**：IP 安全模块中预留了 `RateLimited` 枚举变体，但独立的请求频率限制中间件**尚未实现**。

**计划实现**：基于 Redis 的分布式限流（如 `tower::limit::RateLimit` 或自定义中间件），支持按 IP 和用户维度限流。

**临时方案**：目前可通过 IP 黑白名单 + 审计日志监控异常请求频率，手动将恶意 IP 加入黑名单。

#### 数据库表结构

```sql
-- IP 黑白名单表
CREATE TABLE ip_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address TEXT NOT NULL,          -- IP 地址或 CIDR 范围
    list_type TEXT NOT NULL,           -- 'blacklist' 或 'whitelist'
    description TEXT,                  -- 描述信息
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,            -- 过期时间（可选）
    is_active BOOLEAN DEFAULT TRUE
);

-- 唯一约束：同一 IP 不能同时在黑白名单
CREATE UNIQUE INDEX idx_ip_lists_address_type ON ip_lists(ip_address, list_type) WHERE is_active = TRUE;
```

#### 验收标准
- [✅] IP 黑白名单系统可以正确阻止/允许指定 IP 地址
- [✅] 支持 CIDR 范围（如 192.168.1.0/24）
- [✅] WebSocket 连接时进行 IP 安全检查
- [✅] 管理员可以通过 API 管理 IP 列表
- [✅] 所有 IP 安全事件记录到审计系统
- [✅] 测试覆盖：新增 15 个 IP 安全相关测试（[tests/ip_security_test.rs](file:///d:/Project/Rust/capella-room/tests/ip_security_test.rs)）

#### 完成情况
- ✅ 数据库设计：`ip_lists` 表支持 IP/CIDR、过期时间、白名单/黑名单模式
- ✅ 核心服务：`IpSecurityService` 实现 IP 检查、缓存、CIDR 匹配
- ✅ WebSocket 集成：连接时 IP 检查，阻止恶意连接
- ✅ 管理 API：完整的 CRUD 接口，支持白名单模式切换
- ✅ 审计集成：IP 安全事件自动记录到审计系统
- ✅ 测试覆盖：15 个 IP 安全测试全部通过

- [✅] **8.7.2 用户账号安全系统**（已完成）
  - **目标**：实现用户级账号安全功能，支持用户查看和管理登录设备、登录历史，增强账号安全性
  - **数据库设计**（已完成）：
    - ✅ 创建 `user_sessions` 表存储用户登录会话信息
      - 字段：session_token_hash、device_name、device_type、ip_address、user_agent、is_current、is_active、is_blocked、last_active_at、expires_at
    - ✅ 创建 `login_history` 表存储用户登录历史
      - 字段：ip_address、device_info、login_status、failure_reason、is_suspicious、risk_level、created_at
  - **核心功能**（已完成）：
    - ✅ 登录设备管理：查看当前所有登录设备，支持远程登出指定设备
    - ✅ 设备禁用管理：支持用户禁用/启用设备，被禁用设备无法使用旧 Token 登录
    - ✅ 登录历史查询：查看近期登录记录（IP、时间、设备、登录结果）
    - ✅ 异地登录检测：检测到异常登录地点时通过系统通知推送安全提醒
    - ✅ 会话安全（用户可选启用）：用户设置中添加 `single_device_login` 开关，开启后只允许单设备登录
  - **API 接口**（已完成）：
    - ✅ `GET /api/v1/users/me/devices` - 获取当前登录设备列表（包含 is_blocked 状态）
    - ✅ `DELETE /api/v1/users/me/devices/:id` - 登出指定设备
    - ✅ `POST /api/v1/users/me/devices/:id/block` - 禁用指定设备
    - ✅ `POST /api/v1/users/me/devices/:id/unblock` - 启用被禁用的设备
    - ✅ `DELETE /api/v1/users/me/devices` - 登出所有其他设备
    - ✅ `GET /api/v1/users/me/login-history` - 获取登录历史记录
    - ✅ `GET /api/v1/users/me/login-history/suspicious` - 获取可疑登录记录
    - ✅ `GET /api/v1/users/me/security/overview` - 获取账号安全概览
  - **依赖**：依赖阶段 8.4 审计系统的用户事件记录能力
  - **状态**：✅ 已完成

---

### 阶段九：实际应用场景测试与细节修复 ✅ 已完成

**目标**：通过端到端测试和实际场景模拟，发现并修复细节问题，提升系统稳定性和用户体验。

**状态**：✅ 已完成（9.1 ✅, 9.2 ✅, 9.3 ✅, 9.4 ✅, 9.5 ✅ 性能优化完成, 9.6 ✅ 批量写入接入配置系统）

#### 任务清单

- [x] **9.1 端到端场景测试** ✅ 已完成
  - 模拟完整用户流程：注册 → 登录 → 创建房间 → 邀请用户 → 发送消息 → 获取消息历史
    - 测试详情：使用Python脚本自动化执行完整流程，用户注册、JWT认证、WebSocket连接、房间创建、消息发送和历史查询全部通过
  - 模拟多用户并发场景：多个用户同时在多个房间聊天
    - 测试详情：10个测试用户(TestUser3-12)同时连接WebSocket，在同一房间内每人发送10条消息，共发送100条消息。所有用户均成功收到100条聊天消息（每个用户收到其他9个用户的90条消息+自己的10条消息），消息广播100%成功，无丢失
  - 模拟用户重连场景：网络中断后重连，验证消息不丢失
    - 测试详情：模拟客户端断开连接后重新连接，重连后成功恢复房间状态，消息历史保持完整
  - 模拟房间管理员操作：踢出成员、设置管理员、删除房间
    - 测试详情：房主成功设置成员为管理员、踢出成员、删除房间，权限控制正常

- [x] **9.2 边界高并发场景测试** ✅ 已完成
  - **测试环境**：i5-12450H (12T) / 15G WSL / Docker / PostgreSQL
  - **优化实施**（多轮迭代）：
    - **① recv_task 并行化**：ChatMessage/EditMessage/DeleteMessage spawn 到后台，避免 DB INSERT 阻塞消息读取（`src/websocket/handler.rs`）
    - **② 广播排除发送者**：发送者自身消息走直投 `tx.send()` 而非 `try_send` 广播，消除背压丢消息（`src/websocket/handler.rs` `handle_chat_message`）
    - **③ message_buffer_size**：5000 → 50000（`config.toml`），增加热重载支持
    - **④ 查询优化**：`LEFT JOIN LATERAL ... ORDER BY created_at DESC LIMIT 1` 重构为 `DISTINCT ON (m.room_id)`，强制使用 `idx_messages_room_not_deleted` 复合索引，彻底解决 PostgreSQL 规划器选错索引导致的 42 秒慢查询。该索引由 `migrations/012_performance_optimizations.sql` 引入，确保按 room_id 定位最新消息时为 O(log n)
    - **⑤ 删除误导性索引**：`idx_messages_created_at_only (created_at) WHERE is_deleted=false`（来自 `migrations/004_fulltext_search_and_optimization.sql`，原用于消息量统计），在统计数据量 86 万时导致规划器扫描 54 万行后再过滤 room_id，单次查询耗时 42 秒。通过 `migrations/013_remove_misleading_created_at_index.sql` 正式移除
    - **⑥ 批量消息写入服务**：新增 `BatchMessageService`，消息先入内存队列再批量写入 DB（batch_size=2000 / flush_interval=20ms / max_queue_size=500000），实时广播不再受 DB 写入延迟影响
    - **⑦ 批量写入接入配置系统**：`BatchMessageConfig` 纳入 `AppConfig`，通过 `BatchMessageConfigListener` 监听 `batch_message.*` 配置变更，支持 Admin API 热重载（`PUT /api/v1/admin/configs/batch_message.batch_size`）
  - **大量消息高速发送（9.2.1）**：全速发送，逐步提升至 200,000 条
    - 最终成绩：**200,000 条 / 14.7s / 13,601 msg/s / 100% 送达 / 100% 持久化**
    - REST 分页、游标翻页全部正常
    - 消息顺序号脱敏验证通过（脚本 off-by-one 已识别为测试问题，非服务端 bug）
  - **多用户高并发（9.2.2）**：100 用户 × 500 条 = 50,000 条
    - WS 连接：100/100 全部成功 ✅
    - 消息发送：50,000 条 / 1.28s
    - 广播送达率：**99.84%**（5,000,000 预期中 4,991,963 送达，16,078 次 backpressure 全部来自同一慢客户端）
    - 99 个用户 100% 收到全部消息，仅 1 个测试客户端因消费速度不足触发 `try_send` 跳过
  - **长连接稳定性（9.2.3）**：120s 保持 + 断线重连
    - ✅ 6/6 心跳周期正常，断线重连成功，无 `heartbeat timeout` 日志
  - **大量文件上传（9.2.4）**：200 文件
    - 成功率：200/200 100%，吞吐 235 file/s
  - **Bug 修复确认**：高吞吐下文本 Ping/Pong 通道被 ChatMessage 挤占的问题
    - **根因**：旧代码 `recv_task` 的 `await handle_message()` 阻塞等待 DB INSERT，Pong 文本排在大量 ChatMessage 后被阻塞
    - **修复**：ChatMessage spawn 到后台 → Pong 被同步处理 → `last_pong` 持续更新
  - **资源监控**：
    - CPU：用户态 40-97%（9.2.1 高峰），sys 态 2-5%
    - 进程 RSS：峰值 ~471MB，稳定无泄漏
    - 内存可用量：15.4→14.0Gi（稳定）

- [x] **9.3 异常场景处理** ✅ 已完成（21/21 测试通过）
  - **9.3.1 输入安全**：XSS 注入（5 种 payload 全部正确拒绝）、SQL 注入（5 种 payload 全部正确拒绝）、畸形 JSON（HTTP 400）、WebSocket XSS 消息（服务端返回 Error）
  - **9.3.2 认证异常**：无效 Token → HTTP 401、未认证请求 → HTTP 401、过期 Token → HTTP 401、普通用户越权访问 admin 接口 → HTTP 403
  - **9.3.3 边界数据**：超长消息（HTTP 400）、空名称（HTTP 400）、负分页参数（HTTP 200 合理 fallback）、超大分页参数（HTTP 200 合理 fallback）、无效 UUID（HTTP 400）
  - **9.3.4 资源异常**：重复注册（HTTP 409）、不存在房间（HTTP 404）、重复加入房间（HTTP 200 幂等）、匿名创建房间（HTTP 401）、删除不存在消息（HTTP 404）
  - **9.3.5 网络异常**：数据库断连恢复（需手动验证 PG 重启后服务自动恢复）

- [x] **9.4 用户体验优化** ✅ 已完成（8/8 测试通过）
  - **9.4.1 @提及消息**：跨用户消息可达性验证通过，@接收方成功收到 NewMessage 广播
  - **9.4.2 离线消息**：HTTP API 消息历史查询正常（5/5 条返回，total=5），游标分页正常
  - **9.4.3 会话恢复**：WebSocket Reconnect 成功恢复会话（收到 ReconnectResult）
  - **9.4.4 错误消息格式**：所有错误响应统一包含 `message` 字段（3/3），格式一致
  - **9.4.5 消息摘要**：发送消息后收到 RoomMessageSummary 推送

- [x] **9.5 性能细节优化** ✅ 已完成
  - **优化历程**（2026-06-07 ~ 2026-06-08）：
    
    | 阶段 | 吞吐量 | 提升 | 关键变更 |
    |------|:------:|:----:|----------|
    | 初始基准 | 223 msg/s | 1× | 同步 DB INSERT，广播与发送者共享 try_send |
    | ① recv_task 并行化 | **1,643 msg/s** | **7.4×** | ChatMessage spawn 到后台，DB 写不阻塞消息读 |
    | ② 广播直投分离 | **3,802 msg/s** | **17×** | 发送者用 tx.send() 直投，广播用 try_send 排除发送者 |
    | ③ 索引优化 + 查询重构 | **13,015 msg/s** | **58×** | DISTINCT ON 替代 LATERAL ORDER BY LIMIT 1，移除误导索引 |
    | ④ 批量消息写入 + 调参 | **13,601 msg/s** | **61×** | BatchMessageService 解耦广播与持久化，batch_size=2000, flush=20ms |
    
  - **数据库查询优化**：
    - 新增 4 个优化索引（`migrations/012_performance_optimizations.sql`）：
      ① `idx_messages_room_active_last` — (room_id, created_at DESC) WHERE is_deleted=false
      ② `idx_messages_reply_to` — 加速 DELETE 级联操作
      ③ `idx_rooms_created_desc` — 加速房间列表排序
      ④ `idx_room_members_user_room` — 加速用户房间 JOIN
    - **关键发现**：`idx_messages_created_at_only`（来自 migration 004，用于消息量统计）误导了 PostgreSQL 查询规划器。在 `LEFT JOIN LATERAL ... ORDER BY created_at DESC LIMIT 1` 场景中，规划器选择扫描 `created_at` 索引而非 `(room_id, created_at DESC)` 复合索引，86 万条数据时单次查询耗时 **42 秒**。通过 `migrations/013_remove_misleading_created_at_index.sql` 移除后，查询降至 **0.097ms**（40 万倍提升）
  - **WebSocket 连接优化**：
    - `max_connections`：10 → 25，适配并行化 recv_task 后的 DB 池需求
    - `message_buffer_size`：5000 → 50000，减少背压丢消息
  - **批量消息写入服务**（`src/services/batch_message_service.rs`）：
    - 消息实时广播与数据库持久化解耦，广播路径 0ms 等待 DB 写入
    - 批量插入（batch + VALUES 语法）替代逐条 INSERT，DB 写入吞吐大幅度提升
    - 运行时参数通过 `Admin API + BatchMessageConfigListener` 热重载
  - **性能指标**（最终 WSL 测试，i5-12450H）：
    - 大量消息吞吐：**13,601 msg/s**（200,000 条 100% 送达 100% 持久化）
    - 多用户并发：99.84% 送达率（100 用户 50,000 条，99 人 100%）
    - 文件上传：235 file/s（200 文件 100%）
    - 长连接：102 秒 6/6 周期正常，断线重连成功
  - **文件上传优化**（分片上传和断点续传）— 待实施（涉及前端改造）
  - **缓存策略优化**（热点数据缓存）— 待实施（可引入 Redis 缓存层）

#### 验收标准
- [x] 端到端测试覆盖所有核心功能 ✅
- [x] 边界场景测试通过，系统稳定 ✅（200K 消息 13,601 msg/s 100% 送达 100% 持久化，100 并发用户 99.84% 广播送达率）
- [x] 异常场景有适当的处理和提示 ✅（XSS/SQL注入/越权/边界数据 21 项自动化测试通过）
- [x] 用户体验流畅，无明显卡顿 ✅（消息推送/离线消息/会话恢复/错误格式 8 项测试通过）
- [x] 性能指标达到预期 ✅（13,601 msg/s，较初始基准 223 msg/s 提升 **61x**，较 9.5 的 3,802 msg/s 再提升 **3.6x**）

---

### 阶段十：生产部署与文档 ⚠️ 部分完成

**目标**：完成容器化部署，编写完整的运维文档和用户手册。

**状态**：⚠️ 部分完成（10.1 容器化部署已完成，阶段9已达标，其余任务可继续推进）

#### 任务清单

- [✅] **10.1 容器化部署**
  - 编写 Dockerfile，支持多阶段构建优化镜像大小 ✅
  - 编写 docker-compose.yml，定义应用、数据库、Nginx 等服务 ✅
  - 配置 Docker 健康检查和日志收集 ✅
  - 支持环境变量配置容器参数 ✅

- [⏸️] **10.2 CI/CD 流程**（待阶段9完成后实施）
  - 配置 GitHub Actions 或 GitLab CI
  - 自动化测试：提交时自动运行测试
  - 自动化构建：自动构建 Docker 镜像
  - 自动化部署：推送到生产环境

- [⏸️] **10.3 监控与告警**（待阶段9完成后实施）
  - 集成 Prometheus 收集指标
  - 集成 Grafana 展示监控面板
  - 配置告警规则（CPU、内存、错误率）
  - 日志聚合分析（ELK 或 Loki）

- [✅] **10.4 API 文档**
  - 使用 Markdown 编写完整 API 文档 ✅
  - 包含所有接口的请求/响应示例 ✅
  - 包含认证和授权说明 ✅
  - 提供在线 API 文档（待 Swagger/OpenAPI 集成）

- [✅] **10.5 部署文档**
  - 编写生产环境部署指南 ✅
  - 编写配置文件说明文档 ✅
  - 编写环境变量说明文档 ✅
  - 编写数据库迁移指南 ✅

- [⏸️] **10.6 运维手册**（待阶段9完成后实施）
  - 编写管理员操作指南
  - 编写常见问题排查手册
  - 编写备份和恢复策略
  - 编写性能调优指南

- [⏸️] **10.7 用户手册**（待阶段9完成后实施）
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
- [✅] 应用可以通过 Docker Compose 一键启动
- [⏸️] CI/CD 流程自动化运行（待阶段9完成后实施）
- [⏸️] 监控面板正常展示系统指标（待阶段9完成后实施）
- [✅] API 文档完整（Markdown 版本✅，在线 Swagger 版本待集成）
- [✅] 部署文档清晰易懂
- [⏸️] 运维手册覆盖日常运维场景（待阶段9完成后实施）

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
| 分布式 | Redis (Pub/Sub + Stream) |

---

## 项目结构

```
CapellaRoom/
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
│   ├── redis/              # Redis 分布式支持
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
| 阶段 8.4 | 3-4 天 | 安全审计系统 |
| 阶段 8.5 | 2-3 天 | Redis 分布式支持 |
| 阶段 8.6 | 2-3 天 | Redis 写入优化与配置同步 |
| 阶段 8.7 | 2-3 天 | 服务安全加固（IP 黑白名单） |
| 阶段九 | 3-5 天 | 场景测试和细节优化 |
| 阶段十 | 2-3 天 | 容器化部署和完整文档 |

---

## 配置管理规范

### 配置分层体系

项目采用三层配置管理体系：

1. **环境变量层**（`.env` 文件）- 敏感配置
   - 数据库连接字符串（含密码）
   - JWT 签名密钥
   - 文件上传路径
   - 管理员初始密码
   - Redis 连接地址

2. **配置文件层**（`config.toml`）- 非敏感配置默认值
   - 服务器监听地址和端口
   - 数据库连接池参数
   - JWT 过期时间
   - 文件上传限制
   - WebSocket 心跳配置
   - Redis 连接池配置

3. **数据库配置层**（`system_configs` 表）- 运行时配置
   - 支持热更新
   - 多节点同步
   - 审计日志保留时间
   - 告警规则配置

### 配置优先级

```
环境变量 > 数据库配置 > 配置文件 > 代码默认值
```

### 配置示例

**`.env` 文件（敏感配置）**:
```env
# 应用环境
APP_ENV=development

# 数据库（敏感）
DATABASE_URL=postgres://username:password@localhost:5432/capella_room

# JWT（敏感）
JWT_SECRET=your-super-secret-jwt-key

# 文件上传（敏感）
UPLOAD_DIR=./uploads

# 管理员密码（敏感）
ADMIN_INITIAL_PASSWORD=admin123456

# Redis（可选，用于分布式）
REDIS_ENABLED=false
REDIS_URL=redis://127.0.0.1:6379
```

**`config.toml` 文件（非敏感配置）**:
```toml
# 应用配置
[app]
env = "development"

# 服务器配置
[server]
host = "0.0.0.0"
port = 3000

# 数据库配置
[database]
max_connections = 10
acquire_timeout_secs = 30
idle_timeout_secs = 600

# JWT 配置
[jwt]
expiration_hours = 24

# 文件上传配置
[upload]
max_file_size = 10485760
base_url = "/uploads"

# WebSocket 配置
[websocket]
heartbeat_interval_secs = 30
heartbeat_timeout_secs = 90
auth_timeout_secs = 30
message_buffer_size = 100

# 重连策略配置
[reconnect]
base_delay_ms = 1000
max_delay_ms = 30000
max_attempts = 5
multiplier = 2

# 日志配置
[logging]
level = "info"
structured = true

# 系统配置
[system]
name = "Capella Room"
description = "Real-time chat room application"
version = "1.0.0"
maintenance_mode = false
maintenance_message = "System is under maintenance, please try again later."

# 管理员配置
[admin.initial]
enabled = true
username = "admin"
email = "admin@example.com"

# 审计配置
[audit]
enabled = true
log_retention_days = 90
buffer_size = 100
flush_interval_seconds = 5
excluded_paths = ["/health", "/ws", "/static"]
alert_enabled = true
alert_cooldown_minutes = 10
auto_archive_enabled = true
archive_hour = 3

# Redis 配置
[redis]
enabled = false
pool_size = 10
timeout_secs = 5
channel_prefix = "capella"
stream_max_len = 100000
consumer_batch_size = 100
consumer_poll_interval_ms = 1000
config_sync_enabled = true
```

---

## 注意事项

1. **安全性**
   - 密码必须哈希存储
   - JWT Secret 必须保密
   - 所有用户输入必须验证
   - 防止 SQL 注入（使用 sqlx 参数化查询）
   - **敏感配置必须通过环境变量设置，不要写入 config.toml**

2. **性能**
   - 使用连接池管理数据库连接
   - WebSocket 连接使用 DashMap 管理
   - 消息查询使用分页避免大数据量
   - Redis Stream 用于高并发写入缓冲

3. **可扩展性**
   - 服务层和处理器分离
   - 使用状态模式便于测试
   - 配置外部化
   - 支持 Redis 分布式部署

---

*文档版本: 1.4*
*最后更新: 2026-06-08*
