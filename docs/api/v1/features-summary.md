# Capella Room API v1 功能汇总

> **文档版本**: v1.4  
> **最后更新**: 2026-06-14  
> **适用范围**: 阶段 1-9、阶段 9.5、阶段 9.6、C1-C2 所有已实现功能

---

## 目录

- [项目概述](#项目概述)
- [阶段总览](#阶段总览)
- [HTTP API 功能](#http-api-功能)
- [WebSocket 功能](#websocket-功能)
- [文档索引](#文档索引)

---

## 项目概述

Capella Room 是一个基于 **Axum + WebSocket + PostgreSQL** 构建的实时聊天室应用，支持分布式部署和水平扩展。

### 技术栈

| 组件 | 技术 |
|------|------|
| Web 框架 | Axum (Rust) |
| 实时通信 | WebSocket |
| 数据库 | PostgreSQL |
| 缓存/消息队列 | Redis |
| 认证 | JWT (JSON Web Token) |
| 密码加密 | Argon2 |

---

## 阶段总览

| 阶段 | 名称 | 状态 | 主要功能 |
|------|------|------|----------|
| 1 | 基础架构搭建 | ✅ 已完成 | 配置管理、数据库连接、错误处理、健康检查 |
| 2 | 用户认证系统 | ✅ 已完成 | 注册、登录、JWT Token、密码加密、Token 刷新 |
| 3 | 聊天室管理 | ✅ 已完成 | 房间 CRUD、成员管理、权限控制 |
| 4 | WebSocket 通信 | ✅ 已完成 | 实时消息、心跳、重连、广播、通知系统 |
| 5 | 消息系统 | ✅ 已完成 | 消息存储、历史记录、编辑删除、全文搜索 |
| 6 | 用户功能 | ✅ 已完成 | 用户资料、状态管理、用户列表、消息回复 |
| 6.5 | 文件上传与资源管理 | ✅ 已完成 | 文件上传、分类存储、权限控制、文件去重 |
| 7 | 测试与优化 | ✅ 已完成 | 单元测试、集成测试、性能优化、代码质量 |
| 8 | 配置化与运维管理 | ✅ 已完成 | 配置体系、管理员系统、运维 API |
| 8.4 | 安全审计系统 | ✅ 已完成 | 审计日志、安全告警、合规追溯 |
| 8.5 | Redis 分布式支持 | ✅ 已完成 | 分布式 WebSocket 广播、水平扩展 |
| 8.6 | Redis 数据库优化 | ✅ 已完成 | Redis Stream 异步写入、配置热更新同步 |
| 8.7.1 | IP 安全系统 | ✅ 已完成 | IP 黑名单/白名单系统、CIDR 支持、白名单模式 |
| 8.7.2 | 账号安全系统 | ✅ 已完成 | 用户设置体系、设备管理、单设备登录、设备禁用 |
| 9 | 后端细节优化 | ✅ 已完成 | 搜索功能、私聊、好友系统、房间邀请 |
| 9.5 | OAuth 2.0 与 Webhook | ✅ 已完成 | OAuth 授权码流程、OAuth App 管理、资源绑定、Webhook 订阅、自定义事件 |
| 9.6 | 死信队列与增强监控 | ✅ 已完成 | 死信队列(DLQ)、系统监控、增强统计、Redis 管理、配置同步 |
| C1 | 账号注销 | ✅ 已完成 | 用户软删除、匿名化、Token 失效 |
| C2 | 消息置顶 | ✅ 已完成 | 置顶/取消置顶、置顶列表、WebSocket 实时广播 |

---

## HTTP API 功能

### 1. 系统接口 ([system.md](./http/system.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/health` | 基础健康检查 | 无需 |
| GET | `/health/detail` | 详细健康状态 | 无需 |
| GET | `/health/ready` | Kubernetes 就绪检查 | 无需 |
| GET | `/health/live` | Kubernetes 存活检查 | 无需 |
| GET | `/api/version` | API 版本信息 | 无需 |
| GET | `/api/config/client` | 客户端配置 | 无需 |

**阶段归属**: 阶段 1 (基础架构)

---

### 2. 认证接口 ([auth.md](./http/auth.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v1/auth/register` | 用户注册 | 无需 |
| POST | `/api/v1/auth/login` | 用户登录 | 无需 |
| POST | `/api/v1/auth/refresh` | 刷新 Token | 无需 |

**功能特性**:
- Argon2 密码哈希
- 密码强度验证（大小写字母+数字，至少8位）
- JWT Access Token (24小时) + Refresh Token
- 用户名/邮箱唯一性验证

**阶段归属**: 阶段 2 (用户认证系统)

---

### 3. 用户接口 ([user.md](./http/user.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/v1/users/me` | 获取当前用户信息 | 需要 |
| PUT | `/api/v1/users/me` | 更新当前用户信息 | 需要 |
| PUT | `/api/v1/users/me/password` | 修改密码 | 需要 |
| GET | `/api/v1/users/me/rooms` | 获取我的聊天室列表 | 需要 |
| POST | `/api/v1/users/logout` | 登出 | 需要 |
| GET | `/api/v1/users` | 获取用户列表（搜索+分页） | 需要 |
| **GET** | **`/api/v1/users/search`** | **搜索用户** | **需要** |
| GET | `/api/v1/users/:user_id` | 获取指定用户信息 | 需要 |
| **GET** | **`/api/v1/users/friends`** | **获取好友列表** | **需要** |
| **POST** | **`/api/v1/users/friends/requests`** | **发送好友请求** | **需要** |
| **GET** | **`/api/v1/users/friends/requests/received`** | **获取收到的好友请求** | **需要** |
| **GET** | **`/api/v1/users/friends/requests/sent`** | **获取发送的好友请求** | **需要** |
| **POST** | **`/api/v1/users/friends/requests/handle`** | **处理好友请求** | **需要** |
| **DELETE** | **`/api/v1/users/friends/requests/:id`** | **取消好友请求** | **需要** |
| **DELETE** | **`/api/v1/users/friends/:id`** | **删除好友** | **需要** |
| **DELETE** | **`/api/v1/users/me`** | **注销账号（软删除）** | **需要** |
| **GET** | **`/api/v1/users/me/stats`** | **用户统计** | **需要** |
| **GET** | **`/api/v1/users/recommended`** | **推荐用户（在线优先+随机补充）** | **需要** |

**功能特性**:
- 用户名/邮箱模糊搜索
- 分页支持 (limit/offset)
- 用户在线状态显示
- **用户搜索**: 支持隐私设置过滤（Everyone/Friends/Nobody）
- **推荐用户**: 在线用户优先 + 随机补充
- **用户统计**: 消息数、房间数、好友数等统计
- **好友系统**:
  - 发送/接收好友请求（带附加消息）
  - 接受/拒绝好友请求
  - 取消待处理请求
  - 删除好友
  - 好友列表管理
- **账号注销**: 软删除用户，匿名化个人信息，Token 立即失效

**阶段归属**: 阶段 6 (用户功能)、阶段 9 (后端细节优化)、C1 (账号注销)

---

### 4. 用户设置接口 ([user.md](./http/user.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/v1/users/me/settings` | 获取用户设置 | 需要 |
| PATCH | `/api/v1/users/me/settings` | 部分更新用户设置 | 需要 |

**设置分组**:
| 分组 | 字段数 | 关键功能 |
|------|--------|----------|
| 通知设置 | 8 | 私信、@提及、声音、桌面通知、勿扰 |
| 隐私设置 | 5 | 在线状态可见性、单设备登录开关 |
| 消息设置 | 4 | 消息预览、已读回执、输入指示器 |
| 语言设置 | 5 | 语言、时区、时间/日期格式 |
| 无障碍设置 | 4 | 字体大小、高对比度、减少动画 |
| 媒体设置 | 5 | 自动下载、图片质量、自动播放 |

**阶段归属**: 阶段 8.7.2 (账号安全系统)

**房间级设置**（用户设置扩展）:

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/users/me/rooms/settings` | 列出所有房间级设置 |
| GET | `/api/v1/users/me/rooms/:room_id/settings` | 获取指定房间的设置 |
| PATCH | `/api/v1/users/me/rooms/:room_id/settings` | 更新指定房间的设置 |
| DELETE | `/api/v1/users/me/rooms/:room_id/settings` | 删除指定房间的设置 |

房间级设置覆盖同名字段的全局设置，支持按房间自定义通知偏好等。

---

### 5. 账号安全接口 ([user.md](./http/user.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/v1/users/me/security/overview` | 获取账号安全概览 | 需要 |
| GET | `/api/v1/users/me/devices` | 获取登录设备列表 | 需要 |
| DELETE | `/api/v1/users/me/devices/:device_id` | 登出指定设备 | 需要 |
| POST | `/api/v1/users/me/devices/:device_id/block` | 禁用指定设备 | 需要 |
| POST | `/api/v1/users/me/devices/:device_id/unblock` | 启用被禁用的设备 | 需要 |
| POST | `/api/v1/users/me/devices/terminate-others` | 登出所有其他设备 | 需要 |
| GET | `/api/v1/users/me/login-history` | 获取登录历史 | 需要 |
| GET | `/api/v1/users/me/login-history/suspicious` | 获取可疑登录记录 | 需要 |

**功能特性**:
- 设备信息管理（名称、类型、IP、位置）
- 设备状态追踪（活跃、当前设备、禁用状态）
- 远程登出其他设备
- 设备禁用/启用（被禁用设备无法使用旧 Token）
- 单设备登录控制（用户设置中开启）
- 登录历史记录（成功/失败、风险等级、可疑标记）
- 安全概览统计

**阶段归属**: 阶段 8.7.2 (账号安全系统)

---

### 6. 通知接口 ([notifications.md](./http/notifications.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/v1/notifications` | 获取通知列表 | 需要 |
| GET | `/api/v1/notifications/unread-count` | 获取未读通知数 | 需要 |
| POST | `/api/v1/notifications/:id/read` | 标记通知已读 | 需要 |
| POST | `/api/v1/notifications/read-all` | 标记所有通知已读 | 需要 |

**功能特性**:
- 通知列表分页
- 未读计数
- 单条/全部标记已读
- WebSocket 实时推送 + HTTP 查询双通道

**阶段归属**: 阶段 4.6 (消息通知系统)

---

### 7. 聊天室接口 ([rooms.md](./http/rooms.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/v1/rooms` | 获取聊天室列表（搜索+分页） | 需要 |
| POST | `/api/v1/rooms` | 创建聊天室 | 需要 |
| GET | `/api/v1/rooms/recent` | 获取最近更新的聊天室 | 需要 |
| **POST** | **`/api/v1/rooms/direct`** | **创建或获取私聊房间** | **需要** |
| **GET** | **`/api/v1/rooms/direct/list`** | **获取私聊房间列表** | **需要** |
| GET | `/api/v1/rooms/:room_id` | 获取聊天室详情 | 需要 |
| PUT | `/api/v1/rooms/:room_id` | 更新聊天室信息 | 需要 |
| DELETE | `/api/v1/rooms/:room_id` | 删除聊天室 | 需要 |
| POST | `/api/v1/rooms/:room_id/join` | 加入聊天室 | 需要 |
| DELETE | `/api/v1/rooms/:room_id/leave` | 离开聊天室 | 需要 |
| **POST** | **`/api/v1/rooms/join-by-invite`** | **通过邀请码加入房间** | **需要** |
| **GET** | **`/api/v1/rooms/validate-invite`** | **验证邀请码** | **需要** |
| GET | `/api/v1/rooms/:room_id/members` | 获取成员列表 | 需要 |
| DELETE | `/api/v1/rooms/:room_id/members/:user_id` | 踢出成员 | 需要 |
| PUT | `/api/v1/rooms/:room_id/members/:user_id/role` | 设置成员角色 | 需要 |
| **GET** | **`/api/v1/rooms/:room_id/invitations`** | **获取房间邀请列表** | **需要** |
| **POST** | **`/api/v1/rooms/:room_id/invitations`** | **创建房间邀请** | **需要** |
| **DELETE** | **`/api/v1/rooms/:room_id/invitations/:id`** | **撤销房间邀请** | **需要** |
| GET | `/api/v1/rooms/:room_id/messages` | 获取房间消息历史 | 需要 |

**功能特性**:
- 三级角色系统：Owner / Admin / Member
- 公开/私有房间设置
- 成员数量限制 (2-1000)
- 房间搜索和分页
- 最近活跃房间列表
- **私聊功能**:
  - 1对1私聊房间（`RoomType::Direct`）
  - 自动查找/创建私聊房间
  - 房间名称动态更新（使用对方最新用户名）
- **房间邀请机制**:
  - 生成邀请码（8位字母数字）
  - 邀请码有效期控制（可选）
  - 邀请码使用次数限制（可选）
  - 邀请码冲突自动重试（最多3次）
  - 通过邀请码加入房间
  - 验证邀请码有效性

**阶段归属**: 阶段 3 (聊天室管理)、阶段 9 (后端细节优化)

---

### 8. 消息接口 ([messages.md](./http/messages.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/v1/rooms/:room_id/messages` | 获取房间消息历史 | 需要 |
| GET | `/api/v1/messages/search` | 搜索消息 | 需要 |
| PUT | `/api/v1/messages/:message_id` | 编辑消息 | 需要 |
| DELETE | `/api/v1/messages/:message_id` | 删除消息 | 需要 |
| GET | `/api/v1/messages/:message_id/history` | 获取消息编辑历史 | 需要 |
| POST | `/api/v1/messages/:message_id/reactions` | 添加表情反应 | 需要 |
| DELETE | `/api/v1/messages/:message_id/reactions` | 移除表情反应 | 需要 |
| GET | `/api/v1/messages/:message_id/reactions` | 获取消息反应列表 | 需要 |
| **POST** | **`/api/v1/messages/:message_id/pin`** | **置顶消息** | **需要** |
| **DELETE** | **`/api/v1/messages/:message_id/pin`** | **取消置顶消息** | **需要** |
| **GET** | **`/api/v1/rooms/:room_id/pinned-messages`** | **获取置顶消息列表** | **需要** |

**功能特性**:
- 游标分页（无限滚动）
- 全文搜索（PostgreSQL tsvector + GIN 索引）
- 消息编辑历史追踪
- 消息回复（引用消息）
- 消息软删除
- 多种消息类型：text / image / file / system
- **表情反应**:
  - 添加/移除表情反应（HTTP + WebSocket 双通道）
  - 唯一约束：同一用户对同一消息的同一表情只允许一个反应
  - WebSocket 实时广播反应变更（`ReactionAdded` / `ReactionRemoved`）
  - 消息列表自动附带反应汇总（`reactions` 字段）
  - 独立反应查询接口（`GET /messages/:id/reactions`）
- **消息置顶**:
  - 置顶/取消置顶消息（HTTP + WebSocket 双通道）
  - 唯一约束：同一消息只能被置顶一次
  - WebSocket 实时广播置顶变更（`MessagePinned` / `MessageUnpinned`）
  - 置顶列表查询接口（`GET /rooms/:room_id/pinned-messages`）
  - 已删除消息自动从置顶列表排除

**阶段归属**: 阶段 5 (消息系统)、阶段 6 扩展 (消息编辑/回复)

---

### 9. 文件接口 ([files.md](./http/files.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/v1/files` | 获取文件列表 | 需要 |
| GET | `/api/v1/files/:file_id` | 获取文件详情 | 需要 |
| DELETE | `/api/v1/files/:file_id` | 删除文件 | 需要 |
| POST | `/api/v1/upload` | 通用文件上传 | 需要 |
| POST | `/api/v1/upload/image` | 上传图片 | 需要 |
| POST | `/api/v1/upload/avatar` | 上传头像 | 需要 |
| **POST** | **`/api/v1/upload/chunked/init`** | **初始化分片上传** | **需要** |
| **POST** | **`/api/v1/upload/chunked/:session_id/:chunk_index`** | **上传分片** | **需要** |
| **GET** | **`/api/v1/upload/chunked/:session_id/status`** | **查询分片上传状态** | **需要** |
| **POST** | **`/api/v1/upload/chunked/:session_id/complete`** | **完成分片上传** | **需要** |
| **DELETE** | **`/api/v1/upload/chunked/:session_id`** | **取消分片上传** | **需要** |

**功能特性**:
- 文件分类存储：images / documents / videos / audio / others
- 按年月目录结构组织
- SHA256 文件去重
- 文件类型白名单验证
- 文件大小限制（默认 10MB）
- 用户只能删除自己的文件
- **分片上传**:
  - 初始化分片会话，返回 session_id
  - 按索引顺序上传分片
  - 查询已上传分片状态
  - 完成后台合并文件
  - 支持取消分片会话

**阶段归属**: 阶段 6.5 (文件上传与资源管理)

---

### 10. 管理员接口 ([admin.md](./http/admin.md))

#### 用户管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/api/v1/admin/users` | 获取用户列表 | Admin |
| GET | `/api/v1/admin/users/:user_id` | 获取用户详情 | Admin |
| PUT | `/api/v1/admin/users/:user_id/role` | 修改用户角色 | SuperAdmin |
| PUT | `/api/v1/admin/users/:user_id/status` | 禁用/启用用户 | Admin |
| DELETE | `/api/v1/admin/users/:user_id` | 删除用户 | Admin |
| PUT | `/api/v1/admin/users/:user_id/password` | 重置用户密码 | SuperAdmin |

#### 房间管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/api/v1/admin/rooms` | 获取房间列表 | Admin |
| GET | `/api/v1/admin/rooms/:room_id` | 获取房间详情 | Admin |
| DELETE | `/api/v1/admin/rooms/:room_id` | 强制删除房间 | Admin |
| GET | `/api/v1/admin/rooms/:room_id/messages` | 获取房间消息记录 | Admin |
| DELETE | `/api/v1/admin/rooms/:room_id/members/:user_id` | 踢出房间成员 | Admin |
| PUT | `/api/v1/admin/rooms/:room_id/members/:user_id/role` | 设置成员角色 | Admin |

#### 消息审核

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/api/v1/admin/messages` | 获取所有消息 | Admin |
| DELETE | `/api/v1/admin/messages/:message_id` | 删除违规消息 | Admin |

#### 系统统计

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/api/v1/admin/stats` | 系统统计概览 | Admin |
| GET | `/api/v1/admin/stats/activity` | 活跃度统计 | Admin |
| GET | `/api/v1/admin/stats/performance` | 性能指标 | Admin |
| **GET** | **`/api/v1/admin/stats/users/growth`** | **用户增长统计** | **Admin** |
| **GET** | **`/api/v1/admin/stats/users/behavior`** | **用户行为统计** | **Admin** |
| **GET** | **`/api/v1/admin/stats/users/friends`** | **好友关系统计** | **Admin** |
| **GET** | **`/api/v1/admin/stats/rooms/activity`** | **房间活跃度排行** | **Admin** |
| **GET** | **`/api/v1/admin/stats/rooms/overview`** | **房间概览统计** | **Admin** |
| **GET** | **`/api/v1/admin/stats/messages/types`** | **消息类型分布** | **Admin** |
| **GET** | **`/api/v1/admin/stats/messages/hourly`** | **消息小时分布** | **Admin** |
| **GET** | **`/api/v1/admin/stats/security`** | **安全统计** | **Admin** |

#### 系统监控

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| **GET** | **`/api/v1/admin/monitor`** | **系统监控数据** | **Admin** |

#### 配置管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/api/v1/admin/configs` | 获取所有配置项 | Admin |
| GET | `/api/v1/admin/configs/:key` | 获取指定配置 | Admin |
| PUT | `/api/v1/admin/configs/:key` | 修改配置项 | SuperAdmin |
| POST | `/api/v1/admin/configs/reset` | 重置配置到默认值 | SuperAdmin |

#### 审计系统

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/api/v1/admin/audit/logs` | 查询审计日志 | Admin |
| GET | `/api/v1/admin/audit/logs/:id` | 获取日志详情 | Admin |
| GET | `/api/v1/admin/audit/stats` | 审计统计信息 | Admin |
| GET | `/api/v1/admin/audit/export` | 导出审计日志 | Admin |
| GET | `/api/v1/admin/audit/alerts` | 获取安全告警列表 | Admin |
| PUT | `/api/v1/admin/audit/alerts/:id/status` | 更新告警状态 | Admin |
| GET | `/api/v1/admin/audit/rules` | 获取告警规则 | Admin |
| PUT | `/api/v1/admin/audit/rules/:id` | 修改告警规则 | SuperAdmin |
| POST | `/api/v1/admin/audit/cleanup` | 清理过期日志 | SuperAdmin |

#### IP 安全管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| GET | `/api/v1/admin/security/ip-list` | 查询 IP 列表 | Admin |
| POST | `/api/v1/admin/security/ip-list` | 添加 IP 到列表 | Admin |
| POST | `/api/v1/admin/security/ip-list/batch` | 批量添加 IP | Admin |
| PUT | `/api/v1/admin/security/ip-list/:id` | 更新 IP 条目 | Admin |
| DELETE | `/api/v1/admin/security/ip-list/:id` | 移除 IP | Admin |
| POST | `/api/v1/admin/security/ip-check` | 检查 IP 状态 | Admin |
| GET | `/api/v1/admin/security/stats` | 获取安全统计 | Admin |
| POST | `/api/v1/admin/security/refresh-cache` | 刷新缓存 | Admin |
| POST | `/api/v1/admin/security/cleanup-expired` | 清理过期条目 | Admin |
| GET | `/api/v1/admin/security/whitelist-mode` | 获取白名单模式状态 | Admin |
| POST | `/api/v1/admin/security/whitelist-mode` | 设置白名单模式 | SuperAdmin |

#### 待办通知管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| **POST** | **`/api/v1/admin/pending-actions/:id/respond`** | **处理待办通知** | **Admin** |

#### 死信队列 (DLQ)

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| **GET** | **`/api/v1/admin/dlq/messages`** | **查询死信队列消息列表** | **Admin** |
| **GET** | **`/api/v1/admin/dlq/stats`** | **死信队列统计** | **Admin** |
| **POST** | **`/api/v1/admin/dlq/batch-requeue`** | **批量重新入队** | **Admin** |
| **POST** | **`/api/v1/admin/dlq/:id/requeue`** | **单条重新入队** | **Admin** |
| **DELETE** | **`/api/v1/admin/dlq/:id`** | **删除死信消息** | **Admin** |

#### Redis 管理

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| **GET** | **`/api/v1/admin/redis/status`** | **Redis 连接状态** | **Admin** |
| **GET** | **`/api/v1/admin/redis/stats`** | **Redis 运行统计** | **Admin** |
| **POST** | **`/api/v1/admin/redis/refresh`** | **刷新 Redis 缓存** | **Admin** |

#### 配置同步

| 方法 | 路径 | 说明 | 最低权限 |
|------|------|------|----------|
| **POST** | **`/api/v1/admin/config/sync`** | **触发配置同步** | **SuperAdmin** |
| **GET** | **`/api/v1/admin/config/sync/status`** | **查询配置同步状态** | **Admin** |

**功能特性**:
- 三级角色体系：User / Admin / SuperAdmin
- 审计日志记录所有管理操作
- IP 黑白名单支持 CIDR 范围
- 内存缓存 + 定期刷新机制
- **死信队列**: 消息处理失败自动进入 DLQ，支持查询、重新入队、删除
- **Redis 管理**: 连接状态监控、运行统计、手动刷新缓存
- **配置同步**: 跨节点热更新配置，支持触发和状态查询

**阶段归属**: 阶段 8 (配置化与运维管理)、阶段 8.4 (安全审计系统)、阶段 8.7.1 (IP 安全系统)、阶段 9.6 (死信队列与增强监控)

---

### 11. UI 配置接口 ([ui-config.md](./http/ui-config.md))

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/v1/ui/config` | 获取用户 UI 配置 | 需要 |
| POST | `/api/v1/ui/config` | 保存用户 UI 配置 | 需要 |
| DELETE | `/api/v1/ui/config` | 重置用户 UI 配置 | 需要 |

**功能特性**:
- 模块化配置：app / theme / sidebar / quickbar / dock
- 云端同步用户界面配置
- 专为 CapellaUI 前端框架设计

**阶段归属**: 阶段 8 (配置化与运维管理)

---

### 12. OAuth 2.0 与 Webhook 接口 ([oauth.md](./http/oauth.md), [webhook.md](./http/webhook.md))

#### OAuth 授权流程（公开访问）

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/oauth/authorize` | OAuth 授权页面（浏览器） | 用户会话 |
| POST | `/oauth/authorize` | 提交授权请求 | 用户会话 |
| POST | `/oauth/authorize/consent` | 用户确认授权 | 用户会话 |
| POST | `/oauth/token` | 获取 Access Token | Client Secret |
| GET | `/oauth/userinfo` | 获取用户信息（OAuth） | Access Token |

#### OAuth App 管理（需要用户认证）

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v1/oauth/apps` | 创建 OAuth App | 需要 |
| GET | `/api/v1/oauth/apps` | 获取 App 列表 | 需要 |
| GET | `/api/v1/oauth/apps/:app_id` | 获取 App 详情 | 需要 |
| PUT | `/api/v1/oauth/apps/:app_id` | 更新 App | 需要 |
| DELETE | `/api/v1/oauth/apps/:app_id` | 删除 App | 需要 |
| POST | `/api/v1/oauth/apps/:app_id/rotate-secret` | 轮换 Client Secret | 需要 |

#### 资源映射与绑定

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v1/oauth/mappings` | 创建资源映射 | 需要 |
| GET | `/api/v1/oauth/mappings` | 查询资源映射 | 需要 |
| DELETE | `/api/v1/oauth/mappings/:mapping_id` | 删除映射 | 需要 |
| GET | `/api/v1/oauth/resources` | 查询资源 | 需要 |
| POST | `/api/v1/oauth/resources` | 绑定资源（自动创建） | 需要 |
| POST | `/api/v1/rooms/:room_id/resources` | 绑定资源到房间 | 需要 |
| GET | `/api/v1/rooms/:room_id/resources` | 获取房间资源列表 | 需要 |
| PUT | `/api/v1/rooms/:room_id/resources/:binding_id` | 更新资源绑定 | 需要 |
| DELETE | `/api/v1/rooms/:room_id/resources/:binding_id` | 解绑资源 | 需要 |

#### Webhook 订阅管理

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v1/webhook/subscriptions` | 创建 Webhook 订阅 | 需要 |
| GET | `/api/v1/webhook/subscriptions` | 获取订阅列表 | 需要 |
| GET | `/api/v1/webhook/subscriptions/:id` | 获取订阅详情 | 需要 |
| PUT | `/api/v1/webhook/subscriptions/:id` | 更新订阅 | 需要 |
| DELETE | `/api/v1/webhook/subscriptions/:id` | 删除订阅 | 需要 |
| GET | `/api/v1/webhook/subscriptions/:id/deliveries` | 获取投递记录 | 需要 |
| POST | `/api/v1/webhook/subscriptions/:id/deliveries/:delivery_id/redeliver` | 重新投递 | 需要 |

#### 自定义事件

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v1/rooms/:room_id/custom-events` | 发送自定义事件 | OAuth |
| GET | `/api/v1/rooms/:room_id/custom-events` | 获取错过事件 | OAuth |

**功能特性**:
- **OAuth 2.0**: 标准授权码流程 (Authorization Code Flow)
- **OAuth App**: 注册和管理第三方应用，支持 Client Secret 轮换
- **资源映射**: 外部资源与 Capella Room 资源的映射管理
- **资源绑定**: 外部资源绑定到聊天室，支持 CRUD
- **Webhook**: 事件订阅、签名验证、投递重试、投递记录查询
- **自定义事件**: 外部服务通过 OAuth 向房间发送/接收自定义事件，支持 WebSocket 实时转发和离线同步

**阶段归属**: 阶段 9.5 (OAuth 2.0 与 Webhook)

---

### 13. v2 API 接口 ([auth.md](./http/auth.md))

#### 认证接口（公开访问）

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v2/auth/register/send-code` | 发送注册验证码 | 无需 |
| POST | `/api/v2/auth/register` | 验证码注册 | 无需 |
| POST | `/api/v2/auth/login/send-code` | 发送登录验证码 | 无需 |
| POST | `/api/v2/auth/login` | 验证码登录 | 无需 |
| POST | `/api/v2/auth/reset-password/send-code` | 发送重置密码验证码 | 无需 |
| POST | `/api/v2/auth/reset-password` | 验证码重置密码 | 无需 |
| POST | `/api/v2/auth/login-with-password` | 密码登录 | 无需 |
| POST | `/api/v2/auth/refresh` | 刷新 Token | 无需 |

#### 用户接口

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v2/users/logout` | 登出 | 需要 |

**功能特性**:
- 验证码注册/登录（邮箱验证码）
- 密码重置（验证码）
- 与 v1 共享用户体系，Token 互通

**阶段归属**: 阶段 9.5 (OAuth 2.0 与 Webhook)

---

## WebSocket 功能

### 连接管理 ([index.md](./websocket/index.md))

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `Auth` | C→S | 连接认证 |
| `AuthResult` | S→C | 认证结果 |
| `Ping` | 双向 | 心跳请求 |
| `Pong` | 双向 | 心跳响应 |
| `Reconnect` | C→S | 断线重连 |
| `ReconnectResult` | S→C | 重连结果 |
| `Error` | S→C | 错误消息 |

**功能特性**:
- 连接后 30 秒内必须认证
- 服务端每 30 秒发送 Ping
- 90 秒超时自动断开
- 断线重连自动恢复房间订阅
- 离线消息同步

**阶段归属**: 阶段 4 (WebSocket 通信)

---

### 房间管理 ([room.md](./websocket/room.md))

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `JoinRoom` | C→S | 加入房间 |
| `LeaveRoom` | C→S | 离开房间 |
| `RoomJoined` | S→C | 加入房间成功 |
| `RoomLeft` | S→C | 离开房间成功 |
| `UserJoined` | S→C | 其他用户加入房间（广播） |
| `UserLeft` | S→C | 其他用户离开房间（广播） |
| `OnlineUsers` | S→C | 房间在线用户列表 |

**阶段归属**: 阶段 4 (WebSocket 通信)

---

### 消息通信 ([message.md](./websocket/message.md))

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `ChatMessage` | C→S | 发送聊天消息 |
| `Typing` | C→S | 正在输入状态 |
| `StopTyping` | C→S | 停止输入状态 |
| `MessageRead` | C→S | 消息已读确认 |
| `EditMessage` | C→S | 编辑消息 |
| `DeleteMessage` | C→S | 删除消息 |
| **`AddReaction`** | **C→S** | **添加表情反应** |
| **`RemoveReaction`** | **C→S** | **移除表情反应** |
| **`PinMessage`** | **C→S** | **置顶消息** |
| **`UnpinMessage`** | **C→S** | **取消置顶消息** |
| `GetMissedMessages` | C→S | 获取离线消息 |
| `NewMessage` | S→C | 新消息广播 |
| `UserTyping` | S→C | 用户正在输入（广播） |
| `UserStopTyping` | S→C | 用户停止输入（广播） |
| `MessageReadReceipt` | S→C | 消息已读回执 |
| `MessageEdited` | S→C | 消息已编辑通知 |
| `MessageDeleted` | S→C | 消息已删除通知 |
| `ReactionAdded` | S→C | 表情反应已添加（广播） |
| `ReactionRemoved` | S→C | 表情反应已移除（广播） |
| `MessagePinned` | S→C | 消息已置顶（广播） |
| `MessageUnpinned` | S→C | 消息已取消置顶（广播） |
| `MissedMessages` | S→C | 离线消息列表 |
| `Mentioned` | S→C | @提及通知 |
| `SystemMessage` | S→C | 系统广播消息（显示在聊天框） |
| `RoomUpdated` | S→C | 房间信息更新通知 |
| `SessionRestored` | S→C | 会话恢复完成通知 |

**功能特性**:
- 实时消息广播
- 消息回复（引用消息）
- 正在输入状态同步
- 消息已读确认
- 消息编辑和删除
- @提及检测（正则表达式 `@[a-zA-Z0-9_]{3,20}`）
- **系统广播**：管理员操作通知（删除消息、踢出成员等）显示在聊天框
- **房间更新**：房间信息变更实时通知
- **会话恢复**：断线重连后自动恢复会话状态
- **表情反应**：实时添加/移除表情反应（`AddReaction` / `RemoveReaction`），广播变更给房间所有成员（`ReactionAdded` / `ReactionRemoved`）
- **消息置顶**：实时置顶/取消置顶消息（`PinMessage` / `UnpinMessage`），广播变更给房间所有成员（`MessagePinned` / `MessageUnpinned`）

**阶段归属**: 阶段 4 (WebSocket 通信)、阶段 5 (消息系统)

---

### 通知系统 ([notification.md](./websocket/notification.md))

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `PrivateMessage` | S→C | 私信通知 |
| `Mentioned` | S→C | @提及通知 |
| `RoomInvitation` | S→C | 房间邀请通知 |
| `SystemNotification` | S→C | 系统通知 |
| `FileUploadComplete` | S→C | 文件上传完成通知 |
| `PendingAction` | S→C | 待办通知 |
| `GetOfflineNotifications` | C→S | 获取离线通知 |
| `MarkNotificationRead` | C→S | 标记通知已读 |
| `MarkAllNotificationsRead` | C→S | 标记所有通知已读 |
| `GetPendingActions` | C→S | 获取待办列表 |
| `OfflineNotifications` | S→C | 离线通知列表响应 |
| `NotificationReadConfirm` | S→C | 通知已读确认 |
| `PendingActionsList` | S→C | 待办列表响应 |

**功能特性**:
- 在线实时推送
- 离线通知存储（`notifications` 表）
- 通知历史同步
- 通知类型：new / important / warning
- **通知管理**: 获取离线通知、标记已读、已读确认
- **待办系统**: 待办通知推送、待办列表获取

**阶段归属**: 阶段 4.6 (消息通知系统)

---

### 用户状态管理 ([user-status.md](./websocket/user-status.md))

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `UpdateStatus` | C→S | 更新用户状态 |
| `GetOnlineUsers` | C→S | 获取全局在线用户列表 |
| `UserStatusChanged` | S→C | 用户状态变更通知 |
| `GlobalOnlineUsers` | S→C | 全局在线用户列表 |

**用户状态类型**:
- `online` - 在线
- `away` - 离开
- `busy` - 忙碌
- `offline` - 离线

**阶段归属**: 阶段 6 (用户功能)

---

### 系统日志流 ([system-logs.md](./websocket/system-logs.md))

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `SubscribeLogs` | C→S | 订阅系统日志 |
| `UnsubscribeLogs` | C→S | 取消订阅系统日志 |
| `LogEntry` | S→C | 系统日志条目（实时推送） |
| `LogSubscriptionConfirmed` | S→C | 日志订阅确认 |

**功能特性**:
- **实时日志推送**：服务端日志实时推送到客户端
- **级别过滤**：支持按日志级别过滤（error/warn/info/debug/all）
- **模块过滤**：支持按模块过滤（websocket/room/message/performance/all）
- **管理员专用**：仅管理员可订阅系统日志

**阶段归属**: 阶段 8 (配置化与运维管理)

---

### 房间消息摘要

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `RoomMessageSummary` | S→C | 房间消息摘要（last_message + unread_count） |

用于房间列表实时更新，显示最新消息预览和未读计数。

**阶段归属**: 阶段 4 (WebSocket 通信)

---

### 外部服务与资源绑定

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `CustomEvent` | C→S | 外部服务发送自定义事件 |
| `CustomEventForward` | S→C | 自定义事件转发给客户端 |
| `GetMissedCustomEvents` | C→S | 请求错过自定义事件 |
| `MissedCustomEvents` | S→C | 错过自定义事件列表 |
| `ResourceBound` | S→C | 资源已绑定到房间 |
| `ResourceBindingUpdated` | S→C | 资源绑定已更新 |
| `ResourceUnbound` | S→C | 资源已解绑 |

**功能特性**:
- **自定义事件**: 外部服务通过 OAuth 身份向房间发送自定义事件，客户端实时接收
- **离线同步**: 重连后自动同步错过的自定义事件
- **资源绑定**: 外部资源（如文件、外部链接）绑定到房间时实时通知

**阶段归属**: 阶段 9.5 (OAuth 2.0 与 Webhook)

---

## 高级功能

> 📖 **详细架构说明**: [architecture/README.md](../../architecture/README.md)

### 分布式支持 (阶段 8.5)

- **Redis Pub/Sub**: 跨节点 WebSocket 消息广播
- **水平扩展**: 支持多节点部署
- **频道命名空间**: `{prefix}:room:{room_id}`
- **节点标识**: 避免消息循环

**相关 API**: [admin.md#redis-与分布式管理](./http/admin.md#redis-与分布式管理)

### 数据库优化 (阶段 8.6)

- **Redis Stream**: 审计日志异步写入
- **Consumer Group**: 多节点负载均衡
- **配置热更新**: Redis Pub/Sub 同步配置变更
- **降级机制**: Redis 不可用时直接写 DB

**相关 API**: [admin.md#redis-与分布式管理](./http/admin.md#redis-与分布式管理)

### IP 安全系统 (阶段 8.7.1)

- **IP 黑白名单**: 支持单 IP 和 CIDR 范围
- **白名单模式**: 仅允许特定 IP 访问
- **WebSocket 防护**: 连接层 IP 检查
- **审计追踪**: 所有安全事件记录日志

**相关 API**: [admin.md#ip-安全管理](./http/admin.md#ip-安全管理)

### 账号安全系统 (阶段 8.7.2)

- **用户设置体系**: 6大分组设置（通知、隐私、消息、语言、无障碍、媒体）
- **设备管理**: 多设备登录追踪、设备信息展示
- **单设备登录**: 用户可开启仅允许一个设备登录
- **设备禁用**: 用户可禁用可疑设备，被禁用设备无法使用旧 Token
- **登录历史**: 记录每次登录的 IP、设备、位置、风险等级
- **安全概览**: 活跃设备统计、可疑活动检测

**相关 API**: [user.md](./http/user.md)

### OAuth 2.0 与 Webhook (阶段 9.5)

- **OAuth 授权码流程**: 标准 Authorization Code Flow
- **OAuth App 管理**: 注册第三方应用、Client Secret 轮换
- **资源映射与绑定**: 外部资源映射到 Capella 资源，绑定到聊天室
- **Webhook 订阅**: 事件订阅、签名验证、投递重试、投递记录
- **自定义事件**: 外部服务通过 OAuth 向房间发送自定义事件

**相关 API**: [oauth.md](./http/oauth.md), [webhook.md](./http/webhook.md)

### 死信队列与增强监控 (阶段 9.6)

- **死信队列 (DLQ)**: 消息处理失败自动进入死信队列，支持查询、重新入队、删除
- **增强统计**: 用户增长、用户行为、好友关系、房间活跃度排行、消息类型/时间分布
- **系统监控**: CPU/内存/连接数等实时监控数据
- **Redis 管理**: 连接状态、运行统计、手动刷新
- **配置同步**: 跨节点热更新配置

**相关 API**: [admin.md](./http/admin.md)

### 分片上传

- **大文件支持**: 将大文件切分为多个分片依次上传
- **断点续传**: 查询已上传分片状态，支持续传
- **并发上传**: 分片可并发上传，后台合并

**相关 API**: [files.md](./http/files.md)

---

## 文档索引

### HTTP API 文档

| 文档 | 路径 | 说明 |
|------|------|------|
| [system.md](./http/system.md) | `/health`, `/api/version` | 系统接口 |
| [auth.md](./http/auth.md) | `/api/v1/auth/*` | 认证接口 |
| [user.md](./http/user.md) | `/api/v1/users/*` | 用户接口 |
| [notifications.md](./http/notifications.md) | `/api/v1/notifications/*` | 通知接口 |
| [rooms.md](./http/rooms.md) | `/api/v1/rooms/*` | 聊天室接口 |
| [messages.md](./http/messages.md) | `/api/v1/messages/*` | 消息接口 |
| [files.md](./http/files.md) | `/api/v1/files/*`, `/api/v1/upload/*` | 文件接口 |
| [admin.md](./http/admin.md) | `/api/v1/admin/*` | 管理员接口 |
| [ui-config.md](./http/ui-config.md) | `/api/v1/ui/*` | UI 配置接口 |
| [oauth.md](./http/oauth.md) | `/oauth/*`, `/api/v1/oauth/*`, `/api/v1/rooms/*/resources/*` | OAuth 2.0 接口 |
| [webhook.md](./http/webhook.md) | `/api/v1/webhook/*` | Webhook 接口 |

### WebSocket 文档

| 文档 | 路径 | 说明 |
|------|------|------|
| [index.md](./websocket/index.md) | - | WebSocket 协议总览 |
| [auth.md](./websocket/auth.md) | `Auth`, `AuthResult` | 认证消息 |
| [room.md](./websocket/room.md) | `JoinRoom`, `LeaveRoom` | 房间管理消息 |
| [message.md](./websocket/message.md) | `ChatMessage`, `NewMessage` | 消息通信 |
| [notification.md](./websocket/notification.md) | `PrivateMessage`, `Mentioned` | 通知系统 |
| [user-status.md](./websocket/user-status.md) | `UpdateStatus` | 用户状态管理 |
| [system-message.md](./websocket/system-message.md) | `SystemMessage` | 系统广播消息 |
| [system-logs.md](./websocket/system-logs.md) | `SubscribeLogs` | 系统日志流 |

---

## 测试覆盖

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
| IP 安全测试 | `tests/ip_security_test.rs` | 15 | ✅ 通过 |
| 用户设置测试 | `tests/user_settings_test.rs` | 10 | ✅ 通过 |
| 账号安全测试 | `src/` 内嵌测试 | 6 | ✅ 通过 |
| Redis 集成测试 | `tests/redis_integration_test.rs` | 8 | ✅ 通过 |
| **私聊功能测试** | **`tests/direct_room_test.rs`** | **8** | **✅ 通过** |
| **好友功能测试** | **`tests/friendship_test.rs`** | **10** | **✅ 通过** |
| **房间邀请测试** | **`tests/room_invitation_test.rs`** | **9** | **✅ 通过** |
| **C1 注销账号测试** | **`tests/phaseC1_delete_account_test.rs`** | **4** | **✅ 通过** |
| **C2 消息置顶测试** | **`tests/phaseC2_pinned_messages_test.rs`** | **5** | **✅ 通过** |

---

## 环境变量配置

### 基础配置

| 变量名 | 必填 | 默认值 | 说明 |
|--------|------|--------|------|
| `DATABASE_URL` | 是 | - | PostgreSQL 连接地址 |
| `JWT_SECRET` | 是 | - | JWT 签名密钥 |
| `UPLOAD_DIR` | 否 | `./uploads` | 文件上传目录 |

### Redis 配置 (阶段 8.5+)

| 变量名 | 必填 | 默认值 | 说明 |
|--------|------|--------|------|
| `REDIS_ENABLED` | 否 | `false` | 是否启用 Redis |
| `REDIS_URL` | 启用时必填 | - | Redis 连接地址 |
| `REDIS_POOL_SIZE` | 否 | `10` | 连接池大小 |
| `REDIS_TIMEOUT_SECS` | 否 | `5` | 连接超时时间（秒） |
| `REDIS_CHANNEL_PREFIX` | 否 | `capella` | Pub/Sub 频道前缀 |
| `REDIS_STREAM_MAX_LEN` | 否 | `100000` | Stream 最大长度 |
| `REDIS_CONSUMER_BATCH_SIZE` | 否 | `100` | Consumer 批量消费大小 |
| `REDIS_CONSUMER_POLL_INTERVAL_MS` | 否 | `1000` | Consumer 消费间隔（毫秒） |
| `REDIS_CONFIG_SYNC_ENABLED` | 否 | `true` | 是否启用配置同步 |

---

## 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0 | 2026-04-26 | 初始版本，汇总阶段 1-8 所有功能 |
| v1.1 | 2026-05-07 | 添加阶段 9 功能：搜索、私聊、好友系统、房间邀请 |
| v1.2 | 2026-06-13 | 添加消息表情反应功能（HTTP API + WebSocket） |
| v1.3 | 2026-06-14 | 添加账号注销、消息置顶功能（HTTP API + WebSocket） |
| v1.4 | 2026-06-14 | 补充通知 HTTP API、分片上传、房间级设置、推荐用户、增强统计、系统监控、死信队列、Redis 管理、配置同步、OAuth 2.0、Webhook、自定义事件、资源绑定、v2 API、WS 消息类型补全 |

---

*本文档由开发路线文档自动生成，如有疑问请参考 [development-roadmap.md](../../development-roadmap.md)*
