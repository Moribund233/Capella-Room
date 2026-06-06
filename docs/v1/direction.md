### 一、 核心原则：安全与体验的平衡

在聊天室场景中，**服务端必须持有解密能力**才能实现“全文搜索”、“消息审核”和“离线推送”。因此，不要追求极端的端到端加密（E2EE），而应采用 **传输层 TLS + 存储层 AES-256-GCM + 应用层字段级加密** 的混合模式。

---

### 二、 加密消息系统重构方案

#### 1. 数据库模型变更 (PostgreSQL)
将 `messages` 表从明文改为密文存储，同时保留必要的元数据用于索引。

```sql
ALTER TABLE messages 
ADD COLUMN content_ciphertext BYTEA NOT NULL,      -- AES-256-GCM 密文 (含 nonce)
ADD COLUMN content_nonce BYTEA NOT NULL,           -- 12字节随机 Nonce
ADD COLUMN encryption_key_id VARCHAR(36) NOT NULL, -- 关联密钥版本，支持轮换
ADD COLUMN is_encrypted BOOLEAN DEFAULT TRUE;

-- 废弃或清空原 content 字段（迁移完成后删除）
-- ALTER TABLE messages DROP COLUMN content;
```

> ⚠️ **关键注意**: AES-GCM 模式下，**Nonce 绝不能重复**。建议将 Nonce 与密文拼接存储，或使用 UUIDv7 作为 Nonce 的一部分以保证唯一性。

#### 2. 密钥管理体系 (KMS)
**绝对不要**把密钥硬编码或仅放在环境变量里。

| 方案 | 适用场景 | Axum 集成方式 |
| :--- | :--- | :--- |
| **Envelope Encryption** (推荐) | 生产环境 | 主密钥存于 AWS KMS/Vault，数据密钥(DEK)加密后存DB |
| **pgcrypto + 应用层密钥** | 中小规模 | 密钥通过 Vault/Env 注入，Rust 侧用 `aes-gcm` crate 加解密 |
| **Per-Room Key** | 高安全需求 | 每个房间独立 DEK，成员加入时通过 RSA/OAEP 分发 |

**推荐实现路径：**
-   使用 `ring` 或 `aes-gcm` crate 进行加解密
-   密钥版本化：`encryption_key_id` 字段支持无缝轮换，旧消息用旧密钥解密，新消息用新密钥
-   启动时从安全存储加载密钥到内存（`Arc<RwLock<HashMap<String, Key>>>`），避免每次查库

#### 3. 搜索能力的保留策略
加密后 `tsvector` 全文索引失效。三种解决方案：

| 方案 | 安全性 | 搜索体验 | 复杂度 |
| :--- | :--- | :--- | :--- |
| **客户端搜索** | ✅ 最高 | ❌ 差（需拉全量） | 低 |
| **搜索索引服务** (推荐) | ⚠️ 中 | ✅ 好 | 中 |
| **盲索引/Tokenization** | ⚠️ 中 | ⚠️ 仅精确匹配 | 高 |

**推荐方案：异步搜索索引管线**
1.  消息写入时，先加密存 PG
2.  通过 Redis Stream 异步触发索引构建 Worker
3.  Worker 解密消息 → 脱敏/分词 → 写入 **Meilisearch / Typesense**（独立搜索引擎）
4.  搜索 API 走搜索引擎，结果只返回 message_id，再回 PG 解密展示
5.  搜索引擎可配置 TTL 自动过期，降低长期暴露风险

#### 4. Axum 中间件/Service 层封装

```rust
// 伪代码：消息加解密 Service
pub struct MessageCryptoService {
    keys: Arc<KeyStore>,
}

impl MessageCryptoService {
    pub async fn encrypt(&self, plaintext: &str, room_id: Uuid) -> EncryptedPayload {
        let dek = self.keys.get_current_dek(room_id).await;
        let nonce = generate_nonce(); // 12 bytes
        let ciphertext = aes_gcm_encrypt(&dek, &nonce, plaintext.as_bytes());
        EncryptedPayload { ciphertext, nonce, key_id: dek.id }
    }

    pub async fn decrypt(&self, payload: &EncryptedPayload) -> Result<String> {
        let dek = self.keys.get_dek_by_id(&payload.key_id).await?;
        let plaintext = aes_gcm_decrypt(&dek, &payload.nonce, &payload.ciphertext)?;
        Ok(String::from_utf8(plaintext)?)
    }
}
```

