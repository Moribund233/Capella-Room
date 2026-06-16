# CapellaRoom v2 开发路线图

> **更新日期**: 2026-06-16
> **定位**: 从实时聊天应用 → 实时社交基础设施
> **架构**: CapellaRoom v2 = 规范化认证 + 消息安全体系 + 授权生态扩展
> **当前进度**: Phase 1 ✅ | 1.1 ✅ / 1.2 ✅ / 1.3 ✅ / 1.4 ✅ / 1.5 ✅

---

## 总体目标

v2 在 v1 完备的聊天引擎基础之上，完成三大升级：

| 维度 | 目标 | 对应阶段 |
|------|------|---------|
| **认证体系规范化** | 引入真实邮件服务 + 验证码注册/登录/找回密码，v1 注册端点转为 admin 测试端点 | Phase 1 |
| **消息安全体系** | 消息体存储加密 + 智能审核管线（同步拦截 + 异步 AI 审核） | Phase 2 |
| **授权与生态扩展** | OAuth 2.0 提供商 + 出站 Webhook + 自定义 WS 事件 + 房间资源绑定 + SSO 社交登录 | Phase 3 |

---

## Phase 1：认证体系规范化（4 周）

**目标**: 用真实邮件服务替代 v1 的裸注册登录，形成完整的安全认证闭环

### 现状与问题

v1 的认证流程过于简单：
- 注册仅需 username + email + password，**无邮箱验证**
- 登录仅支持 email + password，**无验证码/无多因素**
- 找回密码**未实现**
- 注册端点对外开放，无法区分"用户注册"和"admin 创建账号"

### 1.1 邮件服务基础设施（1 周）✅ 已完成

**新增配置**:

```toml
[mail]
smtp_host = "smtp.example.com"
smtp_port = 587
smtp_username = "noreply@capella.local"
smtp_password = "${SMTP_PASSWORD}"    # 环境变量注入
smtp_use_tls = true
from_address = "noreply@capella.local"
from_name = "CapellaRoom"
verification_code_ttl = 10            # 验证码有效期（分钟）
```

**新增模块**: `src/services/mail_service.rs`

```rust
pub struct MailService {
    config: MailConfig,
    // 可选：对接 SendGrid / Mailgun / SMTP
}

impl MailService {
    /// 发送验证码邮件
    pub async fn send_verification_code(&self, email: &str, code: &str) -> Result<()>;

    /// 发送密码重置邮件
    pub async fn send_password_reset(&self, email: &str, token: &str) -> Result<()>;

    /// 发送欢迎邮件（注册成功通知）
    pub async fn send_welcome(&self, username: &str, email: &str) -> Result<()>;
}
```

**实现策略**: 适配器模式，支持多种邮件发送后端

| 后端 | 集成方式 | 适用场景 |
|------|---------|---------|
| **SMTP** | `lettre` crate | 自建邮件服务器 |
| **SendGrid** | HTTP API | 云服务（推荐） |
| **Mailgun** | HTTP API | 云服务 |
| **控制台** | 直接输出到日志 | 开发/测试环境 |

### 1.2 验证码生成与存储（0.5 周）✅ 已完成

**新增模型**: `verification_codes`

```sql
CREATE TABLE verification_codes (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email      VARCHAR(255) NOT NULL,
    code       VARCHAR(8) NOT NULL,        -- 6-8 位数字/字母
    purpose    VARCHAR(32) NOT NULL,        -- 'register' / 'login' / 'reset_password'
    expires_at TIMESTAMPTZ NOT NULL,
    used_at    TIMESTAMPTZ,                -- 使用后标记，防止重复消费
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    INDEX idx_vc_email_purpose (email, purpose, created_at DESC)
);
```

**验证码安全策略**:
- 6 位数字验证码（误输率低且方便移动端输入）
- 有效期 10 分钟（配置化）
- 同一邮箱同一用途 60 秒内不能重复发送（防轰炸）
- 连续 5 次验证失败锁定该邮箱 30 分钟（防暴破）
- 验证后标记 `used_at`，不可二次使用

### 1.3 v2 认证 API（1.5 周）✅ 已完成

**实现状态**: 9 个端点全部实现，5 个集成测试通过（112 单元测试通过，0 失败）

