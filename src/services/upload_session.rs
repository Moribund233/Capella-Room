use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::config::UploadConfig;
use crate::models::file::FileUsageType;

/// 上传会话状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    Active,
    Completed,
    Cancelled,
}

/// 上传会话元数据（Redis 存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSessionMeta {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub usage_type: FileUsageType,
    pub total_chunks: u32,
    pub received_chunks: Vec<u32>,
    pub chunk_size: u32,
    pub status: UploadStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// 上传会话管理器
#[derive(Debug)]
pub struct UploadSessionManager {
    upload_dir: String,
    pub default_chunk_size: u32,
    pub session_ttl_hours: u64,
}

impl UploadSessionManager {
    pub fn new(config: &UploadConfig) -> Self {
        let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
        Self {
            upload_dir,
            default_chunk_size: config.default_chunk_size,
            session_ttl_hours: config.session_ttl_hours,
        }
    }

    fn chunks_dir(&self, session_id: Uuid) -> PathBuf {
        PathBuf::from(&self.upload_dir).join(".chunks").join(session_id.to_string())
    }

    /// 创建上传会话
    pub fn create_session(
        &self,
        user_id: Uuid,
        file_name: &str,
        file_size: u64,
        mime_type: &str,
        usage_type: FileUsageType,
        total_chunks: u32,
    ) -> UploadSessionMeta {
        let chunk_size = self.default_chunk_size;
        let now = Utc::now();
        UploadSessionMeta {
            session_id: Uuid::new_v4(),
            user_id,
            file_name: file_name.to_string(),
            file_size,
            mime_type: mime_type.to_string(),
            usage_type,
            total_chunks,
            received_chunks: Vec::new(),
            chunk_size,
            status: UploadStatus::Active,
            created_at: now,
            expires_at: now + chrono::Duration::hours(self.session_ttl_hours as i64),
        }
    }

    /// 保存分片到磁盘
    pub async fn save_chunk(
        &self,
        session: &UploadSessionMeta,
        chunk_index: u32,
        data: &[u8],
    ) -> anyhow::Result<()> {
        let dir = self.chunks_dir(session.session_id);
        tokio::fs::create_dir_all(&dir).await?;

        let path = dir.join(chunk_index.to_string());
        let mut file = tokio::fs::File::create(&path).await?;
        file.write_all(data).await?;

        debug!(
            "Saved chunk {} for session {} ({} bytes)",
            chunk_index, session.session_id, data.len()
        );

        Ok(())
    }

    /// 合并分片为完整文件
    pub async fn merge_chunks(
        &self,
        session: &UploadSessionMeta,
    ) -> anyhow::Result<Vec<u8>> {
        let dir = self.chunks_dir(session.session_id);
        let mut file_data = Vec::with_capacity(session.file_size as usize);

        for i in 0..session.total_chunks {
            let path = dir.join(i.to_string());
            let chunk = tokio::fs::read(&path).await?;
            file_data.extend_from_slice(&chunk);
        }

        debug!(
            "Merged {} chunks for session {} (total: {} bytes)",
            session.total_chunks, session.session_id, file_data.len()
        );

        Ok(file_data)
    }

    /// 清理会话的临时分片
    pub async fn cleanup_session(&self, session_id: Uuid) -> anyhow::Result<()> {
        let dir = self.chunks_dir(session_id);
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir).await?;
            debug!("Cleaned up chunks for session {}", session_id);
        }
        Ok(())
    }

    /// 清理过期的临时目录
    pub async fn cleanup_expired_sessions(&self, sessions: &[UploadSessionMeta]) {
        let now = Utc::now();
        for session in sessions {
            if session.expires_at < now && session.status == UploadStatus::Active {
                if let Err(e) = self.cleanup_session(session.session_id).await {
                    warn!("Failed to cleanup expired session {}: {}", session.session_id, e);
                }
            }
        }
    }

    /// 保存会话元数据到磁盘
    pub async fn save_metadata(&self, meta: &UploadSessionMeta) -> anyhow::Result<()> {
        let dir = self.chunks_dir(meta.session_id);
        tokio::fs::create_dir_all(&dir).await?;
        let json = serde_json::to_string(meta)?;
        tokio::fs::write(dir.join("metadata.json"), &json).await?;
        Ok(())
    }

    /// 从磁盘加载会话元数据
    pub async fn load_metadata(&self, session_id: Uuid) -> anyhow::Result<UploadSessionMeta> {
        let path = self.chunks_dir(session_id).join("metadata.json");
        let json = tokio::fs::read_to_string(&path).await?;
        Ok(serde_json::from_str(&json)?)
    }

    /// 获取会话状态（扫描已接收分片）
    pub async fn get_session_status(&self, session_id: Uuid) -> anyhow::Result<UploadSessionMeta> {
        let meta = self.load_metadata(session_id).await?;
        let dir = self.chunks_dir(session_id);
        let mut received = Vec::new();
        if dir.exists() {
            let mut entries = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await.map(|t| t.is_file()).unwrap_or(false) {
                    if let Some(name) = entry.file_name().to_str() {
                        if let Ok(idx) = name.parse::<u32>() {
                            received.push(idx);
                        }
                    }
                }
            }
        }
        received.sort();
        let mut meta = meta;
        meta.received_chunks = received;
        Ok(meta)
    }

    /// 更新会话元数据中的已接收分片并写回磁盘
    pub async fn update_received_chunk(
        &self,
        session_id: Uuid,
        chunk_index: u32,
    ) -> anyhow::Result<UploadSessionMeta> {
        let mut meta = self.load_metadata(session_id).await?;
        if !meta.received_chunks.contains(&chunk_index) {
            meta.received_chunks.push(chunk_index);
            self.save_metadata(&meta).await?;
        }
        Ok(meta)
    }

    /// 检查分片是否已存在（幂等上传）
    pub async fn chunk_exists(&self, session_id: Uuid, chunk_index: u32) -> bool {
        let path = self.chunks_dir(session_id).join(chunk_index.to_string());
        path.exists()
    }
}
