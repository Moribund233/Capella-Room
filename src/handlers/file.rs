use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::file::{
        FileQueryParams, FileUsageType, UploadChunkResponse, UploadInitRequest,
        UploadInitResponse, UploadSessionResponse,
    },
    services::auth_service::Claims,
    services::upload_session::UploadSessionManager,
    state::AppState,
};

/// 通用文件上传
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    debug!("Processing file upload request");

    let uploader_id = parse_user_id(&claims.sub)?;
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_name: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut usage_type = FileUsageType::General;
    let mut room_id: Option<Uuid> = None;

    // 解析 multipart 表单
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        AppError::Validation(format!("读取上传数据失败: {}", e))
    })? {
        let name: Option<String> = field.name().map(|s| s.to_string());
        let file_name: Option<String> = field.file_name().map(|s| s.to_string());
        let content_type: Option<String> = field.content_type().map(|s| s.to_string());

        match name.as_deref() {
            Some("file") => {
                original_name = file_name;
                mime_type = content_type;
                file_data = Some(field.bytes().await.map_err(|e| {
                    error!("Failed to read file bytes: {}", e);
                    AppError::Validation(format!("读取文件失败: {}", e))
                })?.to_vec());
            }
            Some("usage_type") => {
                let value: String = field.text().await.map_err(|e| {
                    AppError::Validation(format!("读取 usage_type 失败: {}", e))
                })?;
                usage_type = match value.as_str() {
                    "avatar" => FileUsageType::Avatar,
                    "message" => FileUsageType::Message,
                    "room_cover" => FileUsageType::RoomCover,
                    _ => FileUsageType::General,
                };
            }
            Some("room_id") => {
                let value: String = field.text().await.map_err(|e| {
                    AppError::Validation(format!("读取 room_id 失败: {}", e))
                })?;
                room_id = Uuid::parse_str(&value).ok();
            }
            _ => {}
        }
    }

    // 验证必要字段
    let file_data = file_data.ok_or_else(|| {
        AppError::Validation("缺少文件数据".to_string())
    })?;
    let original_name = original_name.unwrap_or_else(|| "unnamed".to_string());
    let mime_type = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());

    debug!(
        "Uploading file: name={}, size={}, mime={}",
        original_name,
        file_data.len(),
        mime_type
    );

    // 调用服务层上传文件
    let response = state
        .file_service()
        .upload_file(
            uploader_id,
            file_data,
            &original_name,
            &mime_type,
            usage_type,
            room_id,
        )
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

/// 上传图片（专门用于图片上传）
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    debug!("Processing image upload request");

    let uploader_id = parse_user_id(&claims.sub)?;
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_name: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut room_id: Option<Uuid> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("读取上传数据失败: {}", e))
    })? {
        let name: Option<String> = field.name().map(|s| s.to_string());
        let file_name: Option<String> = field.file_name().map(|s| s.to_string());
        let content_type: Option<String> = field.content_type().map(|s| s.to_string());

        match name.as_deref() {
            Some("file") => {
                original_name = file_name;
                mime_type = content_type;
                file_data = Some(field.bytes().await.map_err(|e| {
                    AppError::Validation(format!("读取文件失败: {}", e))
                })?.to_vec());
            }
            Some("room_id") => {
                let value: String = field.text().await.map_err(|e| {
                    AppError::Validation(format!("读取 room_id 失败: {}", e))
                })?;
                room_id = Uuid::parse_str(&value).ok();
            }
            _ => {}
        }
    }

    let file_data = file_data.ok_or_else(|| {
        AppError::Validation("缺少文件数据".to_string())
    })?;
    let original_name = original_name.unwrap_or_else(|| "image.png".to_string());
    let mime_type = mime_type.unwrap_or_else(|| "image/png".to_string());

    // 验证是图片类型
    if !mime_type.starts_with("image/") {
        return Err(AppError::Validation("只允许上传图片文件".to_string()));
    }

    let response = state
        .file_service()
        .upload_file(
            uploader_id,
            file_data,
            &original_name,
            &mime_type,
            FileUsageType::Message,
            room_id,
        )
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

