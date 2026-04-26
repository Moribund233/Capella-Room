# Seredeli Room

一个基于 **Axum + WebSocket + PostgreSQL** 构建的实时聊天室应用。

## 项目简介

Seredeli Room 是一个高性能、可扩展的实时聊天系统，支持多房间聊天、用户认证、消息持久化等功能。

### 技术栈

| 类别 | 技术 |
|------|------|
| Web 框架 | [Axum](https://github.com/tokio-rs/axum) |
| 异步运行时 | [Tokio](https://tokio.rs/) |
| 数据库 | [PostgreSQL](https://www.postgresql.org/) + [sqlx](https://github.com/launchbadge/sqlx) |
| WebSocket | axum::extract::ws |
| 认证 | JWT + argon2 |
| 配置管理 | config + dotenvy |
| 日志 | tracing |
| 验证 | validator |
| 分布式 | Redis (Pub/Sub + Stream) |

## 项目结构

```
SeredeliRoom/
├── Cargo.toml              # Rust项目配置和依赖
├── config.toml             # 应用配置文件（三层配置体系）
├── .env.example            # 环境变量示例
├── .env.development        # 开发环境配置
├── README.md               # 项目文档
├── docs/
│   └── development-roadmap.md  # 详细开发路线文档
├── migrations/             # 数据库迁移脚本
│   └── 001_init.sql        # 统一的数据库结构
├── src/
│   ├── main.rs             # 应用入口
│   ├── lib.rs              # 库模块导出
│   ├── config/             # 配置管理
│   │   ├── mod.rs          # 配置结构定义
│   │   ├── loader.rs       # 配置加载器
│   │   ├── manager.rs      # 配置管理器（热更新）
│   │   └── listener.rs     # 配置变更监听器
│   ├── db/                 # 数据库连接池
│   ├── error/              # 错误处理
│   ├── handlers/           # HTTP请求处理器
│   │   ├── admin.rs        # 管理员接口
│   │   ├── auth.rs         # 认证接口
│   │   ├── file.rs         # 文件上传接口
│   │   ├── message.rs      # 消息接口
│   │   ├── room.rs         # 聊天室接口
│   │   └── user.rs         # 用户接口
│   ├── middleware/         # 中间件
│   │   ├── admin.rs        # 管理员认证中间件
│   │   ├── audit.rs        # 审计中间件
│   │   └── auth.rs         # 认证中间件
│   ├── models/             # 数据模型
│   │   ├── file.rs         # 文件模型
│   │   ├── user.rs         # 用户模型
│   │   ├── room.rs         # 聊天室模型
│   │   └── message.rs      # 消息模型
│   ├── redis/              # Redis分布式支持
│   │   ├── mod.rs          # Redis管理器
│   │   ├── pubsub.rs       # Pub/Sub消息广播
│   │   ├── stream.rs       # Stream数据流
│   │   └── config_sync.rs  # 配置同步
│   ├── routes/             # 路由配置
│   ├── services/           # 业务逻辑服务层
│   │   ├── auth_service.rs
│   │   ├── file_service.rs # 文件服务
│   │   ├── message_service.rs
│   │   ├── room_service.rs
│   │   ├── user_service.rs
│   │   ├── audit_service.rs # 审计服务
│   │   └── notification_service.rs # 通知服务
│   ├── state/              # 应用状态
│   ├── utils/              # 工具函数
│   └── websocket/          # WebSocket处理
│       ├── handler.rs
│       ├── manager.rs
│       └── protocol.rs     # WebSocket消息协议
└── tests/                  # 集成测试
    ├── config_system_test.rs
    ├── phase1_infrastructure_test.rs
    ├── phase2_authentication_test.rs
    ├── phase3_room_management_test.rs
    ├── phase4_websocket_test.rs
    ├── phase4_notification_system_test.rs
    ├── phase5_messaging_test.rs
    ├── phase6_user_features_test.rs
    ├── phase6_extra_features_test.rs
    ├── phase6_5_file_upload_test.rs
    ├── phase6_reply_message_test.rs
    ├── phase8_admin_system_test.rs
    ├── phase8_4_alert_system_test.rs
    ├── phase8_4_audit_system_test.rs
    ├── phase8_4_config_performance_test.rs
    ├── redis_integration_test.rs
    └── websocket_test.rs
```

## 开发阶段

### 阶段一：基础架构搭建 ✅ 已完成

- [✅] **1.1 配置管理** - 实现从环境变量加载配置，支持多环境配置
- [✅] **1.2 数据库连接** - 配置 PostgreSQL 连接池，实现数据库迁移
- [✅] **1.3 错误处理** - 完善错误类型定义，实现统一错误响应格式
- [✅] **1.4 项目启动** - 完成应用启动逻辑，支持优雅关闭

**验收标准**：
- ✅ 应用可以正常启动并连接数据库
- ✅ 数据库迁移可以自动执行
- ✅ 日志系统正常工作
- ✅ 健康检查端点返回 200

### 阶段二：用户认证系统 ✅ 已完成

- [✅] **2.1 用户模型** - 完善用户数据模型和验证（用户名、邮箱、密码强度）
- [✅] **2.2 密码安全** - 集成 argon2 进行密码哈希，实现密码强度验证
- [✅] **2.3 JWT 认证** - 实现 Token 生成、验证和刷新机制
- [✅] **2.4 认证接口** - 实现注册、登录、刷新 Token 接口
- [✅] **2.5 认证中间件** - 实现 JWT 认证中间件，保护受保护接口

**验收标准**：
- ✅ 用户可以正常注册账号
- ✅ 用户可以使用邮箱和密码登录
- ✅ JWT Token 可以正确生成和验证
- ✅ 受保护的接口需要有效 Token
- ✅ Token 过期后可以刷新

**测试覆盖**：26 个阶段二功能测试全部通过（`tests/phase2_authentication_test.rs`）

### 阶段三：聊天室管理 ✅ 已完成

- [✅] **3.1 聊天室模型** - 实现聊天室和成员模型，成员角色系统（Owner/Admin/Member）
- [✅] **3.2 聊天室接口** - 创建、获取列表（分页/搜索）、详情、更新、删除接口
- [✅] **3.3 成员管理** - 加入、离开、获取成员列表、踢出成员、设置角色接口
- [✅] **3.4 权限控制** - Owner/Admin/Member 三级权限系统，私有房间访问控制

**验收标准**：
- ✅ 用户可以创建聊天室
- ✅ 用户可以浏览公开聊天室列表
- ✅ 用户可以加入/离开聊天室
- ✅ 聊天室成员角色系统正常工作
- ✅ 权限控制正确生效

**测试覆盖**：7 个阶段三功能测试全部通过（`tests/phase3_room_management_test.rs`）

### 阶段四：WebSocket 实时通信 ✅ 已完成

- [✅] **4.1 WebSocket 管理器** - 管理连接和房间订阅，支持用户连接注册、断开处理
- [✅] **4.2 WebSocket 处理器** - 实现消息收发循环，支持连接升级、认证、断开清理
- [✅] **4.3 消息协议** - 定义 WebSocket 消息格式，支持认证、心跳、房间管理、消息通信等
- [✅] **4.4 房间广播** - 实现房间消息广播、单播、在线用户列表同步
- [✅] **4.5 心跳机制** - 服务端每 30 秒发送 Ping，客户端回复 Pong，90 秒超时检测

**验收标准**：
- ✅ 客户端可以建立 WebSocket 连接
- ✅ 用户可以加入/离开房间
- ✅ 消息可以实时广播到房间所有成员
- ✅ 心跳机制正常工作，超时连接被清理

**测试覆盖**：20 个 WebSocket 测试全部通过（`tests/phase4_websocket_test.rs`）

### 阶段五：消息系统 ✅ 已完成

- [✅] **5.1 消息模型** - 完善 Message 模型，支持 Text/Image/File/System 类型，支持回复功能
- [✅] **5.2 消息存储** - 实现消息持久化到数据库，WebSocket 消息自动存储，软删除机制
- [✅] **5.3 消息查询** - 获取聊天室消息历史（游标分页）、搜索消息（模糊搜索）、离线消息获取
- [✅] **5.4 消息接口** - 实现获取历史、搜索、删除消息的 HTTP API 接口

**验收标准**：
- ✅ 消息可以正确存储到数据库
- ✅ 可以获取聊天室历史消息
- ✅ 消息分页加载正常工作
- ✅ 可以搜索消息内容
- ✅ 消息可以软删除

**测试覆盖**：14 个阶段五功能测试全部通过（`tests/phase5_messaging_test.rs`）

### 阶段六：用户功能完善 ✅ 已完成

- [✅] **6.1 用户资料** - 获取、更新用户信息，头像 URL 支持
- [✅] **6.2 用户状态** - 在线状态管理（Online/Offline/Away），WebSocket 实时更新
- [✅] **6.3 用户列表** - 分页、搜索用户，显示在线状态

**验收标准**：
- ✅ 用户可以查看和修改个人资料
- ✅ 用户在线状态实时更新
- ✅ 可以浏览其他用户信息

**测试覆盖**：17 个阶段六功能测试全部通过（`tests/phase6_user_features_test.rs`）

#### 阶段 6 扩展功能（额外开发）✅ 已完成

**已完成：**

- [✅] **房间响应增强**：`RoomResponse` 添加 `updated_at` 字段
- [✅] **最近房间列表**：新增 `GET /api/v1/rooms/recent` 接口
- [✅] **消息编辑功能** - 支持编辑消息并记录编辑历史
- [✅] **消息全文搜索** - PostgreSQL tsvector 全文搜索优化

- [✅] **消息回复功能** - 支持回复特定消息并显示引用上下文

**测试覆盖**：16 个阶段 6 扩展功能测试全部通过（`tests/phase6_extra_features_test.rs`），13 个消息回复测试全部通过（`tests/phase6_reply_message_test.rs`）

---

### 阶段 6.5：文件上传与资源管理 ✅ 已完成

- [✅] **数据库设计** - `file_resources` 表，支持文件分类、用途、关联关系
- [✅] **存储架构** - 按 `uploads/{category}/{year}/{month}/{uuid}.{ext}` 结构存储
- [✅] **配置管理** - `UPLOAD_DIR` 环境变量配置
- [✅] **API 接口** - 通用上传、图片上传、头像上传、文件查询、删除
- [✅] **安全特性** - JWT 认证、文件类型验证、大小限制、权限控制

**验收标准**：
- ✅ 支持图片和文件消息发送
- ✅ 文件分类存储结构清晰
- ✅ 文件访问有适当的权限控制
- ✅ 文件去重机制正常工作

**测试覆盖**：13 个阶段 6.5 功能测试全部通过（`tests/phase6_5_file_upload_test.rs`）

### 阶段七：测试与优化 ✅ 已完成

**状态**：已完成

- [✅] **7.1 单元测试** - 45 个单元测试全部通过
- [✅] **7.2 集成测试** - 9 个端到端集成测试全部通过
- [✅] **7.3 WebSocket 测试** - 37 个 WebSocket 相关测试全部通过
- [✅] **7.4 性能优化** - 数据库查询优化、连接池优化
- [✅] **7.5 代码质量** - Clippy 检查无警告、代码已格式化

**测试统计**：

| 测试类别 | 测试数量 | 状态 |
|---------|---------|------|
| 单元测试 | 53 | ✅ 通过 |
| 阶段 1-8 功能测试 | 173 | ✅ 通过 |
| 端到端集成测试 | 9 | ✅ 通过 |
| WebSocket 场景测试 | 17 | ✅ 通过 |
| **总计** | **252** | **✅ 全部通过** |

**主要优化点**：
- AppState 参数完善，集成 MetricsCollector
- WebSocket Sender 类型统一
- 异步发送操作完善，消除 Clippy 警告
- 消息内容安全增强，新增 XSS 检测
- 健康检查端点标准化，返回结构化 JSON
- API 版本信息标准化，统一响应格式
- HTTP 方法语义化（`POST /leave` → `DELETE /leave`）

### 阶段八：部署与运维 ✅ 已完成

- [✅] **8.1 三层配置管理体系** - 环境变量 + config.toml + 数据库配置，支持热更新
- [✅] **8.2 管理员系统** - SuperAdmin/Admin/User 三级角色，权限控制
- [✅] **8.3 运维管理 API** - 用户管理、配置管理、房间管理、消息审核、系统统计、日志查看

**验收标准**：
- ✅ 三层配置管理体系正常工作，支持配置热更新
- ✅ 管理员系统完整，支持 SuperAdmin 和 Admin 角色
- ✅ 所有运维管理 API 可用并通过测试
- ✅ 日志系统支持分级和动态调整

**测试覆盖**：13 个阶段八功能测试全部通过（`tests/phase8_admin_system_test.rs`）

### 阶段 8.4：安全审计系统 ✅ 已完成

- [✅] **8.4.1 审计日志系统** - `audit_logs` 表记录所有关键操作，支持异步批量写入
- [✅] **8.4.2 安全告警系统** - 异常登录、暴力破解、越权访问检测，实时告警通知
- [✅] **8.4.3 审计中间件** - 自动记录 HTTP 请求审计日志，支持敏感操作标记
- [✅] **8.4.4 管理员审计 API** - 日志查询、导出、告警管理、规则配置
- [✅] **8.4.5 审计配置管理** - 支持热更新，可配置保留时间、事件类型开关

**验收标准**：
- ✅ 所有关键操作记录审计日志
- ✅ 安全事件实时检测并告警
- ✅ 审计日志支持查询、导出、统计
- ✅ 告警规则可动态配置

**测试覆盖**：38 个阶段 8.4 测试全部通过（`tests/phase8_4_*_test.rs`）

### 阶段 8.5：Redis 分布式支持 ✅ 已完成

- [✅] **8.5.1 Redis 配置管理** - 支持环境变量配置，向后兼容
- [✅] **8.5.2 Redis 连接管理** - `RedisManager` 管理连接池和健康检查
- [✅] **8.5.3 Redis Pub/Sub 模块** - 实现跨节点消息广播
- [✅] **8.5.4 WebSocket 管理器改造** - 支持本地广播 + Redis 发布
- [✅] **8.5.5 AppState 集成** - 根据配置动态启用 Redis
- [✅] **8.5.6 集成测试** - Redis 配置和消息序列化测试

**验收标准**：
- ✅ Redis 配置支持环境变量，向后兼容
- ✅ WebSocket 管理器支持分布式广播
- ✅ 多节点部署时消息可以跨节点同步
- ✅ Redis 为可选组件，不启用时系统正常工作

**测试覆盖**：8 个 Redis 集成测试全部通过（`tests/redis_integration_test.rs`）

### 阶段 8.6：基于 Redis 的数据库写入优化与配置热更新同步 ⚠️ 部分完成

- [✅] **8.6.1 Redis Stream 异步写入架构** - 审计日志先写入 Redis Stream，Consumer 批量写入 PostgreSQL
- [✅] **8.6.2 Redis Pub/Sub 配置热更新同步** - 配置变更通过 Redis 同步到所有节点
- [✅] **8.6.3 代码架构实现** - `stream.rs`、`config_sync.rs` 模块实现
- [✅] **8.6.4 降级与容错机制** - Redis 故障时自动降级，系统可用性不受影响
- [⚠️] **8.6.5 性能测试与验证** - 架构实现✅，性能指标待生产环境验证

**验收标准**：
- ⚠️ Redis Stream 异步写入吞吐量 ≥ 10 万条/秒（架构实现✅，待生产环境验证）
- ⚠️ Consumer 批量写入 DB 延迟 < 1 秒（架构实现✅，待生产环境验证）
- ⚠️ 配置变更在多节点间同步延迟 < 100ms（架构实现✅，待生产环境验证）
- ✅ Redis 故障时自动降级，系统可用性不受影响

### 阶段九：实际应用场景测试与细节修复 ⚠️ 部分完成

- [✅] **9.1 端到端场景测试** - 模拟完整用户场景（注册→创建房间→发送消息→离开）、多用户并发、用户重连、房间管理员操作
- [⏸️] **9.2 边界场景测试** - 大量消息（10万+）、大量用户（1000+）、长连接稳定性（待实施）
- [⏸️] **9.3 异常场景处理** - 数据库断开重连、磁盘空间不足、恶意用户行为防护（待实施）
- [⏸️] **9.4 用户体验优化** - 错误提示、加载状态、消息通知、离线消息处理（待实施）
- [⏸️] **9.5 性能细节优化** - 慢查询分析、WebSocket 连接优化、缓存策略（待实施）

**验收标准**：
- ✅ 端到端场景测试覆盖所有核心功能
- [⏸️] 边界场景测试通过，系统稳定（待实施）
- [⏸️] 异常场景有适当的处理和提示（待实施）
- [⏸️] 用户体验流畅，无明显卡顿（待实施）

### 阶段十：生产部署与文档 ⚠️ 部分完成

- [✅] **10.1 容器化部署** - Dockerfile、docker-compose、健康检查
- [⏸️] **10.2 CI/CD 流程** - GitHub Actions 自动化测试、构建、部署（待阶段9完成后实施）
- [⏸️] **10.3 监控与告警** - Prometheus + Grafana 监控面板（待阶段9完成后实施）
- [✅] **10.4 API 文档** - Markdown 完整文档✅、OpenAPI/Swagger 在线文档（待集成）
- [✅] **10.5 部署文档** - 生产环境部署指南、配置说明
- [⏸️] **10.6 运维手册** - 管理员操作指南、故障排查（待阶段9完成后实施）
- [⏸️] **10.7 用户手册** - 用户使用指南、FAQ（待阶段9完成后实施）

**验收标准**：
- ✅ 应用可以通过 Docker Compose 一键启动
- [⏸️] CI/CD 流程自动化运行（待阶段9完成后实施）
- [⏸️] 监控面板正常展示系统指标（待阶段9完成后实施）
- ✅ API 文档完整（Markdown 版本✅，在线 Swagger 版本待集成）

## 快速开始

### 环境要求

- Rust 1.70+
- PostgreSQL 14+
- sqlx-cli

### 数据库设置

```bash
# 创建数据库
sqlx database create

# 运行迁移
sqlx migrate run
```

### 配置与运行

1. 复制 `.env.example` 为 `.env` 并配置数据库连接等敏感信息
2. 根据需要修改 `config.toml` 中的非敏感配置
3. 运行数据库迁移：`sqlx migrate run`
4. 启动应用：

```bash
cargo run
```

应用将启动在 `http://localhost:3000`

```bash
cargo run
```

验证运行状态：

```bash
curl http://localhost:3000/health
```

## API 文档

详细的 API 文档请参考 [docs/api](docs/api) 目录：

- [HTTP API 文档](docs/api/v1/http/) - RESTful API 接口说明
- [WebSocket API 文档](docs/api/v1/websocket/) - 实时通信协议说明

主要接口概览：
- **认证**: 注册、登录、Token 刷新
- **用户**: 用户信息管理、状态更新
- **聊天室**: 房间 CRUD、成员管理、权限控制
- **消息**: 发送/接收、历史查询、搜索、编辑删除
- **文件**: 上传、下载、管理
- **管理员**: 用户/房间/消息管理、系统配置、审计日志

## 开发指南

### 代码规范

```bash
# 格式化代码
cargo fmt

# 运行 Clippy 检查
cargo clippy -- -D warnings

# 运行测试
cargo test
```

### 数据库迁移

```bash
# 创建新迁移
sqlx migrate add <migration_name>

# 运行迁移
sqlx migrate run

# 回滚迁移
sqlx migrate revert
```

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

[MIT](LICENSE)

## 联系方式

如有问题或建议，欢迎提交 Issue 或 Pull Request。

---

*项目开发中，文档会随开发进度更新*
