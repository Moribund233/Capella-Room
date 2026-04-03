use std::path::{Path, PathBuf};

use chrono::{Datelike, Utc};
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    config::AppConfig,
    db::Database,
    error::{AppError, Result},
    models::file::{
        FileCategory, FileListResponse, FileQueryParams, FileResource,
        FileResponse, FileUploadResponse, FileUsageType, is_allowed_mime_type,
    },
};

/// 文件服务
pub struct FileService {
    db: Database,
    upload_dir: String,
    base_url: String,
    max_file_size: usize,
}

impl FileService {
    /// 创建文件服务
    pub fn new(db: Database, upload_dir: String, base_url: String, max_file_size: usize) -> Self {
        Self {
            db,
            upload_dir,
            base_url,
            max_file_size,
        }
    }

    /// 从配置创建文件服务
    pub fn from_config(db: Database, config: &crate::config::UploadConfig) -> anyhow::Result<Self> {
        let upload_dir = AppConfig::upload_dir()?;
        Ok(Self::new(
            db,
            upload_dir,
            config.base_url.clone(),
            config.max_file_size,
        ))
    }

    /// 上传文件
    pub async fn upload_file(
        &self,
        uploader_id: Uuid,
        file_data: Vec<u8>,
        original_name: &str,
        mime_type: &str,
        usage_type: FileUsageType,
        room_id: Option<Uuid>,
    ) -> Result<FileUploadResponse> {
        // 验证文件大小
        if file_data.len() > self.max_file_size {
            return Err(AppError::Validation(format!(
                "文件大小超过限制，最大允许 {} 字节",
                self.max_file_size
            )));
        }

        // 验证 MIME 类型
        if !is_allowed_mime_type(mime_type) {
            return Err(AppError::Validation(format!(
                "不支持的文件类型: {}",
                mime_type
            )));
        }

        // 确定文件分类
        let category = FileCategory::from_mime_type(mime_type);

        // 计算文件哈希
        let file_hash = self.calculate_hash(&file_data);

        // 检查是否已存在相同文件（去重）
        let existing_file = self.find_by_hash(&file_hash).await?;

        // 生成存储文件名
        let file_id = Uuid::new_v4();
        let extension = Path::new(original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");
        let storage_name = format!("{}. {}", file_id, extension);

        // 构建存储路径: {category}/{year}/{month}/{filename}
        let now = Utc::now();
        let relative_dir = format!(
            "{}/{}/{:02}",
            category.as_directory(),
            now.year(),
            now.month()
        );
        let file_path = format!("{}/{}", relative_dir, storage_name);
        let full_path = PathBuf::from(&self.upload_dir).join(&file_path);

        // 如果文件已存在，使用已存在文件的路径（硬链接或复用路径）
        // 否则创建新文件
        if existing_file.is_none() {
            // 创建目录
            let parent_dir = full_path.parent().ok_or(AppError::Internal)?;
            fs::create_dir_all(parent_dir).await.map_err(|e| {
                AppError::Validation(format!("创建目录失败: {}", e))
            })?;

            // 保存文件
            let mut file = fs::File::create(&full_path).await.map_err(|e| {
                AppError::Validation(format!("创建文件失败: {}", e))
            })?;
            file.write_all(&file_data).await.map_err(|e| {
                AppError::Validation(format!("写入文件失败: {}", e))
            })?;
            file.flush().await.map_err(|e| {
                AppError::Validation(format!("刷新文件失败: {}", e))
            })?;
        }

        // 构建访问 URL
        let file_url = format!("{}/{}", self.base_url, file_path.replace('\\', "/"));

        // 保存到数据库
        let file_resource = sqlx::query_as::<_, FileResource>(
            r#"
            INSERT INTO file_resources 
                (uploader_id, original_name, storage_name, file_path, file_url, 
                 file_size, mime_type, file_hash, category, usage_type, room_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(uploader_id)
        .bind(original_name)
        .bind(&storage_name)
        .bind(&file_path)
        .bind(&file_url)
        .bind(file_data.len() as i64)
        .bind(mime_type)
        .bind(&file_hash)
        .bind(&category)
        .bind(&usage_type)
        .bind(room_id)
        .fetch_one(self.db.pool())
        .await?;

        Ok(FileUploadResponse {
            id: file_resource.id,
            original_name: file_resource.original_name,
            file_url: file_resource.file_url,
            file_size: file_resource.file_size,
            mime_type: file_resource.mime_type,
            category: file_resource.category,
            usage_type: file_resource.usage_type,
        })
    }

    /// 根据哈希查找文件
    async fn find_by_hash(&self, hash: &str) -> Result<Option<FileResource>> {
        let file = sqlx::query_as::<_, FileResource>(
            r#"
            SELECT * FROM file_resources 
            WHERE file_hash = $1 AND is_deleted = false
            LIMIT 1
            "#,
        )
        .bind(hash)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(file)
    }

    /// 计算文件哈希
    fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// 根据 ID 获取文件
    pub async fn get_file_by_id(&self, file_id: Uuid) -> Result<FileResource> {
        let file = sqlx::query_as::<_, FileResource>(
            r#"
            SELECT * FROM file_resources 
            WHERE id = $1 AND is_deleted = false
            "#,
        )
        .bind(file_id)
        .fetch_optional(self.db.pool())
        .await?
        .ok_or(AppError::NotFound)?;

        Ok(file)
    }

    /// 获取用户上传的文件列表
    pub async fn get_files_by_uploader(
        &self,
        uploader_id: Uuid,
        params: FileQueryParams,
    ) -> Result<FileListResponse> {
        let limit = params.limit.unwrap_or(20).clamp(1, 100);
        let offset = params.offset.unwrap_or(0).max(0);

        let files = match (&params.category, &params.usage_type) {
            (Some(cat), Some(usage)) => {
                sqlx::query_as::<_, FileResource>(
                    r#"
                    SELECT * FROM file_resources 
                    WHERE uploader_id = $1 
                    AND category = $2 
                    AND usage_type = $3 
                    AND is_deleted = false
                    ORDER BY created_at DESC
                    LIMIT $4 OFFSET $5
                    "#,
                )
                .bind(uploader_id)
                .bind(cat)
                .bind(usage)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            }
            (Some(cat), None) => {
                sqlx::query_as::<_, FileResource>(
                    r#"
                    SELECT * FROM file_resources 
                    WHERE uploader_id = $1 
                    AND category = $2 
                    AND is_deleted = false
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(uploader_id)
                .bind(cat)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            }
            (None, Some(usage)) => {
                sqlx::query_as::<_, FileResource>(
                    r#"
                    SELECT * FROM file_resources 
                    WHERE uploader_id = $1 
                    AND usage_type = $2 
                    AND is_deleted = false
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(uploader_id)
                .bind(usage)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, FileResource>(
                    r#"
                    SELECT * FROM file_resources 
                    WHERE uploader_id = $1 
                    AND is_deleted = false
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(uploader_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            }
        };

        // 获取总数
        let total = match (&params.category, &params.usage_type) {
            (Some(cat), Some(usage)) => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM file_resources 
                    WHERE uploader_id = $1 
                    AND category = $2 
                    AND usage_type = $3 
                    AND is_deleted = false
                    "#,
                )
                .bind(uploader_id)
                .bind(cat)
                .bind(usage)
                .fetch_one(self.db.pool())
                .await?
            }
            (Some(cat), None) => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM file_resources 
                    WHERE uploader_id = $1 
                    AND category = $2 
                    AND is_deleted = false
                    "#,
                )
                .bind(uploader_id)
                .bind(cat)
                .fetch_one(self.db.pool())
                .await?
            }
            (None, Some(usage)) => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM file_resources 
                    WHERE uploader_id = $1 
                    AND usage_type = $2 
                    AND is_deleted = false
                    "#,
                )
                .bind(uploader_id)
                .bind(usage)
                .fetch_one(self.db.pool())
                .await?
            }
            (None, None) => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM file_resources 
                    WHERE uploader_id = $1 
                    AND is_deleted = false
                    "#,
                )
                .bind(uploader_id)
                .fetch_one(self.db.pool())
                .await?
            }
        };

        let files: Vec<FileResponse> = files.into_iter().map(|f| f.to_response()).collect();

        Ok(FileListResponse { files, total })
    }

    /// 软删除文件
    pub async fn delete_file(&self, file_id: Uuid, uploader_id: Uuid) -> Result<()> {
        // 检查文件是否存在且属于该用户
        let file = self.get_file_by_id(file_id).await?;

        if file.uploader_id != Some(uploader_id) {
            return Err(AppError::Forbidden);
        }

        // 软删除
        sqlx::query(
            r#"
            UPDATE file_resources 
            SET is_deleted = true, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(file_id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 关联文件到消息
    pub async fn link_file_to_message(
        &self,
        file_id: Uuid,
        message_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE file_resources 
            SET message_id = $1, usage_type = 'message', updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(message_id)
        .bind(file_id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 获取文件的完整路径
    pub fn get_full_path(&self, file_path: &str) -> PathBuf {
        PathBuf::from(&self.upload_dir).join(file_path)
    }

    /// 获取最大文件大小
    pub fn max_file_size(&self) -> usize {
        self.max_file_size
    }

    /// 获取基础URL
    pub fn get_base_url(&self) -> String {
        self.base_url.clone()
    }
}