/// 上传头像
pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    debug!("Processing avatar upload request");

    let uploader_id = parse_user_id(&claims.sub)?;
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_name: Option<String> = None;
    let mut mime_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("读取上传数据失败: {}", e))
    })? {
        let name: Option<String> = field.name().map(|s| s.to_string());
        let file_name: Option<String> = field.file_name().map(|s| s.to_string());
        let content_type: Option<String> = field.content_type().map(|s| s.to_string());

        if name.as_deref() == Some("file") {
            original_name = file_name;
            mime_type = content_type;
            file_data = Some(field.bytes().await.map_err(|e| {
                AppError::Validation(format!("读取文件失败: {}", e))
            })?.to_vec());
        }
    }

    let file_data = file_data.ok_or_else(|| {
        AppError::Validation("缺少文件数据".to_string())
    })?;
    let original_name = original_name.unwrap_or_else(|| "avatar.png".to_string());
    let mime_type = mime_type.unwrap_or_else(|| "image/png".to_string());

    // 验证是图片类型
    if !mime_type.starts_with("image/") {
        return Err(AppError::Validation("头像必须是图片文件".to_string()));
    }

    let response = state
        .file_service()
        .upload_file(
            uploader_id,
            file_data,
            &original_name,
            &mime_type,
            FileUsageType::Avatar,
            None,
        )
        .await?;

    // 更新用户头像 URL
    state
        .user_service()
        .update_user_avatar(uploader_id, Some(&response.file_url))
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": response.id,
            "file_url": response.file_url,
            "message": "头像上传成功"
        }
    })))
}

/// 获取文件信息
pub async fn get_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<Uuid>,
) -> Result<Json<Value>> {
    let file = state.file_service().get_file_by_id(file_id).await?;
    Ok(Json(json!({
        "success": true,
        "data": file.to_response()
    })))
}

/// 获取当前用户的文件列表
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<FileQueryParams>,
) -> Result<Json<Value>> {
    let uploader_id = parse_user_id(&claims.sub)?;

    let result = state
        .file_service()
        .get_files_by_uploader(uploader_id, params)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": result
    })))
}

/// 删除文件
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(file_id): Path<Uuid>,
) -> Result<StatusCode> {
    let uploader_id = parse_user_id(&claims.sub)?;

    state.file_service().delete_file(file_id, uploader_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── 分片上传 ────────────────────────────────────────────────

/// 初始化分片上传会话
pub async fn init_chunked_upload(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UploadInitRequest>,
) -> Result<Json<UploadInitResponse>> {
    let config = state.config.read().await;

    if !config.upload.chunked_upload_enabled {
        return Err(AppError::Validation("分片上传未启用".to_string()));
    }

    let user_id = parse_user_id(&claims.sub)?;

    if req.file_size == 0 {
        return Err(AppError::Validation("文件大小不能为0".to_string()));
    }
    if req.total_chunks == 0 {
        return Err(AppError::Validation("分片数不能为0".to_string()));
    }

    let usage_type = req.usage_type.unwrap_or(FileUsageType::General);
    let manager = UploadSessionManager::new(&config.upload);
    let meta = manager.create_session(
        user_id, &req.file_name, req.file_size,
        &req.mime_type, usage_type, req.total_chunks,
    );

    drop(config);

    manager.save_metadata(&meta).await.map_err(|e| {
        error!("保存上传会话元数据失败: {}", e);
        AppError::Internal
    })?;

    Ok(Json(UploadInitResponse {
        session_id: meta.session_id.to_string(),
        chunk_size: meta.chunk_size,
        total_chunks: meta.total_chunks,
    }))
}

/// 上传单个分片
pub async fn upload_chunk(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((session_id, chunk_index)): Path<(String, u32)>,
    mut multipart: Multipart,
) -> Result<Json<UploadChunkResponse>> {
    let user_id = parse_user_id(&claims.sub)?;
    let sid = Uuid::parse_str(&session_id).map_err(|_| {
        AppError::Validation("无效的会话ID".to_string())
    })?;

    let config = state.config.read().await;
    let manager = UploadSessionManager::new(&config.upload);

    let meta = manager.get_session_status(sid).await.map_err(|_| {
        AppError::NotFound
    })?;

    if meta.user_id != user_id {
        return Err(AppError::Auth("无权操作此上传会话".to_string()));
    }

    if chunk_index >= meta.total_chunks {
        return Err(AppError::Validation(format!(
            "分片索引超出范围: {} >= {}", chunk_index, meta.total_chunks
        )));
    }

    drop(config);

    if manager.chunk_exists(sid, chunk_index).await {
        let meta = manager.get_session_status(sid).await.map_err(|_| {
            AppError::NotFound
        })?;
        return Ok(Json(UploadChunkResponse {
            received: meta.received_chunks.len() as u32,
            total: meta.total_chunks,
        }));
    }

    let mut chunk_data: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("读取分片数据失败: {}", e);
        AppError::Validation(format!("读取分片数据失败: {}", e))
    })? {
        if field.name() == Some("chunk") {
            chunk_data = Some(field.bytes().await.map_err(|e| {
                error!("读取分片字节失败: {}", e);
                AppError::Validation(format!("读取分片数据失败: {}", e))
            })?.to_vec());
            break;
        }
    }

    let data = chunk_data.ok_or_else(|| {
        AppError::Validation("未找到分片数据（字段名: chunk）".to_string())
    })?;

    manager.save_chunk(&meta, chunk_index, &data).await.map_err(|e| {
        error!("保存分片失败: {}", e);
        AppError::Internal
    })?;

    let meta = manager.update_received_chunk(sid, chunk_index).await.map_err(|e| {
        error!("更新会话元数据失败: {}", e);
        AppError::Internal
    })?;

    Ok(Json(UploadChunkResponse {
        received: meta.received_chunks.len() as u32,
        total: meta.total_chunks,
    }))
}

