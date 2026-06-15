# CapellaRoom 发展路线分析：从聊天引擎到实时社交基础设施

> **分析日期**: 2026-06-15
> **源文档**: [Perseus 产品路线图](../../../../python/perseus/docs/roadmap.md)
> **当前项目**: CapellaRoom (Rust/Axum)

---

## 一、Perseus 路线图对 CapellaRoom 的定位

Perseus 路线图将 CapellaRoom 定义为**授权 & 实时引擎**，是整个协作平台的**基础设施层**：

| 子系统 | 技术栈 | 对 CapellaRoom 的要求 |
|--------|--------|----------------------|
| **Perseus** | Python/FastAPI + Vue 3 | 将注册/登录委托给 CapellaRoom |
| **CapellaRoom** | Rust/Axum + PG/Redis | 认证中心、WS 实时引擎、团队聊天 |

### 职责矩阵 (来自路线图)

| 能力 | 所有权 | 当前 CapellaRoom 状态 |
|------|--------|---------------------|
| 用户认证授权 (OAuth/SSO) | CapellaRoom 核心 | ✅ JWT, 尚缺 OAuth 协议 |
| WebSocket 实时通信 | CapellaRoom 核心 | ✅ 完备协议, 40+ 事件类型 |
| 团队聊天 | CapellaRoom API+WS | ✅ 完备 |
| 协作文本编辑 | CapellaRoom 实时同步 | ❌ 未实现 |
| 在线状态 / 设备管理 | CapellaRoom | ✅ 完备 |
| 通知推送管道 | CapellaRoom | ✅ 完备 |
| 文件上传(附件) | CapellaRoom | ✅ 完备 |

### 对接阶段要求

**Phase 0 (基础设施对接)** — CapellaRoom 需要暴露：
- OAuth-like 授权端点 (code → token 交换)
- 用户信息查询 API (供 Perseus 同步)
- 统一 JWT 策略

**Phase 2 (实时协作)** — CapellaRoom 需要扩展：
- 仓库↔聊天室关联 API
- 业务事件推送 (PR/Issue/Review)
- 在线成员可见

---

## 二、现状分析：CapellaRoom 当前能力全景

### 已实现的完整模块

```
认证系统      ✅ 注册/登录/JWT/刷新/登出/密码修改
用户管理      ✅ CRUD/搜索/好友/设备/登录历史
房间系统      ✅ CRUD/成员/角色/邀请码/直聊
消息系统      ✅ 发送/编辑/删除/回复/搜索/固定/反应
WebSocket     ✅ 40+ 事件类型/心跳/重连/离线消息
通知系统      ✅ 私信/@提及/系统通知/待办审批
文件上传      ✅ 分类存储/SHA-256 去重/权限控制
在线状态      ✅ 实时跟踪/状态广播/持久化
管理后台      ✅ 用户/房间/消息/配置/Redis 管理
审计系统      ✅ 操作日志/安全告警/规则引擎/分区
安全防护      ✅ IP 黑白名单/CIDR/WS 层防护
分布式        ✅ Redis Pub/Sub + Stream/配置同步
监控          ✅ 健康检查/系统指标/性能定时器
```

### 与路线图要求的关键差距

| 需求 | 当前状态 | 差距分析 |
|------|---------|---------|
| **OAuth 授权端点** | 仅有内部 auth | 无 code → token 交换流程 |
| **外部应用注册** | 无 | 无 client_id/client_secret |
| **用户身份映射** | 无 | Perseus 用户 ID ↔ CapellaRoom 用户 ID |
| **业务事件扩展** | WS 协议固定 | 无法自定义事件类型 |
| **出站 Webhook** | 无 | 无法将事件推送到外部服务 |
| **SSO/社交登录** | 仅 email+password | 无 OAuth provider 集成 |
| **协作文本编辑** | 无 | OT/CRDT 实时协同编辑 |
| **客户端 SDK** | 仅有调试前端 | 无正式 SDK 包 |

---

## 三、发展战略：从聊天引擎到实时社交基础设施

### 核心命题

> **CapellaRoom 不应仅仅是"聊天应用"，而应成为"实时社交能力提供商"**——为 Perseus 以及其他任何服务/应用提供开箱即用的实时通信和社交功能。

