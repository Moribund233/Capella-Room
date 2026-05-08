use async_trait::async_trait;
use tracing::{error, info};

use crate::db::Database;
use crate::redis::{AuditLogStreamMessage, StreamConsumerHandler};

/// 审计日志消费者处理器
///
/// 将 Redis Stream 中的审计日志消息消费并写入 PostgreSQL 数据库
pub struct AuditLogConsumerHandler {
    db: Database,
}

impl AuditLogConsumerHandler {
    /// 创建新的审计日志消费者处理器
    ///
    /// # 参数
    /// - `db`: 数据库连接
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 将 Stream 消息转换为数据库记录并插入
    ///
    /// # 参数
    /// - `msg`: 审计日志 Stream 消息
    ///
    /// # 返回
    /// - 成功返回 Ok(())
    /// - 失败返回 Err
    async fn insert_audit_log(&self, msg: &AuditLogStreamMessage) -> anyhow::Result<()> {
        let pool = self.db.pool();

        // 解析 actor_role (字符串直接绑定，SQLx 会自动转换)
        let actor_role = msg.actor_role.as_deref();

        // 解析 metadata
        let metadata_json = msg
            .metadata
            .as_ref()
            .and_then(|m| serde_json::to_value(m).ok());

        // 插入数据库
        // 注意：event_type 和 severity 是 PostgreSQL 枚举类型
        // 但 SQLx 可以通过字符串自动转换
        // 注意：audit_logs 是分区表，主键是 (id, created_at) 复合主键
        sqlx::query(
            r#"
            INSERT INTO audit_logs 
            (id, event_type, severity, actor_id, actor_name, actor_role, 
             target_type, target_id, action, description, metadata, 
             status, error_message, created_at)
            VALUES ($1, $2::audit_event_type, $3::audit_severity, $4, $5, $6::user_role, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (id, created_at) DO NOTHING
            "#,
        )
        .bind(msg.id)
        .bind(&msg.event_type)  // 字符串会自动转换为枚举
        .bind(&msg.severity)     // 字符串会自动转换为枚举
        .bind(msg.actor_id)
        .bind(None::<String>) // actor_name 从 Stream 消息中无法获取
        .bind(actor_role)        // 字符串会自动转换为枚举
        .bind(&msg.target_type)
        .bind(msg.target_id)
        .bind(&msg.action)
        .bind(&msg.description)
        .bind(metadata_json)
        .bind(&msg.status)
        .bind(&msg.error_message)
        .bind(msg.timestamp)
        .execute(pool)
        .await
        .map_err(|e| {
            error!("Failed to insert audit log from stream: {}", e);
            anyhow::anyhow!("Database error: {}", e)
        })?;

        Ok(())
    }
}

#[async_trait]
impl StreamConsumerHandler for AuditLogConsumerHandler {
    type Message = AuditLogStreamMessage;

    /// 处理单条审计日志消息
    ///
    /// # 参数
    /// - `message`: 反序列化后的审计日志消息
    /// - `message_id`: Stream 消息 ID（用于 ACK）
    ///
    /// # 返回
    /// - 处理成功返回 Ok(())
    /// - 处理失败返回 Err（消息会进入 Pending List）
    async fn handle_message(
        &self,
        message: Self::Message,
        message_id: String,
    ) -> anyhow::Result<()> {
        info!(
            "Processing audit log from stream: id={}, event_type={}, message_id={}",
            message.id, message.event_type, message_id
        );

        self.insert_audit_log(&message).await?;

        info!("Successfully processed audit log: id={}", message.id);

        Ok(())
    }

    /// 处理批量消息
    ///
    /// 批量插入以提高性能
    ///
    /// # 参数
    /// - `messages`: 消息列表（消息 + Stream ID）
    ///
    /// # 返回
    /// - 处理成功返回 Ok(())
    /// - 处理失败返回 Err
    async fn handle_batch(&self, messages: Vec<(Self::Message, String)>) -> anyhow::Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        info!(
            "Processing batch of {} audit logs from stream",
            messages.len()
        );

        let pool = self.db.pool();
        let mut tx = pool.begin().await?;

        for (msg, _message_id) in &messages {
            // 解析 actor_role (字符串直接绑定)
            let actor_role = msg.actor_role.as_deref();

            // 解析 metadata
            let metadata_json = msg
                .metadata
                .as_ref()
                .and_then(|m| serde_json::to_value(m).ok());

            // 插入数据库
            // 注意：audit_logs 是分区表，主键是 (id, created_at) 复合主键
            if let Err(e) = sqlx::query(
                r#"
                INSERT INTO audit_logs 
                (id, event_type, severity, actor_id, actor_name, actor_role, 
                 target_type, target_id, action, description, metadata, 
                 status, error_message, created_at)
                VALUES ($1, $2::audit_event_type, $3::audit_severity, $4, $5, $6::user_role, $7, $8, $9, $10, $11, $12, $13, $14)
                ON CONFLICT (id, created_at) DO NOTHING
                "#,
            )
            .bind(msg.id)
            .bind(&msg.event_type)
            .bind(&msg.severity)
            .bind(msg.actor_id)
            .bind(None::<String>)
            .bind(actor_role)
            .bind(&msg.target_type)
            .bind(msg.target_id)
            .bind(&msg.action)
            .bind(&msg.description)
            .bind(metadata_json)
            .bind(&msg.status)
            .bind(&msg.error_message)
            .bind(msg.timestamp)
            .execute(&mut *tx)
            .await
            {
                error!(
                    "Failed to insert audit log {} from stream: {}",
                    msg.id, e
                );
            }
        }

        tx.commit().await?;

        info!(
            "Successfully processed batch of {} audit logs",
            messages.len()
        );

        Ok(())
    }
}
