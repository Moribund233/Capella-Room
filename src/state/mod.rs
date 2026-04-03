use std::fmt;
use std::sync::Arc;

use crate::config::{JwtConfig, UploadConfig};
use crate::db::Database;
use crate::middleware::rate_limit::RateLimiter;
use crate::services::auth_service::AuthService;
use crate::services::file_service::FileService;
use crate::services::message_service::MessageService;
use crate::services::room_service::RoomService;
use crate::services::user_service::UserService;
use crate::utils::logging::MetricsCollector;
use crate::websocket::manager::WebSocketManager;

/// 应用状态
/// 在 Axum 处理函数中通过 State 提取器访问
pub struct AppState {
    pub db: Database,
    pub ws_manager: Arc<WebSocketManager>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub room_service: RoomService,
    pub message_service: MessageService,
    pub file_service: FileService,
    pub rate_limiter: Option<Arc<RateLimiter>>,
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("db", &self.db)
            .field("ws_manager", &self.ws_manager)
            .field("metrics_collector", &"<MetricsCollector>")
            .field("auth_service", &"<AuthService>")
            .field("user_service", &"<UserService>")
            .field("room_service", &"<RoomService>")
            .field("message_service", &"<MessageService>")
            .field("file_service", &"<FileService>")
            .finish()
    }
}

impl AppState {
    /// 创建应用状态
    pub fn new(
        db: Database,
        ws_manager: Arc<WebSocketManager>,
        jwt_config: JwtConfig,
        upload_config: UploadConfig,
        metrics_collector: Arc<MetricsCollector>,
    ) -> anyhow::Result<Arc<Self>> {
        let auth_service = AuthService::new(jwt_config);
        let user_service = UserService::new(db.clone());
        let room_service = RoomService::new(db.clone());
        let message_service = MessageService::new(db.clone());
        let file_service = FileService::from_config(db.clone(), &upload_config)?;

        // 从环境变量读取是否启用速率限制
        let rate_limiter = if std::env::var("ENABLE_RATE_LIMIT")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true)
        {
            Some(Arc::new(RateLimiter::with_default_config()))
        } else {
            None
        };

        Ok(Arc::new(Self {
            db,
            ws_manager,
            metrics_collector,
            auth_service,
            user_service,
            room_service,
            message_service,
            file_service,
            rate_limiter,
        }))
    }

    /// 获取数据库连接池
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// 获取 WebSocket 管理器
    pub fn ws_manager(&self) -> &WebSocketManager {
        &self.ws_manager
    }

    /// 获取指标收集器
    pub fn metrics_collector(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    /// 获取认证服务
    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }

    /// 获取用户服务
    pub fn user_service(&self) -> &UserService {
        &self.user_service
    }

    /// 获取聊天室服务
    pub fn room_service(&self) -> &RoomService {
        &self.room_service
    }

    /// 获取消息服务
    pub fn message_service(&self) -> &MessageService {
        &self.message_service
    }

    /// 获取文件服务
    pub fn file_service(&self) -> &FileService {
        &self.file_service
    }

    /// 获取速率限制器
    pub fn rate_limiter(&self) -> Option<Arc<RateLimiter>> {
        self.rate_limiter.clone()
    }
}

// 为 Arc<AppState>实现 Clone
impl Clone for AppState {
    fn clone(&self) -> Self {
        // 获取文件服务的配置
        let upload_config = crate::config::UploadConfig {
            max_file_size: self.file_service.max_file_size(),
            base_url: self.file_service.get_base_url(),
        };

        Self {
            db: self.db.clone(),
            ws_manager: Arc::clone(&self.ws_manager),
            metrics_collector: Arc::clone(&self.metrics_collector),
            auth_service: AuthService::new(self.auth_service.jwt_config.clone()),
            user_service: UserService::new(self.db.clone()),
            room_service: RoomService::new(self.db.clone()),
            message_service: MessageService::new(self.db.clone()),
            file_service: FileService::from_config(self.db.clone(), &upload_config)
                .expect("Failed to clone file service"),
            rate_limiter: self.rate_limiter.clone(),
        }
    }
}
