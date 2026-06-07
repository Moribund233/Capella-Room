//! 批量消息写入服务
//!
//! 将消息写入操作从实时路径解耦，使用内存队列批量聚合后写入数据库。
//! 这样可以在保证消息实时广播的同时，提高数据库写入吞吐量。

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::db::Database;
use crate::models::message::MessageType;

/// 批量消息服务错误
#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("消息队列已满，消息被丢弃")]
    QueueFull,
}

/// 待写入的消息项
#[derive(Debug, Clone)]
pub struct PendingMessage {
    /// 消息ID（与广播给客户端的ID一致，由调用方生成）
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<Uuid>,
    /// 消息发送时间（由调用方在收到消息时记录，用于保证排序正确性）
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 批量消息写入器配置
#[derive(Debug, Clone)]
pub struct BatchMessageConfig {
    /// 批量大小，达到此数量立即写入
    pub batch_size: usize,
    /// 刷新间隔（毫秒），达到此时间立即写入
    pub flush_interval_ms: u64,
    /// 队列最大长度，超过此长度将丢弃最旧的消息
    pub max_queue_size: usize,
}

impl Default for BatchMessageConfig {
    fn default() -> Self {
        Self {
            batch_size: 500,        // 增大批量大小，提高写入效率
            flush_interval_ms: 50,  // 缩短刷新间隔，更快写入
            max_queue_size: 100000, // 增大队列，避免高并发时丢消息
        }
    }
}

/// 批量消息写入服务
pub struct BatchMessageService {
    db: Database,
    config: BatchMessageConfig,
    /// 消息队列
    queue: Arc<Mutex<VecDeque<PendingMessage>>>,
    /// 用于通知刷新任务新消息到达
    notify_tx: mpsc::Sender<()>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
}

impl BatchMessageService {
    /// 创建新的批量消息服务
    pub fn new(db: Database, config: BatchMessageConfig) -> (Self, mpsc::Receiver<()>) {
        let (notify_tx, notify_rx) = mpsc::channel(1);
        let queue = Arc::new(Mutex::new(VecDeque::with_capacity(config.max_queue_size)));
        let running = Arc::new(RwLock::new(true));

        let service = Self {
            db,
            config,
            queue,
            notify_tx,
            running,
        };

        (service, notify_rx)
    }

    /// 启动批量写入任务
    pub fn start(&self, mut notify_rx: mpsc::Receiver<()>) {
        let queue = self.queue.clone();
        let db = self.db.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            info!(
                "Batch message writer started (batch_size={}, flush_interval={}ms)",
                config.batch_size, config.flush_interval_ms
            );

            let mut flush_timer = interval(Duration::from_millis(config.flush_interval_ms));
            flush_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    // 定时刷新
                    _ = flush_timer.tick() => {
                        let should_flush = {
                            let q = queue.lock().await;
                            !q.is_empty()
                        };
                        if should_flush {
                            Self::flush_batch(&queue, &db, config.batch_size).await;
                        }
                    }
                    // 收到新消息通知
                    _ = notify_rx.recv() => {
                        let should_flush = {
                            let q = queue.lock().await;
                            q.len() >= config.batch_size
                        };
                        if should_flush {
                            Self::flush_batch(&queue, &db, config.batch_size).await;
                        }
                    }
                }

                // 检查是否停止
                if !*running.read().await {
                    // 停止前刷新剩余消息
                    Self::flush_batch(&queue, &db, usize::MAX).await;
                    break;
                }
            }