在现有的消息 CRUD Handler 中注入此 Service，对上层业务透明。

---

### 三、 智能消息审核系统

利用你已有的 **Redis Stream 异步写入** 基础设施，构建非阻塞审核管线：

#### 1. 审核架构

```
用户发消息 → WebSocket/HTTP → 加密存PG → 发布到 Redis Stream
                                              ↓
                                    Audit Consumer Group
                                      ↙        ↓         ↘
                               敏感词过滤   AI语义分析   图片/文件扫描
                                      ↘        ↓         ↙
                                   审核结果聚合 → 写回 audit_results 表
                                              ↓
                                     决策引擎 (通过/拦截/人工复审)
                                              ↓
                                    WebSocket 通知发送者/管理员
```

#### 2. 分层审核策略

| 层级 | 技术 | 延迟 | 作用 |
| :--- | :--- | :--- | :--- |
| L1 实时 | DFA/AC自动机 敏感词 | <1ms | 拦截明显违规，消息发送前同步检查 |
| L2 准实时 | 本地 ML 模型 (ONNX) | 10-50ms | 语义理解、变体识别、情绪分析 |
| L3 异步 | 外部 AI API / 大模型 | 1-5s | 复杂上下文判断、多模态审核 |
| L4 人工 | 管理后台 | 分钟级 | L2/L3 置信度低的消息进入复审队列 |

#### 3. 与现有系统的集成点

-   **L1 同步拦截**：在 WebSocket `ChatMessage` handler 中，发送前做敏感词快速检查，命中则直接拒绝并返回错误
-   **L2/L3 异步审核**：复用阶段 8.6 的 Redis Stream Consumer Group，新增 `audit` consumer
-   **审核结果回调**：通过现有的 WebSocket `SystemMessage` 或 `Notification` 通道通知用户“消息已被撤回”或通知管理员
-   **审计日志**：所有审核决策写入 `audit_logs` 表（阶段 8.4 已有），支持追溯

#### 4. 用户体验优化

-   **乐观发送 + 异步撤回**：消息先显示在聊天框，审核不通过时再灰显/替换为“消息已因违规被移除”，避免发送卡顿
-   **申诉机制**：被审核拦截的消息允许用户一键申诉，进入人工复审队列
-   **分级提示**：区分“系统自动拦截”和“管理员手动删除”，给出不同提示文案
-   **预览保护**：审核中的图片/文件在通知和消息列表中显示占位符，审核通过后才展示真实内容

---

### 四、 重构路线图建议

| 阶段 | 内容 | 预估工时 | 依赖 |
| :--- | :--- | :--- | :--- |
| **P0** | 密钥管理 + DB Schema 迁移 + 加解密 Service | 3-5天 | 无 |
| **P1** | 消息写入/读取链路改造 + 历史数据迁移脚本 | 3-4天 | P0 |
| **P2** | L1 敏感词同步拦截 + 搜索索引异步管线 | 3-4天 | P1 |
| **P3** | L2/L3 异步审核 Consumer + 审核结果回调 | 4-5天 | P2 |
| **P4** | 管理后台审核面板 + 申诉流程 + 体验优化 | 3-4天 | P3 |

### 五、 安全注意事项

1.  **内存安全**：解密后的明文用完立即 zeroize，使用 `zeroize` crate
2.  **日志脱敏**：确保所有日志（包括 system-logs WebSocket 推送）中不包含消息明文
3.  **密钥轮换**：实现平滑轮换机制，新旧密钥共存期至少覆盖消息最长生命周期
4.  **备份加密**：PG 备份本身也应加密，防止备份泄露导致密文被批量破解
5.  **合规**：如果面向国内用户，智能审核是合规刚需；如果面向海外，需注意 GDPR 对用户数据删除权的要求（加密反而有助于实现 crypto-shredding）

Telegram 的核心并非全盘 E2EE（端到端加密），而是采用了 **“云聊天（Cloud Chats）+ 秘密聊天（Secret Chats）”的双模架构**。对于 Capella Room v1 的重构，我强烈建议采用这种**分层安全模型**，而不是盲目追求全量 E2EE。

以下是基于 Telegram 模式，结合你现有 Axum + PostgreSQL + Redis Stream 架构的具体开发方向：

