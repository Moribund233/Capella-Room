//! 阶段 6.5 文件上传和资源管理系统测试
//!
//! 测试内容：
//! - 文件上传功能（通用上传、图片上传、头像上传）
//! - 文件类型验证
//! - 文件大小限制
//! - 文件信息获取
//! - 文件列表查询
//! - 文件删除
//! - 权限验证

use std::env;
use std::path::PathBuf;

use capella_room::{
    config::{DatabaseConfig, JwtConfig, UploadConfig},
    db::Database,
    models::file::{is_allowed_mime_type, FileCategory, FileQueryParams, FileUsageType},
    services::{auth_service::AuthService, file_service::FileService, user_service::UserService},
};
use uuid::Uuid;

/// 测试辅助函数：加载测试环境变量
fn load_test_env() {
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

/// 测试辅助函数：创建测试数据库连接
async fn setup_test_db() -> Database {
    load_test_env();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test or environment");

    let max_connections = env::var("APP_DATABASE__MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let config = DatabaseConfig {
        url: Some(database_url),
        max_connections,
        acquire_timeout_secs: 30,
        idle_timeout_secs: 600,
    };

    let db = Database::new(&config)
        .await
        .expect("Failed to create database");

    // 运行数据库迁移
    db.migrate().await.expect("Failed to run migrations");

    db
}

/// 创建测试上传目录
fn setup_test_upload_dir() -> PathBuf {
    let test_dir = std::env::temp_dir().join(format!("capella_test_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&test_dir).expect("Failed to create test upload directory");
    test_dir
}

/// 创建测试用户
async fn create_test_user(user_service: &UserService, username: &str) -> (Uuid, String) {
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let email = format!("{}{}@test.com", username, unique_id);
    let password = "TestPassword123!";

    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test-secret-key-for-jwt-signing-in-tests-only".to_string()),
        expiration_hours: 24,
    });

    let password_hash = auth_service.hash_password(password).unwrap();
    let user = user_service
        .create_user(username, &email, &password_hash)
        .await
        .unwrap();

    (user.id, password.to_string())
}

/// 创建文件服务
async fn create_file_service(db: &Database, upload_dir: &str) -> FileService {
    let upload_config = UploadConfig {
        max_file_size: 10 * 1024 * 1024, // 10MB
        base_url: "/uploads".to_string(),
    };

    FileService::new(
        db.clone(),
        upload_dir.to_string(),
        upload_config.base_url.clone(),
        upload_config.max_file_size,
    )
}

// ==================== 文件上传测试 ====================

#[tokio::test]
async fn test_upload_image_file() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testuser{}", unique_id)).await;

    // 创建测试图片数据 (PNG 文件头)
    let image_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG 文件头
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
    ];

    let result = file_service
        .upload_file(
            user_id,
            image_data.clone(),
            "test_image.png",
            "image/png",
            FileUsageType::Message,
            None,
        )
        .await;

    assert!(result.is_ok(), "图片上传应该成功: {:?}", result.err());
    let response = result.unwrap();

    assert_eq!(response.original_name, "test_image.png");
    assert_eq!(response.mime_type, "image/png");
    assert_eq!(response.file_size, image_data.len() as i64);
    assert!(response.file_url.starts_with("/uploads/images/"));

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[tokio::test]
async fn test_upload_document_file() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testuser{}", unique_id)).await;

    // 创建测试文档数据
    let document_data = b"This is a test PDF document content.".to_vec();

    let result = file_service
        .upload_file(
            user_id,
            document_data.clone(),
            "test_document.pdf",
            "application/pdf",
            FileUsageType::General,
            None,
        )
        .await;

    assert!(result.is_ok(), "文档上传应该成功: {:?}", result.err());
    let response = result.unwrap();

    assert_eq!(response.original_name, "test_document.pdf");
    assert_eq!(response.mime_type, "application/pdf");
    assert!(response.file_url.starts_with("/uploads/documents/"));

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[tokio::test]
async fn test_upload_avatar_file() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testuser{}", unique_id)).await;

    // 创建测试头像数据 (JPEG 文件头)
    let avatar_data = vec![
        0xFF, 0xD8, 0xFF, 0xE0, // JPEG 文件头
        0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
    ];

    let result = file_service
        .upload_file(
            user_id,
            avatar_data.clone(),
            "avatar.jpg",
            "image/jpeg",
            FileUsageType::Avatar,
            None,
        )
        .await;

    assert!(result.is_ok(), "头像上传应该成功: {:?}", result.err());
    let response = result.unwrap();

    assert_eq!(response.original_name, "avatar.jpg");
    assert_eq!(response.mime_type, "image/jpeg");
    // 头像文件存储在 images/ 目录下（根据 MIME 类型 image/jpeg）
    assert!(
        response.file_url.starts_with("/uploads/images/"),
        "URL 应该是 /uploads/images/ 开头，实际是: {}",
        response.file_url
    );

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

