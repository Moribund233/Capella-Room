# Seredeli Room

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Axum](https://img.shields.io/badge/Axum-0.7+-blue.svg)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-14+-blue.svg)](https://www.postgresql.org)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

一个基于 **Axum + WebSocket + PostgreSQL** 构建的高性能实时聊天室应用。

## 功能特性

- **实时通信** - WebSocket 支持，消息实时广播，心跳检测
- **用户认证** - JWT + Argon2 密码加密，Token 刷新机制
- **聊天室管理** - 房间 CRUD、成员角色（Owner/Admin/Member）、权限控制
- **消息系统** - 消息持久化、历史记录、全文搜索、编辑删除、回复功能
- **文件上传** - 分类存储、权限控制、文件去重
- **管理员系统** - 三级角色权限、运维 API、配置热更新
- **安全审计** - 审计日志、安全告警、IP 黑白名单
- **分布式支持** - Redis Pub/Sub 跨节点消息同步

## 技术栈

| 类别 | 技术 |
|------|------|
| Web 框架 | [Axum](https://github.com/tokio-rs/axum) |
| 异步运行时 | [Tokio](https://tokio.rs/) |
| 数据库 | [PostgreSQL](https://www.postgresql.org/) + [sqlx](https://github.com/launchbadge/sqlx) |
| WebSocket | axum::extract::ws |
| 认证 | JWT + argon2 |
| 配置管理 | config + dotenvy（支持热更新） |
| 日志 | tracing |
| 验证 | validator |
| 分布式 | Redis (Pub/Sub + Stream) |

## 快速开始

### 环境要求

- Rust 1.70+
- PostgreSQL 14+
- sqlx-cli

### 安装与运行

```bash
# 克隆项目
git clone <repository-url>
cd seredeli-room

# 创建数据库
sqlx database create

# 运行迁移
sqlx migrate run

# 配置环境变量
cp .env.example .env
# 编辑 .env 文件配置数据库连接等信息

# 启动应用
cargo run
```

应用将启动在 `http://localhost:3000`

### 验证运行状态

```bash
curl http://localhost:3000/health
```

## 项目结构

```
seredeli-room/
├── Cargo.toml              # Rust 项目配置
├── config.toml             # 应用配置文件
├── .env.example            # 环境变量示例
├── migrations/             # 数据库迁移脚本
├── src/
│   ├── main.rs             # 应用入口
│   ├── config/             # 配置管理（支持热更新）
│   ├── handlers/           # HTTP 请求处理器
│   ├── middleware/         # 中间件（认证、审计）
│   ├── models/             # 数据模型
│   ├── redis/              # Redis 分布式支持
│   ├── services/           # 业务逻辑服务层
│   ├── websocket/          # WebSocket 处理
│   └── ...
├── tests/                  # 集成测试
└── docs/                   # 项目文档
```

## 文档

| 文档 | 说明 |
|------|------|
| [docs/api/](docs/api/) | API 文档 - HTTP 和 WebSocket 接口说明 |
| [docs/architecture/](docs/architecture/) | 架构文档 - 分布式架构与设计 |
| [docs/v1/roadmap.md](docs/v1/roadmap.md) | 开发路线图 - 详细开发阶段和里程碑 |
| [docs/test-reports/](docs/test-reports/) | 测试报告 - 测试覆盖与性能报告 |

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test phase1_infrastructure_test
```

**测试统计**：252 个测试全部通过

| 测试类别 | 数量 | 状态 |
|---------|------|------|
| 单元测试 | 53 | ✅ 通过 |
| 功能测试 | 173 | ✅ 通过 |
| 集成测试 | 9 | ✅ 通过 |
| WebSocket 测试 | 17 | ✅ 通过 |

## 开发规范

```bash
# 格式化代码
cargo fmt

# 运行 Clippy 检查
cargo clippy -- -D warnings

# 创建数据库迁移
sqlx migrate add <migration_name>
```

## 主要 API 概览

- **认证** - `POST /api/v1/auth/register`, `POST /api/v1/auth/login`
- **用户** - `GET /api/v1/users/me`, `PUT /api/v1/users/me`
- **聊天室** - `GET /api/v1/rooms`, `POST /api/v1/rooms`, `GET /api/v1/rooms/:id`
- **消息** - `GET /api/v1/rooms/:id/messages`, `POST /api/v1/rooms/:id/messages`
- **WebSocket** - `ws://localhost:3000/ws`
- **管理员** - `GET /api/v1/admin/users`, `GET /api/v1/admin/audit/logs`

详细 API 文档请参考 [docs/api/](docs/api/)。

## 许可证

[MIT](LICENSE)

## 贡献

欢迎提交 Issue 和 Pull Request。

---
*项目持续开发中*