### 一、 核心架构：双模消息系统

不要把所有消息都加密或都不加密，而是根据场景拆分：

| 模式 | 对应 Telegram | 存储方式 | 搜索/审核 | 适用场景 | Capella 实现策略 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **云消息** | Cloud Chat | 服务端 AES-256-GCM 字段级加密 | ✅ 支持 | 群聊、普通私聊、文件传输 | **默认模式**。复用现有 Redis Stream 异步写入，增加加解密 Service 层 |
| **秘密消息** | Secret Chat | 纯客户端 E2EE (X25519 + AES-GCM) | ❌ 不支持 | 高敏感 1:1 私聊 | **可选模式**。服务端仅转发密文 Blob，不存明文，不参与审核 |

> 💡 **关键决策**：Capella Room 作为聊天室应用，**90% 的场景应使用“云消息”模式**。这保证了你可以继续利用阶段 5 的全文搜索、阶段 8.4 的审计系统和即将开发的智能审核。E2EE 仅作为高级隐私功能提供给 1:1 私聊。

### 二、 云消息模式重构方向（重点）

这是你当前 v1 → v2 升级的主战场，参考 Telegram 的服务端加密实践：

#### 1. 密钥分层体系（Envelope Encryption）
Telegram 服务端并不用一把钥匙开所有锁，而是严格分层：

```
Master Key (KMS/Vault)
    └── Room Key (DEK, 每个房间独立)
            └── Message Key (从 DEK + msg_id 派生，每条消息唯一)
```

-   **Room Key (DEK)**：每个聊天室生成独立的 AES-256 密钥，存储在 `room_keys` 表中（被 Master Key 加密）
-   **Message Key 派生**：`HMAC-SHA256(room_dek, message_id || nonce)` → 取前 32 字节作为该条消息的实际加密密钥
-   **优势**：即使某条消息的 Nonce 出问题，也不会波及其他消息；删除房间时只需销毁 DEK 即可实现 **Crypto Shredding**（密码学擦除），完美契合 GDPR

#### 2. 数据库 Schema 演进

```sql
-- 新增房间密钥表
CREATE TABLE room_encryption_keys (
    room_id UUID PRIMARY KEY REFERENCES rooms(id),
    encrypted_dek BYTEA NOT NULL,        -- Master Key 加密后的 DEK
    dek_version INT NOT NULL DEFAULT 1,  -- 支持密钥轮换
    created_at TIMESTAMPTZ NOT NULL,
    rotated_at TIMESTAMPTZ               -- 上次轮换时间
);

-- messages 表改造
ALTER TABLE messages 
ADD COLUMN ciphertext BYTEA,             -- 加密内容
ADD COLUMN nonce BYTEA,                  -- 12 bytes
ADD COLUMN key_version INT DEFAULT 1,    -- 关联 DEK 版本
ADD COLUMN auth_tag BYTEA;               -- GCM 认证标签（也可拼接到 ciphertext）
```

#### 3. 与现有 Redis Stream 管线的集成

你已在阶段 8.6 实现了 Redis Stream 异步写入，加密改造可以无缝嵌入：

```
WebSocket ChatMessage 
    → Handler 同步 L1 敏感词检查（明文阶段）
    → 调用 MessageCryptoService.encrypt()
    → 写入 PG（密文）
    → XADD 到 Redis Stream（密文 + 元数据）
        ├── Audit Consumer: 解密 → AI 审核 → 写 audit_results
        ├── Search Index Consumer: 解密 → 分词 → 写 Meilisearch
        └── Notification Consumer: 解密 → 提取 @提及 → 推送通知
```

> ⚠️ **安全要点**：Redis Stream 中传输的也是密文。Consumer 在内存中解密后立即 zeroize，审核/索引服务与主应用共享同一套 KMS 访问权限但独立部署。

### 三、 秘密聊天模式（E2EE）开发方向

作为差异化功能，后期可按以下路径实现：

#### 1. 协议选型
直接采用 **Signal Protocol** 或简化版 **X25519 + AES-256-GCM**：
-   客户端生成临时密钥对，通过服务端交换公钥
-   服务端仅作为不可信中继，存储 `{from, to, encrypted_blob, timestamp}`
-   消息不进入 Redis Stream、不触发审核、不建搜索索引