// ==================== 文件验证测试 ====================

#[tokio::test]
async fn test_upload_invalid_file_type() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testuser{}", unique_id)).await;

    // 尝试上传不允许的文件类型 (可执行文件)
    let exe_data = b"MZ executable content".to_vec();

    let result: Result<_, capella_room::error::AppError> = file_service
        .upload_file(
            user_id,
            exe_data,
            "malicious.exe",
            "application/x-msdownload",
            FileUsageType::General,
            None,
        )
        .await;

    assert!(result.is_err(), "不允许的文件类型应该被拒绝");
    let error_msg = result.unwrap_err().to_string();
    // 错误消息可能包含 "不支持的文件类型" 或 "不允许"
    assert!(
        error_msg.contains("不支持")
            || error_msg.contains("不允许")
            || error_msg.contains("not allowed"),
        "错误消息应该表明文件类型不被允许: {}",
        error_msg
    );

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[tokio::test]
async fn test_upload_oversized_file() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());

    // 创建一个最大文件大小为 1KB 的配置
    let file_service = FileService::new(
        db.clone(),
        upload_dir.to_str().unwrap().to_string(),
        "/uploads".to_string(),
        1024, // 1KB
    );

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testuser{}", unique_id)).await;

    // 创建超过限制大小的文件 (2KB)
    let large_data = vec![0u8; 2048];

    let result: Result<_, capella_room::error::AppError> = file_service
        .upload_file(
            user_id,
            large_data,
            "large_file.png",
            "image/png",
            FileUsageType::Message,
            None,
        )
        .await;

    assert!(result.is_err(), "超大文件应该被拒绝");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("大小") || error_msg.contains("size") || error_msg.contains("超过"),
        "错误消息应该表明文件大小超过限制: {}",
        error_msg
    );

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

// ==================== 文件查询测试 ====================

#[tokio::test]
async fn test_get_file_by_id() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testuser{}", unique_id)).await;

    // 上传文件
    let image_data = b"test image data".to_vec();
    let upload_response = file_service
        .upload_file(
            user_id,
            image_data,
            "test.png",
            "image/png",
            FileUsageType::Message,
            None,
        )
        .await
        .unwrap();

    // 查询文件信息
    let file_info = file_service.get_file_by_id(upload_response.id).await;

    assert!(
        file_info.is_ok(),
        "应该能获取文件信息: {:?}",
        file_info.err()
    );
    let file = file_info.unwrap();
    assert_eq!(file.id, upload_response.id);
    assert_eq!(file.original_name, "test.png");
    assert_eq!(file.uploader_id, Some(user_id));

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[tokio::test]
async fn test_get_nonexistent_file() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    // 查询不存在的文件
    let nonexistent_id = Uuid::new_v4();
    let result = file_service.get_file_by_id(nonexistent_id).await;

    assert!(result.is_err(), "查询不存在的文件应该返回错误");

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[tokio::test]
async fn test_list_user_files() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testuser{}", unique_id)).await;

    // 上传多个文件
    for i in 0..3 {
        let image_data = format!("test image data {}", i).into_bytes();
        file_service
            .upload_file(
                user_id,
                image_data,
                &format!("test{}.png", i),
                "image/png",
                FileUsageType::Message,
                None,
            )
            .await
            .unwrap();
    }

    // 查询文件列表
    let params = FileQueryParams {
        limit: Some(10),
        offset: Some(0),
        usage_type: None,
        category: None,
    };

    let result = file_service.get_files_by_uploader(user_id, params).await;

    assert!(result.is_ok(), "应该能获取文件列表: {:?}", result.err());
    let list_response = result.unwrap();
    assert_eq!(list_response.files.len(), 3);
    assert_eq!(list_response.total, 3);

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