### 架构演进

```
当前:    浏览器 → CapellaRoom (自包含应用)
目标:    浏览器 → Perseus ──→ CapellaRoom (基础设施)
               其他 App ──→ CapellaRoom (基础设施)
```

### 战略三阶段

---

## Phase A：服务化 (Service Enablement)

**目标**: 将 CapellaRoom 从"自包含应用"变为"可被外部服务集成的基础设施"

### A-01：外部应用注册系统

**描述**: 其他服务可以在 CapellaRoom 中注册为"应用"，获取 client_id 和 client_secret。

**新增模型**: `oauth_apps` 表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID PK | 应用标识 (client_id) |
| name | VARCHAR | 应用名称 (如 "Perseus") |
| description | TEXT | 应用描述 |
| client_secret | VARCHAR | 加密存储 |
| redirect_uris | TEXT[] | 允许的回调 URI |
| scopes | TEXT[] | 请求的权限范围 |
| owner_id | UUID FK → users | 应用所有者 |
| is_active | BOOLEAN | 是否启用 |
| logo_url | VARCHAR | 应用图标 |
| created_at | TIMESTAMPTZ | |

**API**:
```
POST   /api/v1/oauth/apps                    # 注册应用
GET    /api/v1/oauth/apps                    # 我的应用列表
PUT    /api/v1/oauth/apps/:app_id            # 更新应用
DELETE /api/v1/oauth/apps/:app_id            # 删除应用
POST   /api/v1/oauth/apps/:app_id/secret     # 重新生成 secret
```

### A-02：OAuth 2.0 授权码流程

**描述**: 实现标准的 OAuth 2.0 Authorization Code Grant，让 Perseus 等外部服务可以委托 CapellaRoom 进行用户认证。

**流程**:
```
用户 → Perseus 登录页
    → "使用 CapellaRoom 登录"
    → 跳转 CapellaRoom /oauth/authorize?client_id=...&redirect_uri=...&response_type=code
    → 用户确认授权
    → 返回 callback URL 附带 code
    → Perseus 后端用 code + client_secret 调用 /oauth/token 换取 access_token
```

**新增端点**:
```
GET    /oauth/authorize                       # 用户授权页
POST   /oauth/token                           # code → token 交换
GET    /api/v1/oauth/userinfo                 # 用户信息端点 (供资源服务使用)
```

**新增模型**: `oauth_authorization_codes` 表 (短时效，用完即删)

| 字段 | 类型 | 说明 |
|------|------|------|
| code | VARCHAR PK | 授权码 |
| app_id | UUID FK | 对应的应用 |
| user_id | UUID FK | 授权用户 |
| scopes | TEXT[] | 授权范围 |
| redirect_uri | TEXT | 回调 URI |
| expires_at | TIMESTAMPTZ | 过期时间 (通常 10 分钟) |
| used_at | TIMESTAMPTZ | 使用时间 (用完即删) |

### A-03：用户身份映射

**描述**: 外部服务 (如 Perseus) 有自己的用户 ID 体系，CapellaRoom 需要提供双向映射能力。

**机制**:
- 通过 OAuth userinfo 端点返回的 `sub` 可以是 CapellaRoom 的 user_id
- 外部服务在本地存储 外键 user_id ↔ CR user_id 的映射
- 或者 CapellaRoom 支持 `external_user_id` 字段，允许外部服务注册用户时指定

**新增模型**: `user_identity_mappings` 表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID PK | |
| user_id | UUID FK → users | CapellaRoom 用户 ID |
| app_id | UUID FK → oauth_apps | 外部应用 |
| external_user_id | VARCHAR | 外部服务的用户 ID |
| external_username | VARCHAR | 外部用户名 (缓存) |
| mapped_at | TIMESTAMPTZ | |

**API**:
```
POST   /api/v1/oauth/mappings                 # 创建映射 (由外部服务调用)
GET    /api/v1/oauth/mappings                 # 查询映射
DELETE /api/v1/oauth/mappings/:id             # 解除映射
```

### A-04：出站 Webhook 系统

