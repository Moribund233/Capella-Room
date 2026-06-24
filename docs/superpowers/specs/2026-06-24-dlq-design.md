# 死信队列（DLQ）设计

## 背景

当前 `src/redis/stream.rs` 中的 `claim_pending_messages` 仅对 Redis Stream Pending List 中的消息做 XCLAIM + 重新消费，没有重试次数上限，失败消息会在重试循环中无限停留，最终阻塞消费者。路线图 8.6.4 标注 DLQ 已完成但实际未实现。

## 设计原则

- 不引入新依赖，纯 Redis Stream 实现
- 区分可重试 vs 不可重试错误
- 管理 API 支持消息回放，而非仅被动记录

## 架构

```
                    ┌───────────────────┐
                    │  主 Consumer       │
                    │  StreamConsumer    │
                    └───────┬───────────┘
                            │ 处理失败
                            ▼
              ┌─────────────────────────┐
              │  错误分类器               │
              │  is_retryable(error)     │
              └───────┬─────────┬───────┘
                      │         │
             不可重试   │         │ 可重试
                      │         ▼
                      │   ┌─────────────────┐
                      │   │  Pending List    │
                      │   │  XCLAIM 重试     │
                      │   │  delivery ≥ N   │
                      └───┴─────────┬───────┘
                                    │ 超过上限
                                    ▼
                     ┌─────────────────────────┐
                     │  DLQ Stream              │
                     │  capella:stream:dead_letter│
                     │  (XTRIM 7天)              │
                     └───────────┬─────────────┘
                                 │
                     ┌───────────┴───────────┐
                     │  管理 API              │
                     │  GET/DELETE /requeue  │
                     └───────────────────────┘
```

## 重试策略

| 错误类型 | 行为 | 示例 |
|---------|------|------|
| `non_retryable` | 直接入 DLQ | 反序列化失败、消息格式错误 |
| `retryable` | 重试 up to N 次 | 网络超时、Redis 断连、下游 503 |

重试来源：`XPENDING` 返回的 delivery count（每个消息的第 4 个字段），无需额外存储。

默认 N=3，配置项 `max_retry_count: u32`。

## DLQ 消息结构

```rust
pub struct DeadLetterMessage {
    pub original_stream: String,     // 来源 Stream 名称
    pub original_id: String,         // 原始消息 ID
    pub payload: String,             // 原始消息 JSON
    pub error_type: String,          // "retryable" | "non_retryable"
    pub error_message: String,       // 错误详情
    pub retry_count: u32,            // 失败时的已重试次数
    pub failed_at: DateTime<Utc>,    // 首次入 DLQ 时间
    pub source_node: String,         // 处理节点 ID
}
```

存储至 `capella:stream:dead_letter`，序列化为 JSON。

## 文件变更

### 新增文件

| 文件 | 内容 |
|------|------|
| `src/redis/dlq.rs` | DeadLetterMessage、DLQ producer/API 逻辑 |
| `src/handlers/dlq_admin.rs` | DLQ 管理 API 处理器 |

### 改动文件

| 文件 | 改动 |
|------|------|
| `src/redis/mod.rs` | 导出 dlq 模块 + DeadLetterMessage |
| `src/redis/stream.rs` | `consume_batch` 增加错误分类与 DLQ 路由；`claim_pending_messages` 增加 delivery count 检查 |
| `src/config/mod.rs` | `RedisConfig` 增加 `dlq_enabled: bool`, `dlq_max_retries: u32` 字段（`#[serde(default)]`） |
| `config.toml` | 在 `[redis]` 节增加 `dlq_enabled = true`、`dlq_max_retries = 3` 默认值（遵循系统标准，不在 Rust 代码中编码默认值） |
| `src/routes/mod.rs` | 挂载 `/api/v1/admin/dlq/*` 路由 |
| `src/state/mod.rs` | 初始化 DLQ 管理器 |
| `src/main.rs` | 创建 DLQ 管理器传给 AppState |

## 管理 API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/v1/admin/dlq/messages` | GET | 分页查询死信（query: stream, page, size） |
| `/api/v1/admin/dlq/{id}/requeue` | POST | 重投单条到原 Stream |
| `/api/v1/admin/dlq/batch-requeue` | POST | 批量重投（body: ids[]） |
| `/api/v1/admin/dlq/{id}` | DELETE | 删除单条 |
| `/api/v1/admin/dlq/stats` | GET | 概览（总量、按 stream 分组） |

## TTL 与清理

- 每次 DLQ 写入后，附带 `XTRIM ... MAXLEN ~ 100000` 控制总量
- 后台定时任务每 1 小时清理超过 7 天的消息（比对 `failed_at` 字段）
- 手动 DELETE API 即时清理

## 监控指标

- `dlq.message_count` — 当前死信总量
- `dlq.retry_exhausted_rate` — 重试耗尽率（次/分钟）
- `dlq.requeue_count` — 重投递次数

通过日志输出，后续可接入 Prometheus。

## 测试

- `test_dlq_message_serialization` — 序列化/反序列化
- `test_retryable_error_routes_to_dlq` — 可重试错误超限后入 DLQ
- `test_non_retryable_error_direct_dlq` — 不可重试错误直接入 DLQ
- `test_dlq_requeue` — 重投回主 Stream
- `test_dlq_auto_trim` — TTL 清理