| 端点 | 状态 | 说明 |
|------|------|------|
| `POST /api/v2/auth/register/send-code` | ✅ | 验证邮箱 → 创建验证码 → 邮件发送 |
| `POST /api/v2/auth/register` | ✅ | 验证码校验 → 用户创建 → JWT + 会话 |
| `POST /api/v2/auth/login/send-code` | ✅ | 验证邮箱已注册 → 发送验证码 |
| `POST /api/v2/auth/login` | ✅ | 验证码登录 → 单设备会话 + JWT |
| `POST /api/v2/auth/reset-password/send-code` | ✅ | 验证邮箱 → 发送重置验证码 |
| `POST /api/v2/auth/reset-password` | ✅ | 验证码 → 密码更新 |
| `POST /api/v2/auth/login-with-password` | ✅ | 兼容模式，含设备信息录入 |
| `POST /api/v2/auth/refresh` | ✅ | Refresh JWT → 新 Token 对 |
| `POST /api/v2/users/logout` | ✅ | 登出（token 失效待 v2 完善） |

**关键实现细节**:
- `MailService` 使用控制台后端（日志输出），适配器模式预留 SMTP/SendGrid/Mailgun 扩展
- `VerificationCodeService` 实现 60s 防轰炸冷却、10 分钟有效期（配置化）、单次消费
- 数据库迁移 `017_verification_codes.sql` 含索引 `(email, purpose, created_at DESC)`
- `BatchMessageConfig` 整改：移除代码默认值，缺失/零值由 `validate_config()` 拒绝启动
- 共享辅助函数 `issue_auth_tokens()` 集中处理登录礼仪（JWT + 会话 + 审计）

### 1.4 v1 注册端点迁移（0.5 周）✅ 已完成

```
v1: POST /api/v1/auth/register  →  保留但限制为 admin 角色可调用的内部测试端点
v1: POST /api/v1/auth/login     →  保留，保持向后兼容
v1: POST /api/v1/auth/refresh   →  保留

v2 上线后外部用户注册必须走 /api/v2/auth/register + 邮箱验证码
```

**变更内容**:
- `/api/v1/auth/register` 从公共路由中移除，移至独立 `register_admin_router`，叠加 `auth_middleware` + `admin_auth_middleware`
- 非管理员请求返回 403 FORBIDDEN，未认证请求返回 401
- v1 login/refresh 保持不变（不影响现有客户端）
- 4 个 TDD 测试覆盖：无认证、非 admin、正确 admin、保障 v1 login 不受影响

### 1.5 用户表扩展（0.5 周）✅ 已完成

```sql
ALTER TABLE users
  ADD COLUMN email_verified_at  TIMESTAMPTZ,     -- 邮箱验证时间，NULL=未验证
  ADD COLUMN email_verified     BOOLEAN NOT NULL DEFAULT false;
```

**变更内容**:
- 迁移 `018_add_email_verified_to_users.sql`：新增 `email_verified BOOLEAN NOT NULL DEFAULT false`、`email_verified_at TIMESTAMPTZ`
- `src/models/user.rs`：新增字段 `email_verified: bool`、`email_verified_at: Option<DateTime<Utc>>`，使用 `#[sqlx(default)]` 保证兼容性
- `src/services/user_service.rs`：全部 14 处 `query_as::<_, User>` 的 SQL 查询更新 SELECT/RETURNING 子句
- v1 API 的 `UserResponse` 不暴露 `email_verified` 字段
- 2 个 TDD 测试覆盖：新用户创建 `email_verified = false`、User 结构体字段存在性验证

---

## Phase 2：消息安全体系（5 周）

**目标**: 消息存储加密 + 智能审核管线，在保障安全的同时保持搜索和审核能力

### 2.1 消息加密（3 周）

**加密范围**: 仅对 `messages.content` 字段做存储层加密。WebSocket 传输仍走 TLS（已有），WebSocket 广播给在线用户仍使用明文（客户端不解密）。

**架构**: Envelope Encryption（信封加密）

```
Master Key (KEK) ── 环境变量/Vault，永不落盘
    └── Room DEK ── 每个房间独立，加密后存 room_encryption_keys 表
         └── Per-Message Key ── HKDF(DEK, message_id || nonce)，用完即焚
```

**新增模型**:

