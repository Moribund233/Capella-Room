use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::file::{FileQueryParams, FileUsageType},
    services::auth_service::Claims,
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

/// 解析用户ID字符串为 UUID
fn parse_user_id(sub: &str) -> Result<Uuid> {
    Uuid::parse_str(sub).map_err(|_| {
        AppError::Auth("无效的用户ID".to_string())
    })
}