**描述**: CapellaRoom 将实时事件通过 Webhook 推送给外部服务，让 Perseus 等应用可以接收到离线事件并做业务处理。

**新增模型**: `webhook_subscriptions` 表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID PK | |
| app_id | UUID FK → oauth_apps | 订阅的应用 |
| url | TEXT | Webhook 回调 URL |
| secret | VARCHAR | 签名密钥 (HMAC-SHA256) |
| events | TEXT[] | 订阅的事件类型 |
| is_active | BOOLEAN | |
| retry_count | INT | 最大重试次数 |
| retry_interval | INT | 重试间隔 (秒) |
| timeout_ms | INT | 请求超时 |
| created_at | TIMESTAMPTZ | |

**新增模型**: `webhook_deliveries` 表 (投递记录)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID PK | |
| subscription_id | UUID FK | |
| event_type | TEXT | |
| payload | JSONB | 事件内容 |
| status | ENUM (pending/success/failed) | |
| response_code | INT | HTTP 状态码 |
| response_body | TEXT | |
| attempts | INT | 已重试次数 |
| next_retry_at | TIMESTAMPTZ | |
| created_at | TIMESTAMPTZ | |

**Webhook 事件类型**:
```
message.created      # 新消息
message.edited       # 消息编辑
message.deleted      # 消息删除
room.user_joined     # 用户加入房间
room.user_left       # 用户离开房间
user.status_changed  # 用户在线状态变更
user.mentioned       # @提及
notification.created # 通知创建
```

**安全**: 每个 webhook 请求携带 `X-Capella-Signature: sha256=<HMAC>` 签名头。

**API**:
```
POST   /api/v1/webhooks                       # 创建 Webhook 订阅
GET    /api/v1/webhooks                       # 订阅列表
PUT    /api/v1/webhooks/:id                   # 更新订阅
DELETE /api/v1/webhooks/:id                   # 删除订阅
GET    /api/v1/webhooks/:id/deliveries        # 投递记录
POST   /api/v1/webhooks/:id/redeliver/:did    # 重新投递
```

---

## Phase B：生态化 (Ecosystem Expansion)

**目标**: 扩展 CapellaRoom 的能力，使其能覆盖更广泛的实时社交场景，成为通用基础设施。

### B-01：SSO / 社交登录集成

**描述**: CapellaRoom 自身支持通过第三方 OAuth 提供商 (GitHub、Google、GitLab 等) 登录，同时也能作为 SSO 提供商服务于其他应用。

**能力**:
- 作为 **Relying Party**: 用户可通过 GitHub/Google 等登录 CapellaRoom
- 作为 **Identity Provider**: 通过 CapellaRoom 登录的用户可 SSO 到 Perseus

**新增模型**: `oauth_providers` 表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID PK | |
| provider | VARCHAR | github/google/gitlab/etc |
| client_id | VARCHAR | |
| client_secret | VARCHAR | 加密存储 |
| scopes | TEXT[] | |
| is_enabled | BOOLEAN | |
| created_at | TIMESTAMPTZ | |

**新增模型**: `user_oauth_links` 表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID PK | |
| user_id | UUID FK → users | |
| provider | VARCHAR | |
| provider_user_id | VARCHAR | |
| provider_username | VARCHAR | |
| linked_at | TIMESTAMPTZ | |

**API**:
```
GET    /api/v1/auth/sso/github                # 跳转 GitHub OAuth
GET    /api/v1/auth/sso/github/callback       # GitHub OAuth 回调
GET    /api/v1/auth/sso/google                # 跳转 Google OAuth
GET    /api/v1/auth/sso/google/callback       # Google OAuth 回调
GET    /api/v1/users/me/sso-links             # 已绑定的 SSO 账号
POST   /api/v1/users/me/sso-links/:id/unlink  # 解绑 SSO 账号
```

### B-02：自定义事件类型

**描述**: 允许外部服务通过 API 注册自定义 WebSocket 事件类型，使得 Perseus 可以将 PR/Issue/Review 等业务事件通过 CapellaRoom 的 WS 通道实时推送给前端。