```sql
CREATE TABLE room_encryption_keys (
    room_id        UUID PRIMARY KEY REFERENCES rooms(id) ON DELETE CASCADE,
    dek_ciphertext BYTEA NOT NULL,           -- KEK 加密后的 DEK
    dek_version    INT NOT NULL DEFAULT 1,
    algorithm      VARCHAR(20) NOT NULL DEFAULT 'aes-256-gcm',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at     TIMESTAMPTZ
);
```

**Messages 表变更**:

```sql
ALTER TABLE messages
  ADD COLUMN ciphertext    BYTEA,             -- AES-GCM 密文 (含 auth_tag)
  ADD COLUMN nonce         BYTEA,             -- 12 bytes random nonce
  ADD COLUMN key_version   INT DEFAULT 1,
  ADD COLUMN is_encrypted  BOOLEAN DEFAULT false;
```

**新增 Service**: `MessageCryptoService`

```rust
pub struct MessageCryptoService {
    kek: Zeroizing<[u8; 32]>,          // Master Key，进程启动时加载
    dek_cache: Arc<DashMap<(Uuid, i32), Zeroizing<[u8; 32]>>>,
    db: Database,
}

impl MessageCryptoService {
    pub async fn encrypt(room_id, message_id, plaintext) -> Result<EncryptedPayload>;
    pub async fn decrypt(room_id, message_id, payload) -> Result<Vec<u8>>;
    pub async fn rotate_room_key(room_id) -> Result<()>;  // 密钥轮换
}
```

**加密流程**:

```
Handler 收到 ChatMessage
  → 同步 L1 敏感词检查（明文阶段，检查后才加密）
  → crypto_service.encrypt(room_id, msg_id, content)
  → INSERT INTO messages (ciphertext, nonce, key_version)
  → 用原始明文广播给房间在线用户（WebSocket 已有 TLS）
  → Redis Stream 中传输密文
```

**搜索适配**: `content_tsv` 全文索引不再适用。改造策略：

```
搜索 API:
  1. 用户输入关键词 → 搜索服务从 Meilisearch/内存索引 搜索 message_id
  2. 回 PG 根据 message_id 查找密文
  3. crypto_service.decrypt() 解密
  4. 返回明文给用户
```

**适用范围**: 所有房间都默认启用加密（无明文存储选项）。秘密聊天（E2EE）暂不纳入 v2。

### 2.2 消息智能审核（2 周）

**架构**: L1 同步拦截 + L2/L3 异步审核 + 乐观发送

#### L1：同步敏感词拦截（发送前）

```
ChatMessage 到达 WebSocket Handler
  → moderation_service.check_sync(content)
  → 命中 L1 规则 → 直接返回 Error { code: "CONTENT_BLOCKED", message: "..." }
  → 未命中 → 继续加密 + 存储 + 广播流程
```

**L1 技术选型**:

| 方案 | 性能 | 集成复杂度 | 推荐 |
|------|------|-----------|------|
| DFA/AC 自动机 | <1ms/条 | 低 | ✅ v2 采用 |
| 正则表达式 | 1-5ms/条 | 低 | 辅助 |

**实现**: 敏感词列表从 `system_configs` 表加载（支持热更新），AC 自动机构建 DFA 状态机，运行时 `O(n)` 匹配。

#### L2：异步语义审核

```
消息持久化后 → 通过 Redis Stream 推送审核任务
                    ↓
           Audit Consumer Group
              ↙           ↘
       L2 本地模型          L3 外部 API
       (敏感词变体/          (AI 大模型
        情绪分析)             语义理解)
              ↘           ↙
           审核结果聚合
              ↓
        通过 → 无操作
        可疑 → 标记消息，审核后台可见
        违规 → 撤回消息，notice 发送者
```

**L2 实现**:
- v2 先实现 L2 框架 + 回调通知机制
- 具体的 L2/L3 审核模型接入作为可插拔模块（预留接口）
- L2 默认使用规则引擎（扩展关键词、频率检测），不强制依赖外部 ML 模型

#### 乐观发送体验

```
用户发送 → 消息立即显示在聊天框（乐观渲染）
         → 同步发送到服务端
         → L1 命中 → 服务端返回 Error → 客户端灰显/替换消息
         → L1 通过 → 消息正常广播
         → L2 判定违规 → 服务端推送 "MessageDeleted" → 客户端灰显
```