#### 2. 与现有系统的隔离
-   WebSocket 新增 `SecretChatMessage` 消息类型，与 `ChatMessage` 完全分离
-   服务端不做任何解析，直接按 user_id 路由转发
-   离线消息存入独立的 `secret_messages` 表（纯密文，无外键关联内容）

#### 3. 渐进式落地
**v2.0 先不做 E2EE**，优先把云消息加密 + 智能审核做扎实。E2EE 可作为 v2.5 或 v3.0 的功能，因为它的客户端复杂度远高于服务端。

### 四、 智能审核的 Telegram 式体验优化

Telegram 的审核对用户几乎无感，这得益于其**异步 + 乐观更新**机制：

| 体验点 | 传统做法 | Telegram/Capella 推荐做法 |
| :--- | :--- | :--- |
| 发送延迟 | 等审核通过才显示 | **立即显示**，审核失败后替换为"已撤回" |
| 误杀申诉 | 联系客服邮箱 | 消息旁显示 ⚠️ 图标，点击一键申诉 |
| 审核粒度 | 全量拦截 | **分级处置**：L1 同步拦截明显违规；L2/L3 异步标记可疑但不阻断 |
| 管理员可见性 | 看到原始消息 | 审核后台显示脱敏摘要 + 风险评分，点击查看原文需二次确认 |

结合你已有的 **WebSocket SystemMessage** 和 **Notification 系统**，审核结果可以通过现有的 `SystemMessage` 通道实时推送给发送者和管理员，无需新建通信管道。

### 五、 推荐开发路线图

| 优先级 | 里程碑 | 内容 | 预估周期 |
| :--- | :--- | :--- | :--- |
| **P0** | 云消息加密基础设施 | Envelope Encryption + DB Schema + Crypto Service + 历史数据迁移 | 2 周 |
| **P1** | 审核管线升级 | L1 同步拦截 + Redis Stream 异步审核 Consumer + 乐观发送体验 | 1.5 周 |
| **P2** | 搜索能力恢复 | Meilisearch/Typesense 集成 + 异步索引构建 + 搜索结果解密回显 | 1.5 周 |
| **P3** | 管理后台适配 | 审核面板 + 密钥轮换 UI + Crypto Shredding 功能 | 1 周 |
| **P4** | E2EE 秘密聊天（可选） | X25519 密钥交换 + SecretChatMessage 协议 + 客户端 SDK | 3-4 周 |


以下是 **Envelope Encryption + DB Schema + Crypto Service** 三大核心模块的详细落地设计：

### 一、 Envelope Encryption（信封加密）架构设计

不要使用单一密钥加密所有消息。采用三层密钥体系，兼顾安全性与性能：

```
┌─────────────────────────────────────────────┐
│           Master Key (KEK)                  │ ← 存于 Vault / AWS KMS / 环境变量(开发)
│         (AES-256-GCM, 永不落盘DB)            │
└──────────────────┬──────────────────────────┘
                   │ 加密/解密
                   ▼
┌─────────────────────────────────────────────┐
│      Room Data Encryption Key (DEK)         │ ← 每个房间独立，加密后存 room_keys 表
│      (AES-256-GCM, 支持版本轮换)             │
└──────────────────┬──────────────────────────┘
                   │ 派生 (HKDF-SHA256)
                   ▼
┌─────────────────────────────────────────────┐
│          Per-Message Key                    │ ← 每条消息唯一，内存中生成，用完即焚
│   HKDF(DEK, msg_id || nonce, info="msg")    │
└─────────────────────────────────────────────┘
```

#### 为什么需要 Per-Message Key？
-   **Nonce 重用防护**：即使 DEK 相同，每条消息的派生密钥不同，彻底杜绝 AES-GCM Nonce 重复灾难
-   **并行加解密**：无需为每条消息单独调用 KMS，DEK 缓存到内存后可批量派生
-   **Crypto Shredding**：删除房间时只需销毁 DEK，所有历史消息瞬间不可恢复（GDPR 合规）

---

### 二、 DB Schema 变更

基于你现有的 `messages` 表和阶段 3 的房间系统，新增以下结构：

#### 1. 房间密钥表（新增）