            info!("Batch message writer stopped");
        });
    }

    /// 将消息加入写入队列
    /// 立即返回，不等待数据库写入完成
    ///
    /// # 错误
    /// - `BatchError::QueueFull`: 队列已满，已丢弃最旧消息以腾出空间，新消息仍成功入队
    pub async fn enqueue(&self, message: PendingMessage) -> std::result::Result<(), BatchError> {
        let mut queue = self.queue.lock().await;

        if queue.len() >= self.config.max_queue_size {
            warn!(
                "Message queue full ({}), dropping oldest message",
                self.config.max_queue_size
            );
            queue.pop_front();
            queue.push_back(message);
            let queue_len = queue.len();
            drop(queue);

            // 通知刷新任务
            let _ = self.notify_tx.send(()).await;
            debug!("Message enqueued under queue pressure, queue size: {}", queue_len);
            return Err(BatchError::QueueFull);
        }

        queue.push_back(message);
        let queue_len = queue.len();
        drop(queue);

        // 通知刷新任务
        let _ = self.notify_tx.send(()).await;

        debug!(
            "Message enqueued for batch write, queue size: {}",
            queue_len
        );

        Ok(())
    }

    /// 执行批量写入
    async fn flush_batch(
        queue: &Arc<Mutex<VecDeque<PendingMessage>>>,
        db: &Database,
        batch_size: usize,
    ) {
        // 取出待写入的消息
        let messages: Vec<PendingMessage> = {
            let mut q = queue.lock().await;
            let take_count = q.len().min(batch_size);
            q.drain(0..take_count).collect()
        };

        if messages.is_empty() {
            return;
        }

        let count = messages.len();
        debug!("Flushing {} messages to database", count);

        // 执行批量插入
        match Self::batch_insert(db, &messages).await {
            Ok(inserted) => {
                debug!("Successfully inserted {} messages", inserted);
            }
            Err(e) => {
                error!("Failed to batch insert messages: {}", e);
                // 写入失败时将消息重新入队（插回队首），后续 flush 会重试
                let mut q = queue.lock().await;
                for msg in messages.into_iter().rev() {
                    q.push_front(msg);
                }
                debug!("Re-enqueued {} messages for retry", q.len());
            }
        }
    }

    /// 批量插入消息到数据库
    /// 使用多行 VALUES 语法，兼容 PostgreSQL 枚举类型
    async fn batch_insert(db: &Database, messages: &[PendingMessage]) -> anyhow::Result<usize> {
        if messages.is_empty() {
            return Ok(0);
        }

        let count = messages.len();

        // 构建多行 VALUES 子句
        // 例如: ($1, $2, $3, $4, $5, $6, $7), ($8, $9, $10, $11, $12, $13, $14), ...
        let mut values_clauses = Vec::new();
        let mut param_index = 1;

        for _ in messages {
            let clause = format!(
                "(${}, ${}, ${}, ${}, ${}::message_type, ${}, ${})",
                param_index,
                param_index + 1,
                param_index + 2,
                param_index + 3,
                param_index + 4,
                param_index + 5,
                param_index + 6,
            );
            values_clauses.push(clause);
            param_index += 7;
        }

        let sql = format!(
            "INSERT INTO messages (id, room_id, sender_id, content, message_type, reply_to, created_at) VALUES {}",
            values_clauses.join(", ")
        );

        // 构建查询并绑定参数
        let mut query = sqlx::query(&sql);

        for msg in messages {
            let message_type_str = match msg.message_type {
                crate::models::message::MessageType::Text => "text",
                crate::models::message::MessageType::Image => "image",
                crate::models::message::MessageType::File => "file",
                crate::models::message::MessageType::System => "system",
            };

            query = query
                .bind(msg.id)
                .bind(msg.room_id)
                .bind(msg.sender_id)
                .bind(&msg.content)
                .bind(message_type_str)
                .bind(msg.reply_to)
                .bind(msg.created_at);
        }

        query.execute(db.pool()).await?;

        Ok(count)
    }

    /// 停止批量写入服务
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Batch message writer stopping...");
    }

    /// 获取当前队列长度
    pub async fn queue_len(&self) -> usize {
        self.queue.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_default() {
        let config = BatchMessageConfig::default();
        assert_eq!(config.batch_size, 500);
        assert_eq!(config.flush_interval_ms, 50);
        assert_eq!(config.max_queue_size, 100000);
    }
}