**实现方式**:
- 外部服务通过 API 发送 `CustomEvent` WS 消息
- CapellaRoom 验证发送者有权限后，广播给房间内的所有客户端
- 自定义事件使用 `"type": "Custom"` + `event_name` 字段区分

**新增 WS 消息类型**:
```json
// 外部服务 → CapellaRoom → 客户端 (透传)
{
  "type": "CustomEvent",
  "payload": {
    "event_name": "perseus:pr_updated",
    "room_id": "uuid",
    "data": {
      "pr_id": 42,
      "status": "merged",
      "title": "Fix login bug"
    }
  }
}
```

**安全控制**:
- 只有通过 OAuth 认证的外部服务可以发送自定义事件
- 自定义事件名需要符合 `{app_name}:{event_type}` 命名空间约定
- 可选的 schema 验证 (JSON Schema)

### B-03：房间-资源关联

**描述**: 将 CapellaRoom 的房间与外部服务的业务资源 (如 Perseus 的仓库/PR/Issue) 关联起来，使房间具有"业务上下文"。

**新增模型**: `room_resource_bindings` 表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID PK | |
| room_id | UUID FK → rooms | |
| app_id | UUID FK → oauth_apps | 外部应用 |
| resource_type | VARCHAR | 如 "repository" / "pull_request" |
| resource_id | VARCHAR | 外部资源 ID |
| resource_url | TEXT | 外部资源链接 |
| resource_name | VARCHAR | 资源名称 (缓存) |
| created_at | TIMESTAMPTZ | |

**API**:
```
POST   /api/v1/rooms/:room_id/resources       # 绑定资源
GET    /api/v1/rooms/:room_id/resources       # 资源列表
DELETE /api/v1/rooms/:room_id/resources/:id   # 解绑资源
```

### B-04：官方客户端 SDK

**描述**: 提供多语言 SDK，让外部服务开发者可以在 5 分钟内完成 CapellaRoom 集成。

**首批 SDK**:
- **JavaScript/TypeScript** — 浏览器端 (WS 客户端 + HTTP 客户端)
- **Python** — 后端服务集成 (Perseus 场景)
- **Rust** — 高性能集成

**SDK 核心能力**:
```
connect(token)             # 连接 WebSocket
auth(code)                 # OAuth 授权码获取 token
joinRoom(roomId)           # 加入房间
sendMessage(roomId, text)  # 发送消息
onMessage(handler)         # 监听消息
onCustomEvent(handler)     # 监听自定义事件
sendCustomEvent(event)     # 发送自定义事件
subscribeWebhook(events)   # 订阅 Webhook
```

---

## Phase C：协同化 (Collaboration Platform)

**目标**: 增加实时协同编辑等高级能力，使 CapellaRoom 成为真正的多人实时协作平台。

### C-01：协作文本编辑器

**描述**: 基于 CRDT (Conflict-free Replicated Data Types) 或 OT (Operational Transform) 的实时多人协同编辑能力。

**实现方案**: 使用 Yjs (CRDT) 或类似算法，通过 CapellaRoom 的 WebSocket 管道传输编辑操作。

**架构**:
```
用户 A 输入 → Yjs 生成 Op → WS → CapellaRoom → Redis Pub/Sub → WS → 用户 B 应用 Op
                    → WS → 用户 C 应用 Op
```

**集成方式**:
- CapellaRoom 作为 Op 的中继层，不关心 Op 内容
- 客户端使用 Yjs 等 CRDT 库处理文档同步
- 通过自定义事件传输编辑操作

**新增 WS 消息类型**:
```json
// 协同编辑操作
{
  "type": "CollabOp",
  "payload": {
    "doc_id": "uuid",
    "ops": [...],
    "version": 42
  }
}

// 文档状态快照
{
  "type": "CollabSnapshot",
  "payload": {
    "doc_id": "uuid",
    "content": "...",
    "version": 42
  }
}
```

### C-02：富媒体消息

**描述**: 在现有文本/图片/文件消息基础上，增加富媒体消息类型：

- 代码片段 (语法高亮)
- Markdown 渲染
- 嵌入式视频/音频
- 卡片消息 (链接预览)
- 消息分片 (长消息自动分片)

