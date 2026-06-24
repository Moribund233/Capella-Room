use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::RedisManager;

const DLQ_STREAM: &str = "capella:stream:dead_letter";
const MAX_BATCH: usize = 100;

/// 死信消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterMessage {
    pub original_stream: String,
    pub original_id: String,
    pub payload: String,
    pub error_type: String,
    pub error_message: String,
    pub retry_count: u32,
    pub failed_at: DateTime<Utc>,
    pub source_node: String,
}

impl DeadLetterMessage {
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// 死信队列管理器
#[derive(Debug)]
pub struct DLQManager {
    manager: Arc<RedisManager>,
    max_retries: u32,
    node_id: String,
}

impl DLQManager {
    pub fn new(manager: Arc<RedisManager>, max_retries: u32) -> Self {
        let node_id = manager.node_id().to_string();
        info!(
            "DLQManager created: node_id={}, max_retries={}",
            node_id, max_retries
        );
        Self {
            manager,
            max_retries,
            node_id,
        }
    }

    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// 将消息路由到死信队列
    pub async fn route_to_dlq(
        &self,
        original_stream: &str,
        original_id: &str,
        payload: &str,
        error_type: &str,
        error_message: &str,
        retry_count: u32,
    ) -> anyhow::Result<String> {
        let msg = DeadLetterMessage {
            original_stream: original_stream.to_string(),
            original_id: original_id.to_string(),
            payload: payload.to_string(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            retry_count,
            failed_at: Utc::now(),
            source_node: self.node_id.clone(),
        };

        let json = msg.to_json()?;

        let mut conn = self
            .manager
            .get_connection()
            .await
            .ok_or_else(|| anyhow::anyhow!("Redis connection not available"))?;

        let id: String = conn.xadd(DLQ_STREAM, "*", &[("payload", &json)]).await?;

        // 限制 DLQ 大小
        let _: Result<i64, _> = redis::cmd("XTRIM")
            .arg(DLQ_STREAM)
            .arg("MAXLEN")
            .arg("~")
            .arg(100000)
            .query_async(&mut conn)
            .await;

        warn!(
            "Message routed to DLQ: stream={}, original_id={}, retry_count={}, error={}",
            original_stream, original_id, retry_count, error_message
        );

        Ok(id)
    }

    /// 分页查询死信消息
    pub async fn list_messages(
        &self,
        start: &str,
        end: &str,
        count: usize,
    ) -> anyhow::Result<Vec<(String, DeadLetterMessage)>> {
        let mut conn = self
            .manager
            .get_connection()
            .await
            .ok_or_else(|| anyhow::anyhow!("Redis connection not available"))?;

        let count = count.min(MAX_BATCH);
        let results: Vec<(String, std::collections::HashMap<String, redis::Value>)> = conn
            .xrange_count(DLQ_STREAM, start, end, count)
            .await?;

        let mut messages = Vec::new();
        for (id, map) in results {
            if let Some(redis::Value::Data(bytes)) = map.get("payload") {
                let payload_str = String::from_utf8_lossy(bytes).to_string();
                match DeadLetterMessage::from_json(&payload_str) {
                    Ok(msg) => messages.push((id, msg)),
                    Err(e) => warn!("Failed to parse DLQ message {}: {}", id, e),
                }
            }
        }

        Ok(messages)
    }

    /// 获取死信消息总数
    pub async fn message_count(&self) -> i64 {
        let mut conn = match self.manager.get_connection().await {
            Some(c) => c,
            None => return 0,
        };

        let len: Result<i64, _> = redis::cmd("XLEN")
            .arg(DLQ_STREAM)
            .query_async(&mut conn)
            .await;

        len.unwrap_or(0)
    }

    /// 重投单条消息到原 Stream
    pub async fn requeue(&self, dlq_id: &str) -> anyhow::Result<()> {
        let mut conn = self
            .manager
            .get_connection()
            .await
            .ok_or_else(|| anyhow::anyhow!("Redis connection not available"))?;

        let results: Vec<(String, std::collections::HashMap<String, redis::Value>)> = conn
            .xrange_count(DLQ_STREAM, dlq_id, dlq_id, 1)
            .await?;

        let (_, map) = results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("DLQ message not found: {}", dlq_id))?;

        let payload_str = match map.get("payload") {
            Some(redis::Value::Data(bytes)) => String::from_utf8_lossy(bytes).to_string(),
            _ => return Err(anyhow::anyhow!("Invalid DLQ message payload")),
        };

        let dlq_msg = DeadLetterMessage::from_json(&payload_str)?;

        // 写回原 Stream
        let _: String = conn
            .xadd(
                &dlq_msg.original_stream,
                "*",
                &[("payload", &dlq_msg.payload)],
            )
            .await?;

        // 从 DLQ 删除
        let _: Result<(), _> = conn.xdel::<&str, &str, ()>(DLQ_STREAM, &[dlq_id]).await;

        info!(
            "Requeued DLQ message to stream: {} (dlq_id: {})",
            dlq_msg.original_stream, dlq_id
        );

        Ok(())
    }