**新增配置**:
```toml
[moderation]
l1_enabled = true
l1_word_list = []              # 从 system_configs 加载
l2_enabled = false             # v2 默认关闭，按需开启
l2_consumer_batch_size = 50
```

---

## Phase 3：授权与生态扩展（6 周）

**目标**: CapellaRoom 成为可供 Perseus 等外部服务集成的实时社交基础设施

### 3.1 外部应用注册（1 周）

**新增模型**: `oauth_apps`

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID PK | 作为 client_id |
| name | VARCHAR(128) | 应用名称 |
| description | TEXT | |
| client_secret | VARCHAR | 加密存储 |
| redirect_uris | TEXT[] | OAuth 回调 URI |
| scopes | TEXT[] | 权限范围 |
| owner_id | UUID FK → users | |
| is_active | BOOLEAN | |
| created_at | TIMESTAMPTZ | |

**API**:
```
POST   /api/v2/oauth/apps                        # 注册应用
GET    /api/v2/oauth/apps                        # 我的应用列表
GET    /api/v2/oauth/apps/:app_id                # 应用详情
PUT    /api/v2/oauth/apps/:app_id                # 更新
DELETE /api/v2/oauth/apps/:app_id                # 删除
POST   /api/v2/oauth/apps/:app_id/secret         # 重新生成 secret
```

### 3.2 OAuth 2.0 授权码流程（1.5 周）

**协议**: OAuth 2.0 Authorization Code Grant

**端点**:
```
GET  /oauth/authorize?response_type=code&client_id=...&redirect_uri=...&state=...
POST /oauth/token       (grant_type=authorization_code / refresh_token)
GET  /api/v2/oauth/userinfo
     Authorization: Bearer <access_token>
```

**用户身份映射**:

```sql
CREATE TABLE user_identity_mappings (
    id                UUID PRIMARY KEY,
    user_id           UUID NOT NULL REFERENCES users(id),
    app_id            UUID NOT NULL REFERENCES oauth_apps(id),
    external_user_id  VARCHAR(255) NOT NULL,
    external_username VARCHAR(255),
    mapped_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, external_user_id)
);
```

```
POST   /api/v2/oauth/mappings                    # 创建映射
GET    /api/v2/oauth/mappings/lookup             # 反查
DELETE /api/v2/oauth/mappings/:id                # 解除
```

### 3.3 出站 Webhook（1.5 周）

**事件列表**: `message.created`, `message.edited`, `message.deleted`, `room.user_joined`, `room.user_left`, `user.status_changed`, `user.mentioned`, `reaction.added`, `reaction.removed`

**投递格式**:
```
POST <webhook_url>
X-Capella-Signature: sha256=HMAC-SHA256(secret, timestamp + "." + body)
X-Capella-Timestamp: <unix_ts>
X-Capella-Delivery-Id: <uuid>
X-Capella-Event-Type: message.created

{ "event": "message.created", "event_id": "...", "payload": {...} }
```

**重试**: 指数退避（10s → 20s → 40s），最多 3 次。后台每 30s 扫描重试失败投递。

**API**:
```
POST   /api/v2/webhooks                     # 创建订阅
GET    /api/v2/webhooks                     # 列表
PUT    /api/v2/webhooks/:id                 # 更新
DELETE /api/v2/webhooks/:id                 # 删除
GET    /api/v2/webhooks/:id/deliveries      # 投递历史
POST   /api/v2/webhooks/:id/redeliver/:did  # 重试
```

### 3.4 自定义 WS 事件类型（1 周）

**理念**: CapellaRoom 不做业务理解，只做权限验证 + 中继

```
Perseus 服务端 → WS CustomEvent → CapellaRoom 验证 → 广播 CustomEventForward → 房间客户端
                    │                    │
              事件名必须以          检查发送者是
              "perseus:" 开头       房间成员
```

**WS 扩展**:
```rust
// 外部服务 → CapellaRoom
CustomEvent { event_name: String, room_id: Uuid, data: Value, persistent: Option<bool> }

// CapellaRoom → 客户端
CustomEventForward { event_name, room_id, source_app, data, timestamp }
```

可选持久化 + 重连补拉、HTTP API 兜底（`POST /api/v2/events/custom`）。