---

## 四、与 Perseus 路线图的对应关系

| Perseus Phase | Perseus 任务 | 依赖的 CapellaRoom 能力 | 对应 Phase |
|:---:|---|:---|---:|
| P-001 | OAuth-like 授权端点 | OAuth 授权码流程 + 应用注册 | **A-01 + A-02** |
| P-002 | 用户信息查询 API | OAuth userinfo 端点 | **A-02** |
| P-003 | 认证客户端 SDK | Python SDK | **B-04** |
| P-004 | 统一 JWT 策略 | JWT 配置共享 / Token 交换 | **A-02** |
| P-005 | 部署整合 | docker-compose 网络互通 | 运维 |
| P-006 | Perseus 认证替换 | OAuth 流程 + 身份映射 | **A-03** |
| F-201 | 项目房间映射 | 房间-资源关联 API | **B-03** |
| F-203 | 业务事件推送 | 自定义事件类型 | **B-02** |
| F-204 | 协作文本编辑器 | 协同编辑能力 | **C-01** |

---

## 五、实施优先级建议

### 第一优先 (Phase A — 完成基础设施对接)

```
A-01: 外部应用注册    ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  1 周
A-02: OAuth 授权流程   ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  2 周
A-03: 用户身份映射      ⬛⬛⬛⬛⬛⬛⬛  1 周
A-04: 出站 Webhook     ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  2 周
```

**总计**: 约 6 周 → 可实现 Perseus Phase 0 完全对接

### 第二优先 (Phase B — 扩展生态能力)

```
B-01: SSO 社交登录      ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  2 周
B-02: 自定义事件类型     ⬛⬛⬛⬛⬛⬛  1 周
B-03: 房间资源关联       ⬛⬛⬛⬛⬛⬛  1 周
B-04: 客户端 SDK         ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  3 周
```

**总计**: 约 7 周 → 可实现 Perseus Phase 2 完全支撑

### 第三优先 (Phase C — 协同能力增强)

```
C-01: 协同编辑          ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  4 周
C-02: 富媒体消息         ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  2 周
```

**总计**: 约 6 周

---

## 六、关键技术决策

### 1. OAuth vs 自定义协议

**推荐**: OAuth 2.0 Authorization Code Grant (标准协议)
- 开发者无需学习专有协议
- 社区工具链完善 (大量 OAuth 客户端库)
- Perseus 可重用标准的 OAuth 中间件
- 未来可扩展 PKCE、Client Credentials 等流程

### 2. Webhook 签名

**推荐**: HMAC-SHA256 + 时间戳
```
signature = HMAC-SHA256(secret, timestamp + "." + body)
```
接收方验证签名并拒绝超过 5 分钟的请求 (防重放攻击)。

### 3. CRDT vs OT

**推荐**: Yjs (CRDT)
- 无需中心化服务端做操作转换
- 天然支持离线编辑
- Rust 有 `yrs` crate
- CapellaRoom 只需做 Op 中继，不关心文档内容

### 4. 多语言 SDK 优先级

1. **TypeScript** — 浏览器端和 Node.js (Perseus 前端和 Perseus 后端都会用到)
2. **Python** — Perseus 后端直接集成
3. **Rust** — 高性能场景

---

## 七、总结

CapellaRoom 当前的能力已经远超一个"聊天室应用"的范畴——它拥有完备的认证系统、实时通信协议、用户社交关系、设备管理、安全审计、分布式支持等能力。

**核心使命转变**:
> ~~Capella Room 是一个基于 Axum + WebSocket + PostgreSQL 构建的高性能实时聊天室应用。~~
>
> **CapellaRoom 是一个实时社交基础设施，为 Perseus 及其他服务/应用提供认证授权、实时通信、在线状态、消息推送、文件存储等开箱即用的社交能力。**

通过 Phase A (服务化) 完成 OAuth 协议和身份映射，Perseus 等外部服务可以像使用 Auth0/Firebase 一样使用 CapellaRoom。通过 Phase B (生态化) 和 Phase C (协同化)，CapellaRoom 将成为完整的实时协作平台底座。

---

*文档版本: 1.0.0*
*最后更新: 2026-06-15*