```sql
CREATE TABLE room_encryption_keys (
    room_id     UUID PRIMARY KEY REFERENCES rooms(id) ON DELETE CASCADE,
    dek_ciphertext BYTEA NOT NULL,          -- KEK 加密后的 DEK
    dek_version  INT NOT NULL DEFAULT 1,    -- 密钥版本号
    algorithm    VARCHAR(20) NOT NULL DEFAULT 'aes-256-gcm',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at   TIMESTAMPTZ,               -- 上次轮换时间
    
    -- 约束：确保同一房间不会插入多条记录（UPSERT 语义）
    CONSTRAINT chk_dek_version_positive CHECK (dek_version > 0)
);

-- 索引：按版本查找（轮换时需要查旧版本）
CREATE INDEX idx_room_keys_version ON room_encryption_keys(room_id, dek_version DESC);
```

#### 2. Messages 表改造

```sql
ALTER TABLE messages 
    ADD COLUMN ciphertext    BYTEA,          -- AES-GCM 密文 (含 auth_tag)
    ADD COLUMN nonce         BYTEA,          -- 12 bytes random nonce
    ADD COLUMN key_version   INT DEFAULT 1,  -- 关联 room_encryption_keys.dek_version
    ADD COLUMN is_encrypted  BOOLEAN DEFAULT FALSE;  -- 迁移期间兼容标记

-- 迁移完成后执行：
-- ALTER TABLE messages ALTER COLUMN ciphertext SET NOT NULL;
-- ALTER TABLE messages ALTER COLUMN nonce SET NOT NULL;
-- ALTER TABLE messages DROP COLUMN content;  -- 删除明文列
-- DROP INDEX IF EXISTS idx_messages_content_search;  -- 删除旧 tsvector 索引
```

> ⚠️ **迁移策略**：设置 `is_encrypted` 过渡字段，编写后台迁移 Worker 逐批加密历史消息。读取时根据该字段决定是否需要解密，实现零停机迁移。

---

### 三、 Crypto Service Rust 实现

作为 Axum 的 State 注入，封装所有加解密逻辑，对 Handler 层透明。

#### 1. 依赖选型

```toml
[dependencies]
aes-gcm = "0.10"
hkdf = "0.12"
sha2 = "0.10"
zeroize = { version = "1", features = ["derive"] }
rand = "0.8"
base64 = "0.22"
```

#### 2. 核心 Service 定义

```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroizing;
use std::sync::Arc;
use dashmap::DashMap;

/// 内存中的 DEK 缓存（避免每次查库）
type DekCache = DashMap<(Uuid, i32), Zeroizing<[u8; 32]>>;

pub struct MessageCryptoService {
    kek: Zeroizing<[u8; 32]>,       // Master Key，启动时加载
    dek_cache: Arc<DekCache>,
    db: PgPool,
}

impl MessageCryptoService {
    /// 加密消息（写入路径）
    pub async fn encrypt(
        &self,
        room_id: Uuid,
        message_id: Uuid,
        plaintext: &[u8],
    ) -> Result<EncryptedPayload> {
        // 1. 获取当前 DEK（带缓存）
        let (dek, version) = self.get_or_create_dek(room_id).await?;
        
        // 2. 生成随机 Nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        
        // 3. HKDF 派生 Per-Message Key
        let msg_key = self.derive_message_key(&dek, message_id, &nonce_bytes);
        
        // 4. AES-256-GCM 加密
        let cipher = Aes256Gcm::new_from_slice(&msg_key)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        Ok(EncryptedPayload {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            key_version: version,
        })
    }

    /// 解密消息（读取路径）
    pub async fn decrypt(
        &self,
        room_id: Uuid,
        message_id: Uuid,
        payload: &EncryptedPayload,
    ) -> Result<Vec<u8>> {
        // 1. 根据 key_version 获取对应 DEK
        let dek = self.get_dek_by_version(room_id, payload.key_version).await?;
        
        // 2. 派生相同的 Per-Message Key
        let nonce_bytes: [u8; 12] = payload.nonce.as_slice().try_into()
            .map_err(|_| CryptoError::InvalidNonce)?;
        let msg_key = self.derive_message_key(&dek, message_id, &nonce_bytes);
        
        // 3. 解密
        let cipher = Aes256Gcm::new_from_slice(&msg_key)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        cipher.decrypt(nonce, payload.ciphertext.as_ref())
            .map_err(|_| CryptoError::DecryptionFailed)
    }

    /// HKDF 密钥派生（纯计算，无 IO）
    fn derive_message_key(
        &self,
        dek: &[u8; 32],
        message_id: Uuid,
        nonce: &[u8; 12],
    ) -> Zeroizing<[u8; 32]> {
        let mut ikm = Vec::with_capacity(48);
        ikm.extend_from_slice(&message_id.as_bytes());
        ikm.extend_from_slice(nonce);
        
        let hk = Hkdf::<Sha256>::new(Some(b"capella-msg-v1"), dek);
        let mut okm = Zeroizing::new([0u8; 32]);
        hk.expand(&ikm, &mut *okm).expect("HKDF expand failed");
        okm
    }

    /// 密钥轮换：生成新 DEK，旧 DEK 保留用于解密历史消息
    pub async fn rotate_room_key(&self, room_id: Uuid) -> Result<()> {
        // 1. 用 KEK 加密新 DEK
        // 2. INSERT INTO room_encryption_keys ... (version = old + 1)
        // 3. 清除缓存中的旧条目，强制下次重新加载
        self.dek_cache.remove(&(room_id, /* old_version */));
        Ok(())
    }
}
```

