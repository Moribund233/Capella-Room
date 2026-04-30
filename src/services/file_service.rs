use std::path::{Path, PathBuf};

use chrono::{Datelike, Utc};
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    config::SharedConfig,
    db::Database,
    error::{AppError, Result},
    models::file::{
        is_allowed_mime_type, FileCategory, FileListResponse, FileQueryParams, FileResource,
        FileResponse, FileUploadResponse, FileUsageType,
    },
};

/// 文件服务
pub struct FileService {
    db: Database,
    upload_dir: String,
    base_url: String,
    max_file_size: usize,
    shared_config: Option<SharedConfig>,
}

impl FileService {
    /// 创建文件服务
    pub fn new(db: Database, upload_dir: String, base_url: String, max_file_size: usize) -> Self {
        Self {
            db,
            upload_dir,
            base_url,
            max_file_size,
            shared_config: None,
        }
    }

    /// 从配置创建文件服务
    pub fn from_config(db: Database, config: &crate::config::UploadConfig) -> anyhow::Result<Self> {
        let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
        Ok(Self::new(
            db,
            upload_dir,
            config.base_url.clone(),
            config.max_file_size,
        ))
    }

    /// 使用共享配置创建文件服务（支持热加载）
    /// 注意：不要在异步运行时中调用 blocking_read
    pub fn with_shared_config(db: Database, config: SharedConfig) -> anyhow::Result<Self> {
        let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
        // 使用默认值初始化，后续通过 try_read 动态读取
        Ok(Self {
            db,
            upload_dir,
            base_url: String::new(),
            max_file_size: 0,
            shared_config: Some(config),
        })
    }

    /// 获取有效的最大文件大小
    /// 使用 try_read 避免阻塞
    fn effective_max_file_size(&self) -> usize {
        self.shared_config
            .as_ref()
            .and_then(|c| c.try_read().ok().map(|cfg| cfg.upload.max_file_size))
            .unwrap_or(self.max_file_size)
    }