    /// 批量重投
    pub async fn batch_requeue(&self, ids: &[String]) -> anyhow::Result<(usize, Vec<String>)> {
        let mut success = 0;
        let mut failed = Vec::new();

        for id in ids {
            if let Err(e) = self.requeue(id).await {
                error!("Failed to requeue {}: {}", id, e);
                failed.push(id.clone());
            } else {
                success += 1;
            }
        }

        Ok((success, failed))
    }

    /// 删除单条死信
    pub async fn delete(&self, dlq_id: &str) -> anyhow::Result<()> {
        let mut conn = self
            .manager
            .get_connection()
            .await
            .ok_or_else(|| anyhow::anyhow!("Redis connection not available"))?;

        let _: () = conn.xdel(DLQ_STREAM, &[dlq_id]).await?;
        debug!("Deleted DLQ message: {}", dlq_id);
        Ok(())
    }

    /// 清理旧消息（超过 TTL 天数）
    pub async fn cleanup_expired(&self, ttl_days: i64) -> anyhow::Result<u64> {
        let messages = self.list_messages("-", "+", MAX_BATCH).await?;
        let cutoff = Utc::now() - chrono::Duration::days(ttl_days);
        let mut deleted = 0u64;

        for (id, msg) in &messages {
            if msg.failed_at < cutoff {
                if let Err(e) = self.delete(id).await {
                    warn!("Failed to cleanup DLQ message {}: {}", id, e);
                } else {
                    deleted += 1;
                }
            }
        }

        if deleted > 0 {
            info!("Cleaned up {} expired DLQ messages (TTL: {}d)", deleted, ttl_days);
        }

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_letter_message_serialization() {
        let msg = DeadLetterMessage {
            original_stream: "capella:stream:audit_logs".to_string(),
            original_id: "1234567890-0".to_string(),
            payload: r#"{"event":"user_login"}"#.to_string(),
            error_type: "retryable".to_string(),
            error_message: "Connection timeout".to_string(),
            retry_count: 3,
            failed_at: Utc::now(),
            source_node: "node-1".to_string(),
        };

        let json = msg.to_json().unwrap();
        let deserialized = DeadLetterMessage::from_json(&json).unwrap();

        assert_eq!(msg.original_stream, deserialized.original_stream);
        assert_eq!(msg.original_id, deserialized.original_id);
        assert_eq!(msg.payload, deserialized.payload);
        assert_eq!(msg.error_type, deserialized.error_type);
        assert_eq!(msg.error_message, deserialized.error_message);
        assert_eq!(msg.retry_count, deserialized.retry_count);
        assert_eq!(msg.source_node, deserialized.source_node);
    }

    #[test]
    fn test_dead_letter_message_invalid_json() {
        let result = DeadLetterMessage::from_json("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_dlq_manager_creation() {
        let config = crate::config::RedisConfig {
            url: "redis://localhost:6379".to_string(),
            enabled: true,
            pool_size: 10,
            timeout_secs: 5,
            channel_prefix: "test".to_string(),
            config_sync_enabled: true,
            consumer_batch_size: 100,
            consumer_poll_interval_ms: 1000,
            stream_max_len: 10000,
            dlq_enabled: true,
            dlq_max_retries: 3,
        };
        let manager = Arc::new(super::super::RedisManager {
            client: None,
            connection: tokio::sync::RwLock::new(None),
            config,
            node_id: "test-node".to_string(),
        });
        let dlq = DLQManager::new(manager, 3);
        assert_eq!(dlq.max_retries(), 3);
    }
}