#### 3. 与现有 Redis Stream 管线的集成点

你已在阶段 8.6 实现了 Redis Stream 异步写入。加密改造只需在 **写入前** 和 **消费时** 各加一步：

```
[WebSocket Handler]
    ChatMessage 到达
    → L1 敏感词同步检查（此时仍是明文 ✅）
    → crypto_service.encrypt(room_id, msg_id, content)  ← 新增
    → INSERT INTO messages (ciphertext, nonce, key_version)
    → XADD capella:stream:messages {encrypted_payload}  ← Stream 中也是密文

[Audit Consumer / Search Index Consumer]
    XREADGROUP 消费消息
    → crypto_service.decrypt(room_id, msg_id, payload)  ← 新增
    → 审核 / 分词索引
    → zeroize 明文缓冲区  ← 关键安全措施
```

> 💡 **关键点**：L1 敏感词检查必须在加密**之前**执行。这是 Telegram 模式的核心取舍——牺牲少量隐私换取实时拦截能力。L2/L3 异步审核则在 Consumer 中解密后进行。

---

### 四、 安全加固清单

| 项目 | 措施 | 对应 Crate/工具 |
| :--- | :--- | :--- |
| 内存清零 | 所有密钥、明文使用后立即 zeroize | `zeroize` + `Zeroizing<T>` |
| 日志脱敏 | 禁止记录 plaintext、dek、kek 的任何片段 | tracing filter + 自定义 formatter |
| 密钥加载 | KEK 仅通过环境变量/Vault 注入，永不写入配置文件或 DB | `dotenvy` + Vault Agent |
| 错误处理 | 解密失败不暴露具体原因（防 oracle 攻击），统一返回 `DecryptionFailed` | 自定义 Error 类型 |
| 备份安全 | PG 备份启用透明加密（TDE），防止备份文件泄露 | pg_dump + gpg / 云厂商 TDE |
| 测试验证 | 单元测试覆盖：加密→解密往返、密钥轮换后旧消息仍可解密、篡改密文解密失败 | `proptest` 模糊测试 |

### 五、 与你现有系统的兼容性保障

| 现有功能 | 影响 | 适配方案 |
| :--- | :--- | :--- |
| 阶段 5 全文搜索 | tsvector 索引失效 | P2 阶段接入 Meilisearch，Consumer 解密后建索引 |
| 阶段 8.4 审计日志 | 审计内容需可读 | Audit Consumer 解密后写入审计表（审计表本身可单独加密） |
| 阶段 8.6 Redis Stream | Stream 消息格式变更 | 新增 `EncryptedStreamMessage` 类型，Consumer Group 兼容新旧格式 |
| 阶段 9 私聊 | 私聊也走云消息加密 | 复用同一套 Crypto Service，私聊房间也有独立 DEK |
| WebSocket NewMessage 广播 | 广播内容需解密 | Handler 加密存库后，用**原始明文**广播给同房间用户（避免客户端解密负担） |

> ⚠️ **最后一点至关重要**：WebSocket 广播给房间内其他用户的消息**不需要加密传输**（已有 TLS），也不需要客户端解密。加密仅针对**持久化存储**。这保持了客户端的轻量性，也是 Telegram Cloud Chat 的核心设计哲学。