### 3.5 房间-资源绑定（0.5 周）

```sql
CREATE TABLE room_resource_bindings (
    id              UUID PRIMARY KEY,
    room_id         UUID REFERENCES rooms(id),
    app_id          UUID REFERENCES oauth_apps(id),
    resource_type   VARCHAR(64),
    resource_id     VARCHAR(255),
    resource_url    TEXT,
    resource_name   VARCHAR(255),
    metadata        JSONB DEFAULT '{}',
    UNIQUE (app_id, resource_type, resource_id)
);
```

**API**:
```
POST   /api/v2/rooms/:room_id/resources           # 绑定
DELETE /api/v2/rooms/:room_id/resources/:id       # 解绑
GET    /api/v2/rooms/:room_id/resources           # 列表
GET    /api/v2/resources/lookup                   # 反查
PATCH  /api/v2/rooms/:room_id/resources/:id       # 更新元数据
```

### 3.6 SSO 社交登录（0.5 周）

**支持的提供商**: GitHub / Google / GitLab

**端点**:
```
GET /api/v2/auth/sso/github
GET /api/v2/auth/sso/github/callback
GET /api/v2/auth/sso/google
GET /api/v2/auth/sso/google/callback
GET /api/v2/auth/sso/gitlab
GET /api/v2/auth/sso/gitlab/callback
```

**用户侧**:
```
GET    /api/v2/users/me/sso-links
DELETE /api/v2/users/me/sso-links/:id
```

**流程**: 跳转第三方 → 授权回调 → 查找/创建 CapellaRoom 账号 → 颁发 JWT

---

## 阶段总览

| Phase | 模块 | 预估工时 | 状态 | 依赖 |
|:---:|---|:---:|:---:|:---:|
| **1** | **认证体系规范化** | **4 周** | ✅ **完成** | 无 |
| 1.1 | 邮件服务基础设施 | 1 周 | ✅ 完成 | 无 |
| 1.2 | 验证码生成与存储 | 0.5 周 | ✅ 完成 | 1.1 |
| 1.3 | v2 认证 API | 1.5 周 | ✅ 完成 | 1.2 |
| 1.4 | v1 注册端点迁移 | 0.5 周 | ✅ 完成 | 1.3 |
| 1.5 | 用户表扩展 | 0.5 周 | ✅ 完成 | 1.3 |
| **2** | **消息安全体系** | **5 周** | 无 |
| 2.1 | 消息加密 | 3 周 | 无 |
| 2.2 | 消息智能审核 | 2 周 | 2.1 |
| **3** | **授权与生态扩展** | **6 周** | Phase 1 |
| 3.1 | 外部应用注册 | 1 周 | 无 |
| 3.2 | OAuth 2.0 + 身份映射 | 1.5 周 | 3.1 |
| 3.3 | 出站 Webhook | 1.5 周 | 无 |
| 3.4 | 自定义 WS 事件 | 1 周 | 无 |
| 3.5 | 房间-资源绑定 | 0.5 周 | 3.1 |
| 3.6 | SSO 社交登录 | 0.5 周 | Phase 1 |
| | **合计** | **~15 周** | |

---

## 兼容性保证

| 项目 | 策略 |
|------|------|
| v1 API | 完全保留，仅 `/api/v1/auth/register` 加 admin 权限限制 |
| 现有客户端 | 不受影响，可继续使用 v1 端点 |
| 数据库迁移 | v2 全部使用 `ALTER TABLE ADD COLUMN`，无破坏性变更 |
| 配置 | `mail.*` 为新增可选段（缺省则代码全零但可运行）；`batch_message.*` 为必需段（缺省或零值拒绝启动） |
| WebSocket 协议 | 新增 CustomEvent/CustomEventForward 变体，旧客户端忽略即可 |

---

## 文档索引

| 文档 | 说明 |
|------|------|
| [superpowers/CAPELLA_ROADMAP_ANALYSIS.md](../superpowers/CAPELLA_ROADMAP_ANALYSIS.md) | Perseus 路线图对比分析 |
| [superpowers/integration-design.md](../superpowers/integration-design.md) | Webhook/自定义事件/资源绑定详细设计 |

---

*文档版本: 1.0.0*
*最后更新: 2026-06-16*
