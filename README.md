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

## 项目结构

```
SeredeliRoom/
├── Cargo.toml              # Rust项目配置和依赖
├── .env.example            # 环境变量示例
├── .env.development        # 开发环境配置
├── README.md               # 项目文档
├── docs/
│   └── development-roadmap.md  # 详细开发路线文档
├── migrations/             # 数据库迁移脚本
│   └── 001_init.sql        # 初始数据库结构
├── src/
│   ├── main.rs             # 应用入口
│   ├── lib.rs              # 库模块导出
│   ├── config/             # 配置管理
│   ├── db/                 # 数据库连接池
│   ├── error/              # 错误处理
│   ├── handlers/           # HTTP请求处理器
│   │   ├── auth.rs         # 认证接口
│   │   ├── message.rs      # 消息接口
│   │   ├── room.rs         # 聊天室接口
│   │   └── user.rs         # 用户接口
│   ├── middleware/         # 中间件
│   ├── models/             # 数据模型
│   │   ├── user.rs         # 用户模型
│   │   ├── room.rs         # 聊天室模型
│   │   └── message.rs      # 消息模型
│   ├── routes/             # 路由配置
│   ├── services/           # 业务逻辑服务层
│   │   ├── auth_service.rs
│   │   ├── message_service.rs
│   │   ├── room_service.rs
│   │   └── user_service.rs
│   ├── state/              # 应用状态
│   ├── utils/              # 工具函数
│   └── websocket/          # WebSocket处理
│       ├── handler.rs
│       └── manager.rs
└── tests/                  # 集成测试
    ├── integration_test.rs
    └── websocket_test.rs
```

## 开发阶段

### 阶段一：基础架构搭建 ✅ 已完成

- [x] **1.1 配置管理** - 实现从环境变量加载配置，支持多环境配置
- [x] **1.2 数据库连接** - 配置 PostgreSQL 连接池，实现数据库迁移
- [x] **1.3 错误处理** - 完善错误类型定义，实现统一错误响应格式
- [x] **1.4 项目启动** - 完成应用启动逻辑，支持优雅关闭

**验收标准**：
- ✅ 应用可以正常启动并连接数据库
- ✅ 数据库迁移可以自动执行
- ✅ 日志系统正常工作
- ✅ 健康检查端点返回 200

### 阶段二：用户认证系统 🔄 待开发

- [ ] **2.1 用户模型** - 完善用户数据模型和验证
- [ ] **2.2 密码安全** - 集成 argon2 进行密码哈希
- [ ] **2.3 JWT 认证** - 实现 Token 生成、验证和刷新
- [ ] **2.4 认证接口** - 实现注册、登录接口
- [ ] **2.5 认证中间件** - 实现 JWT 认证中间件

### 阶段三：聊天室管理 ⏳ 待开发

- [ ] **3.1 聊天室模型** - 实现聊天室和成员模型
- [ ] **3.2 聊天室接口** - 创建、获取列表、详情接口
- [ ] **3.3 成员管理** - 加入、离开、成员列表接口
- [ ] **3.4 权限控制** - 实现角色权限检查

### 阶段四：WebSocket 实时通信 ⏳ 待开发

- [ ] **4.1 WebSocket 管理器** - 管理连接和房间订阅
- [ ] **4.2 WebSocket 处理器** - 实现消息收发循环
- [ ] **4.3 消息协议** - 定义 WebSocket 消息格式
- [ ] **4.4 房间广播** - 实现消息广播和单播
- [ ] **4.5 心跳机制** - 实现心跳检测和超时处理

### 阶段五：消息系统 ⏳ 待开发

- [ ] **5.1 消息模型** - 支持多种消息类型
- [ ] **5.2 消息存储** - 实现消息持久化
- [ ] **5.3 消息查询** - 历史消息、搜索功能
- [ ] **5.4 消息接口** - 获取历史、删除消息

### 阶段六：用户功能完善 ⏳ 待开发

- [ ] **6.1 用户资料** - 获取、更新用户信息
- [ ] **6.2 用户状态** - 在线状态管理
- [ ] **6.3 用户列表** - 分页、搜索用户

### 阶段七：测试与优化 ⏳ 待开发

- [ ] **7.1 单元测试** - 服务层单元测试
- [ ] **7.2 集成测试** - API 集成测试
- [ ] **7.3 WebSocket 测试** - WebSocket 连接测试
- [ ] **7.4 性能优化** - 数据库查询优化
- [ ] **7.5 代码质量** - Clippy 检查、代码格式化

### 阶段八：部署与运维 ⏳ 待开发

- [ ] **8.1 容器化** - Dockerfile、docker-compose
- [ ] **8.2 配置管理** - 生产环境配置
- [ ] **8.3 监控** - 日志收集、性能监控
- [ ] **8.4 文档** - API 文档、部署文档

## 快速开始

### 环境要求

- Rust 1.70+
- PostgreSQL 14+
- sqlx-cli

### 安装依赖

```bash
# 安装 sqlx-cli
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# 克隆项目
git clone <repository-url>
cd SeredeliRoom
```

### 数据库设置

```bash
# 创建数据库
sqlx database create

# 运行迁移
sqlx migrate run
```

### 配置环境变量

复制 `.env.example` 为 `.env.development` 并修改配置：

```bash
cp .env.example .env.development
```

编辑 `.env.development`：

```env
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# 数据库配置
DATABASE_URL=postgres://username:password@localhost:5432/seredeli_room
DATABASE_MAX_CONNECTIONS=10

# JWT配置
JWT_SECRET=your-super-secret-jwt-key
JWT_EXPIRATION_HOURS=24

# 日志级别
RUST_LOG=info
```

### 运行应用

```bash
cargo run
```

应用将启动在 `http://localhost:3000`

### 验证运行状态

```bash
curl http://localhost:3000/health
# 应返回: OK
```

## API 端点

### 健康检查
- `GET /health` - 健康检查

### WebSocket
- `GET /ws` - WebSocket 连接端点

### 认证
- `POST /api/auth/register` - 用户注册
- `POST /api/auth/login` - 用户登录
- `POST /api/auth/refresh` - 刷新 Token

### 用户
- `GET /api/users/me` - 获取当前用户信息
- `PUT /api/users/me` - 更新用户信息
- `GET /api/users/` - 获取用户列表

### 聊天室
- `GET /api/rooms/` - 获取聊天室列表
- `POST /api/rooms/` - 创建聊天室
- `GET /api/rooms/:room_id` - 获取聊天室详情
- `POST /api/rooms/:room_id/join` - 加入聊天室
- `POST /api/rooms/:room_id/leave` - 离开聊天室
- `GET /api/rooms/:room_id/members` - 获取成员列表
- `GET /api/rooms/:room_id/messages` - 获取消息历史

### 消息
- `GET /api/messages/search` - 搜索消息
- `DELETE /api/messages/:message_id` - 删除消息

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

### 环境变量

- `APP_ENV` - 应用环境（development/production）
- `RUST_LOG` - 日志级别（info/debug/trace）

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