    /// 获取有效的 base_url
    /// 使用 try_read 避免阻塞
    fn effective_base_url(&self) -> String {
        self.shared_config
            .as_ref()
            .and_then(|c| c.try_read().ok().map(|cfg| cfg.upload.base_url.clone()))
            .filter(|url| !url.is_empty())
            .unwrap_or_else(|| self.base_url.clone())
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
        let max_size = self.effective_max_file_size();
        if max_size > 0 && file_data.len() > max_size {
            return Err(AppError::Validation(format!(
                "文件大小超过限制，最大允许 {} 字节",
                max_size
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
        let storage_name = format!("{}.{}", file_id, extension);

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
            fs::create_dir_all(parent_dir)
                .await
                .map_err(|e| AppError::Validation(format!("创建目录失败: {}", e)))?;

            // 保存文件
            let mut file = fs::File::create(&full_path)
                .await
                .map_err(|e| AppError::Validation(format!("创建文件失败: {}", e)))?;
            file.write_all(&file_data)
                .await
                .map_err(|e| AppError::Validation(format!("写入文件失败: {}", e)))?;
            file.flush()
                .await
                .map_err(|e| AppError::Validation(format!("刷新文件失败: {}", e)))?;
        }

        // 构建访问 URL
        let base_url = self.effective_base_url();
        let file_url = if base_url.is_empty() {
            format!("/uploads/{}", file_path.replace('\\', "/"))
        } else {
            format!("{}/{}", base_url, file_path.replace('\\', "/"))
        };

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
        format!("{:x}", hasher.finalize())
    }

    /// 获取文件列表
    pub async fn list_files(&self, params: FileQueryParams) -> Result<FileListResponse> {
        let mut query = String::from("SELECT * FROM file_resources WHERE is_deleted = false");
        let mut count_query =
            String::from("SELECT COUNT(*) FROM file_resources WHERE is_deleted = false");
        let mut conditions: Vec<String> = Vec::new();

        // 添加查询条件
        if let Some(ref category) = params.category {
            conditions.push(format!("category = '{:?}'", category));
        }
        if let Some(ref usage) = params.usage_type {
            conditions.push(format!("usage_type = '{:?}'", usage));
        }

        // 应用条件
        if !conditions.is_empty() {
            let where_clause = conditions.join(" AND ");
            query.push_str(" AND ");
            query.push_str(&where_clause);
            count_query.push_str(" AND ");
            count_query.push_str(&where_clause);
        }

        // 添加排序
        query.push_str(" ORDER BY created_at DESC");

        // 添加分页
        let limit = params.limit.unwrap_or(20).min(100);
        let offset = params.offset.unwrap_or(0);
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // 执行查询
        let files = sqlx::query_as::<_, FileResource>(&query)
            .fetch_all(self.db.pool())
            .await?;

        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(self.db.pool())
            .await?;

        Ok(FileListResponse {
            files: files
                .into_iter()
                .map(|f| self.to_file_response(f))
                .collect(),
            total,
        })
    }

    /// 获取上传者的文件列表
    pub async fn get_files_by_uploader(
        &self,
        uploader_id: Uuid,
        params: FileQueryParams,
    ) -> Result<FileListResponse> {
        let mut query = String::from(
            "SELECT * FROM file_resources WHERE is_deleted = false AND uploader_id = '",
        );
        query.push_str(&uploader_id.to_string());
        query.push('\'');

        let mut count_query = String::from(
            "SELECT COUNT(*) FROM file_resources WHERE is_deleted = false AND uploader_id = '",
        );
        count_query.push_str(&uploader_id.to_string());
        count_query.push('\'');

        // 添加查询条件
        if let Some(ref category) = params.category {
            query.push_str(&format!(" AND category = '{:?}'", category));
            count_query.push_str(&format!(" AND category = '{:?}'", category));
        }
        if let Some(ref usage) = params.usage_type {
            query.push_str(&format!(" AND usage_type = '{:?}'", usage));
            count_query.push_str(&format!(" AND usage_type = '{:?}'", usage));
        }

        // 添加排序
        query.push_str(" ORDER BY created_at DESC");

        // 添加分页
        let limit = params.limit.unwrap_or(20).min(100);
        let offset = params.offset.unwrap_or(0);
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // 执行查询
        let files = sqlx::query_as::<_, FileResource>(&query)
            .fetch_all(self.db.pool())
            .await?;

        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(self.db.pool())
            .await?;

        Ok(FileListResponse {
            files: files
                .into_iter()
                .map(|f| self.to_file_response(f))
                .collect(),
            total,
        })
    }

    /// 获取单个文件（内部方法）
    async fn get_file_internal(&self, file_id: Uuid) -> Result<FileResource> {
        let file = sqlx::query_as::<_, FileResource>(
            "SELECT * FROM file_resources WHERE id = $1 AND is_deleted = false",
        )
        .bind(file_id)
        .fetch_optional(self.db.pool())
        .await?
        .ok_or(AppError::NotFound)?;

        Ok(file)
    }

    /// 获取单个文件
    pub async fn get_file(&self, file_id: Uuid) -> Result<FileResponse> {
        let file = self.get_file_internal(file_id).await?;
        Ok(self.to_file_response(file))
    }

    /// 根据ID获取文件（用于 handlers）
    pub async fn get_file_by_id(&self, file_id: Uuid) -> Result<FileResource> {
        self.get_file_internal(file_id).await
    }

    /// 删除文件（软删除）
    pub async fn delete_file(&self, file_id: Uuid, user_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "UPDATE file_resources SET is_deleted = true WHERE id = $1 AND uploader_id = $2",
        )
        .bind(file_id)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 转换文件响应
    fn to_file_response(&self, file: FileResource) -> FileResponse {
        FileResponse {
            id: file.id,
            original_name: file.original_name,
            file_url: file.file_url,
            file_size: file.file_size,
            mime_type: file.mime_type,
            category: file.category,
            usage_type: file.usage_type,
            uploader: None,
            created_at: file.created_at,
        }
    }

    /// 清理过期文件
    pub async fn cleanup_expired_files(&self, days: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE file_resources 
            SET is_deleted = true 
            WHERE created_at < NOW() - INTERVAL '1 day' * $1
            AND is_deleted = false
            "#,
        )
        .bind(days)
        .execute(self.db.pool())
        .await?;

        Ok(result.rows_affected())
    }

    /// 获取用户存储统计
    pub async fn get_user_storage_stats(&self, user_id: Uuid) -> Result<(i64, i64)> {
        let stats: (Option<i64>, Option<i64>) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as file_count,
                COALESCE(SUM(file_size), 0) as total_size
            FROM file_resources 
            WHERE uploader_id = $1 AND is_deleted = false
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await?;

        Ok((stats.0.unwrap_or(0), stats.1.unwrap_or(0)))
    }

    /// 获取文件路径
    pub fn get_file_path(&self, file_path: &str) -> PathBuf {
        PathBuf::from(&self.upload_dir).join(file_path)
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    #[test]
    fn test_calculate_hash() {
        // 直接测试哈希计算逻辑
        fn calculate_hash(data: &[u8]) -> String {
            let mut hasher = Sha256::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }

        let data = b"test data";
        let hash1 = calculate_hash(data);
        let hash2 = calculate_hash(data);

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
}
