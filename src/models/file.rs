use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 文件分类
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "file_category", rename_all = "lowercase")]
pub enum FileCategory {
    Image,
    Document,
    Video,
    Audio,
    Other,
}

impl FileCategory {
    /// 根据 MIME 类型判断文件分类
    pub fn from_mime_type(mime_type: &str) -> Self {
        if mime_type.starts_with("image/") {
            Self::Image
        } else if mime_type.starts_with("video/") {
            Self::Video
        } else if mime_type.starts_with("audio/") {
            Self::Audio
        } else if mime_type.starts_with("application/")
            || mime_type.starts_with("text/")
            || mime_type == "application/pdf"
        {
            Self::Document
        } else {
            Self::Other
        }
    }

    /// 获取分类对应的目录名
    pub fn as_directory(&self) -> &'static str {
        match self {
            Self::Image => "images",
            Self::Document => "documents",
            Self::Video => "videos",
            Self::Audio => "audio",
            Self::Other => "other",
        }
    }
}

/// 文件用途类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "file_usage_type", rename_all = "lowercase")]
pub enum FileUsageType {
    Avatar,
    Message,
    RoomCover,
    #[default]
    General,
}

/// 文件资源数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct FileResource {
    pub id: Uuid,
    pub uploader_id: Option<Uuid>,
    pub original_name: String,
    pub storage_name: String,
    pub file_path: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: String,
    pub file_hash: Option<String>,
    pub category: FileCategory,
    pub usage_type: FileUsageType,
    pub room_id: Option<Uuid>,
    pub message_id: Option<Uuid>,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FileResource {
    /// 转换为文件响应DTO
    pub fn to_response(&self) -> FileResponse {
        FileResponse {
            id: self.id,
            original_name: self.original_name.clone(),
            file_url: self.file_url.clone(),
            file_size: self.file_size,
            mime_type: self.mime_type.clone(),
            category: self.category.clone(),
            usage_type: self.usage_type.clone(),
            created_at: self.created_at,
        }
    }

    /// 检查是否是图片
    pub fn is_image(&self) -> bool {
        matches!(self.category, FileCategory::Image)
    }

    /// 检查是否是文档
    pub fn is_document(&self) -> bool {
        matches!(self.category, FileCategory::Document)
    }

    /// 检查是否已被删除
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

/// 文件上传请求
#[derive(Debug, Clone, Deserialize)]
pub struct FileUploadRequest {
    pub usage_type: Option<FileUsageType>,
    pub room_id: Option<Uuid>,
}

/// 文件上传响应
#[derive(Debug, Clone, Serialize)]
pub struct FileUploadResponse {
    pub id: Uuid,
    pub original_name: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: String,
    pub category: FileCategory,
    pub usage_type: FileUsageType,
}

/// 文件响应
#[derive(Debug, Clone, Serialize)]
pub struct FileResponse {
    pub id: Uuid,
    pub original_name: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: String,
    pub category: FileCategory,
    pub usage_type: FileUsageType,
    pub created_at: DateTime<Utc>,
}

/// 文件列表响应
#[derive(Debug, Clone, Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileResponse>,
    pub total: i64,
}

/// 文件查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct FileQueryParams {
    pub category: Option<FileCategory>,
    pub usage_type: Option<FileUsageType>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for FileQueryParams {
    fn default() -> Self {
        Self {
            category: None,
            usage_type: None,
            limit: Some(20),
            offset: Some(0),
        }
    }
}

/// 文件元数据（用于上传处理）
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub original_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub category: FileCategory,
}

/// 允许的图片 MIME 类型
pub const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/svg+xml",
];

/// 允许的文档 MIME 类型
pub const ALLOWED_DOCUMENT_TYPES: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "text/plain",
    "text/markdown",
];

/// 允许的视频 MIME 类型
pub const ALLOWED_VIDEO_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/ogg",
    "video/quicktime",
];

/// 允许的音频 MIME 类型
pub const ALLOWED_AUDIO_TYPES: &[&str] = &[
    "audio/mpeg",
    "audio/ogg",
    "audio/wav",
    "audio/webm",
    "audio/aac",
];

/// 检查 MIME 类型是否允许
pub fn is_allowed_mime_type(mime_type: &str) -> bool {
    ALLOWED_IMAGE_TYPES.contains(&mime_type)
        || ALLOWED_DOCUMENT_TYPES.contains(&mime_type)
        || ALLOWED_VIDEO_TYPES.contains(&mime_type)
        || ALLOWED_AUDIO_TYPES.contains(&mime_type)
}

/// 根据用途类型获取允许的文件类型
pub fn get_allowed_types_for_usage(usage_type: &FileUsageType) -> Vec<&'static str> {
    match usage_type {
        FileUsageType::Avatar => ALLOWED_IMAGE_TYPES.to_vec(),
        FileUsageType::Message => [
            ALLOWED_IMAGE_TYPES,
            ALLOWED_DOCUMENT_TYPES,
            ALLOWED_VIDEO_TYPES,
            ALLOWED_AUDIO_TYPES,
        ]
        .concat(),
        FileUsageType::RoomCover => ALLOWED_IMAGE_TYPES.to_vec(),
        FileUsageType::General => [
            ALLOWED_IMAGE_TYPES,
            ALLOWED_DOCUMENT_TYPES,
            ALLOWED_VIDEO_TYPES,
            ALLOWED_AUDIO_TYPES,
        ]
        .concat(),
    }
}