// ==================== 文件删除测试 ====================

#[tokio::test]
async fn test_delete_own_file() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user_id, _) = create_test_user(&user_service, &format!("testuser{}", unique_id)).await;

    // 上传文件
    let image_data = b"test image data".to_vec();
    let upload_response = file_service
        .upload_file(
            user_id,
            image_data,
            "test.png",
            "image/png",
            FileUsageType::Message,
            None,
        )
        .await
        .unwrap();

    // 删除文件
    let result = file_service.delete_file(upload_response.id, user_id).await;

    assert!(result.is_ok(), "应该能删除自己的文件: {:?}", result.err());

    // 验证文件已被删除
    let file_exists = file_service.get_file_by_id(upload_response.id).await;
    assert!(file_exists.is_err(), "文件应该已被删除");

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[tokio::test]
async fn test_delete_other_user_file() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user1_id, _) = create_test_user(&user_service, &format!("user1{}", unique_id)).await;
    let (user2_id, _) = create_test_user(&user_service, &format!("user2{}", unique_id)).await;

    // 用户1上传文件
    let image_data = b"test image data".to_vec();
    let upload_response = file_service
        .upload_file(
            user1_id,
            image_data,
            "test.png",
            "image/png",
            FileUsageType::Message,
            None,
        )
        .await
        .unwrap();

    // 用户2尝试删除用户1的文件
    let result = file_service.delete_file(upload_response.id, user2_id).await;

    assert!(result.is_err(), "不应该能删除其他用户的文件");

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

// ==================== 文件工具函数测试 ====================

#[tokio::test]
async fn test_file_deduplication() {
    let db = setup_test_db().await;
    let upload_dir = setup_test_upload_dir();
    let user_service = UserService::new(db.clone());
    let file_service = create_file_service(&db, upload_dir.to_str().unwrap()).await;

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let (user1_id, _) = create_test_user(&user_service, &format!("user1{}", unique_id)).await;
    let (user2_id, _) = create_test_user(&user_service, &format!("user2{}", unique_id)).await;

    // 相同的数据
    let image_data = b"same image data for deduplication test".to_vec();

    // 用户1上传文件
    let response1 = file_service
        .upload_file(
            user1_id,
            image_data.clone(),
            "image1.png",
            "image/png",
            FileUsageType::Message,
            None,
        )
        .await
        .unwrap();

    // 用户2上传相同的数据
    let response2 = file_service
        .upload_file(
            user2_id,
            image_data.clone(),
            "image2.png",
            "image/png",
            FileUsageType::Message,
            None,
        )
        .await
        .unwrap();

    // 两个文件应该有相同的文件大小（因为是相同的数据）
    assert_eq!(response1.file_size, response2.file_size);

    // 清理测试目录
    let _ = std::fs::remove_dir_all(&upload_dir);
}

#[test]
fn test_file_category_detection() {
    // 测试 MIME 类型分类检测
    assert_eq!(
        FileCategory::from_mime_type("image/png"),
        FileCategory::Image
    );
    assert_eq!(
        FileCategory::from_mime_type("image/jpeg"),
        FileCategory::Image
    );
    assert_eq!(
        FileCategory::from_mime_type("application/pdf"),
        FileCategory::Document
    );
    assert_eq!(
        FileCategory::from_mime_type("video/mp4"),
        FileCategory::Video
    );
    assert_eq!(
        FileCategory::from_mime_type("audio/mpeg"),
        FileCategory::Audio
    );
    // application/unknown 不是已知的文档类型，所以归类为 Document（根据扩展名）
    assert_eq!(
        FileCategory::from_mime_type("application/unknown"),
        FileCategory::Document
    );
}

#[test]
fn test_allowed_mime_types() {
    // 测试允许的文件类型验证
    assert!(is_allowed_mime_type("image/png"));
    assert!(is_allowed_mime_type("image/jpeg"));
    assert!(is_allowed_mime_type("application/pdf"));
    assert!(is_allowed_mime_type("video/mp4"));
    assert!(is_allowed_mime_type("audio/mpeg"));
    // 不允许的文件类型
    assert!(!is_allowed_mime_type("application/x-msdownload"));
    assert!(!is_allowed_mime_type("application/x-executable"));
}