/// 查询上传会话状态
pub async fn get_upload_status(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<String>,
) -> Result<Json<UploadSessionResponse>> {
    let user_id = parse_user_id(&claims.sub)?;
    let sid = Uuid::parse_str(&session_id).map_err(|_| {
        AppError::Validation("无效的会话ID".to_string())
    })?;

    let config = state.config.read().await;
    let manager = UploadSessionManager::new(&config.upload);
    drop(config);

    let meta = manager.get_session_status(sid).await.map_err(|_| {
        AppError::NotFound
    })?;

    if meta.user_id != user_id {
        return Err(AppError::Auth("无权查看此上传会话".to_string()));
    }

    let total = meta.total_chunks;
    let received: Vec<u32> = meta.received_chunks.clone();
    let missing: Vec<u32> = (0..total).filter(|i| !received.contains(i)).collect();

    Ok(Json(UploadSessionResponse {
        session_id: meta.session_id.to_string(),
        file_name: meta.file_name,
        file_size: meta.file_size,
        mime_type: meta.mime_type,
        status: format!("{:?}", meta.status).to_lowercase(),
        total_chunks: total,
        received_chunks: received,
        missing_chunks: missing,
        created_at: meta.created_at.to_rfc3339(),
    }))
}

/// 完成分片上传（合并分片并存储）
pub async fn complete_chunked_upload(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<String>,
) -> Result<Json<Value>> {
    let user_id = parse_user_id(&claims.sub)?;
    let sid = Uuid::parse_str(&session_id).map_err(|_| {
        AppError::Validation("无效的会话ID".to_string())
    })?;

    let config = state.config.read().await;
    let manager = UploadSessionManager::new(&config.upload);
    drop(config);

    let meta = manager.get_session_status(sid).await.map_err(|_| {
        AppError::NotFound
    })?;

    if meta.user_id != user_id {
        return Err(AppError::Auth("无权完成此上传".to_string()));
    }

    if meta.received_chunks.len() as u32 != meta.total_chunks {
        return Err(AppError::Validation(format!(
            "分片未上传完成: {}/{}", meta.received_chunks.len(), meta.total_chunks
        )));
    }

    let file_data = manager.merge_chunks(&meta).await.map_err(|e| {
        error!("合并分片失败: {}", e);
        AppError::Internal
    })?;

    let result = state
        .file_service()
        .upload_file(user_id, file_data, &meta.file_name, &meta.mime_type, meta.usage_type, None)
        .await?;

    if let Err(e) = manager.cleanup_session(sid).await {
        warn!("清理临时分片失败: {}", e);
    }

    Ok(Json(json!({
        "file": result,
        "session_id": session_id,
    })))
}

/// 取消分片上传
pub async fn cancel_chunked_upload(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<String>,
) -> Result<StatusCode> {
    let user_id = parse_user_id(&claims.sub)?;
    let sid = Uuid::parse_str(&session_id).map_err(|_| {
        AppError::Validation("无效的会话ID".to_string())
    })?;

    let config = state.config.read().await;
    let manager = UploadSessionManager::new(&config.upload);
    drop(config);

    let meta = manager.get_session_status(sid).await.map_err(|_| {
        AppError::NotFound
    })?;

    if meta.user_id != user_id {
        return Err(AppError::Auth("无权取消此上传".to_string()));
    }

    manager.cleanup_session(sid).await.map_err(|e| {
        error!("清理上传会话失败: {}", e);
        AppError::Internal
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// 解析用户ID字符串为 UUID
fn parse_user_id(sub: &str) -> Result<Uuid> {
    Uuid::parse_str(sub).map_err(|_| {
        AppError::Auth("无效的用户ID".to_string())
    })
}
