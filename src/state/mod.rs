use std::fmt;
use std::sync::Arc;

use crate::config::{start_config_listeners, AppConfig, ConfigManager};
use crate::db::Database;
use crate::services::audit_service::AuditService;
use crate::services::auth_service::AuthService;
use crate::services::file_service::FileService;
use crate::services::message_service::MessageService;
use crate::services::notification_service::NotificationService;
use crate::services::room_service::RoomService;
use crate::services::user_service::UserService;
use crate::utils::logging::MetricsCollector;
use crate::websocket::manager::WebSocketManager;

pub struct AppState {
    pub db: Database,
    pub ws_manager: Arc<WebSocketManager>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub room_service: RoomService,
    pub message_service: MessageService,
    pub file_service: FileService,
    pub notification_service: Arc<NotificationService>,
    pub audit_service: Arc<AuditService>,
    pub config: Arc<tokio::sync::RwLock<AppConfig>>,
    pub config_manager: Arc<ConfigManager>,
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
            .field("notification_service", &"<NotificationService>")
            .field("audit_service", &"<AuditService>")
            .finish_non_exhaustive()
    }
}

impl AppState {
    pub async fn new(
        db: Database,
        ws_manager: Arc<WebSocketManager>,
        config: AppConfig,
        metrics_collector: Arc<MetricsCollector>,
        config_manager: Arc<ConfigManager>,
    ) -> anyhow::Result<Arc<Self>> {
        let jwt_config = crate::config::JwtConfig {
            secret: config.jwt.secret.clone(),
            expiration_hours: config.jwt.expiration_hours,
        };

        let auth_service = AuthService::new(jwt_config);
        let user_service = UserService::new(db.clone());
        let room_service = RoomService::new(db.clone());
        let message_service = MessageService::new(db.clone());
        let notification_service =
            Arc::new(NotificationService::new(db.clone(), ws_manager.clone()));
        let audit_service = Arc::new(
            AuditService::new(
                db.clone(),
                notification_service.clone(),
                config_manager.clone(),
            )
            .await,
        );

        let upload_config = crate::config::UploadConfig {
            max_file_size: config.upload.max_file_size,
            base_url: config.upload.base_url.clone(),
        };
        let file_service = FileService::from_config(db.clone(), &upload_config)?;

        let shared_config = Arc::new(tokio::sync::RwLock::new(config));

        let state = Arc::new(Self {
            db,
            ws_manager: ws_manager.clone(),
            metrics_collector,
            auth_service,
            user_service,
            room_service,
            message_service,
            file_service,
            notification_service,
            audit_service,
            config: shared_config,
            config_manager: config_manager.clone(),
        });

        // 启动配置监听器
        start_config_listeners(config_manager, ws_manager);

        Ok(state)
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn ws_manager(&self) -> &WebSocketManager {
        &self.ws_manager
    }

    pub fn metrics_collector(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }

    pub fn user_service(&self) -> &UserService {
        &self.user_service
    }

    pub fn room_service(&self) -> &RoomService {
        &self.room_service
    }

    pub fn message_service(&self) -> &MessageService {
        &self.message_service
    }

    pub fn file_service(&self) -> &FileService {
        &self.file_service
    }

    pub fn notification_service(&self) -> &NotificationService {
        &self.notification_service
    }

    pub fn audit_service(&self) -> &AuditService {
        &self.audit_service
    }

    pub fn config(&self) -> Arc<tokio::sync::RwLock<AppConfig>> {
        self.config.clone()
    }

    pub fn config_manager(&self) -> Arc<ConfigManager> {
        self.config_manager.clone()
    }

    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        let upload_config = crate::config::UploadConfig {
            max_file_size: self.file_service.max_file_size(),
            base_url: self.file_service.get_base_url(),
        };

        let jwt_config = crate::config::JwtConfig {
            secret: self.auth_service.jwt_config.secret.clone(),
            expiration_hours: self.auth_service.jwt_config.expiration_hours,
        };

        Self {
            db: self.db.clone(),
            ws_manager: Arc::clone(&self.ws_manager),
            metrics_collector: Arc::clone(&self.metrics_collector),
            auth_service: AuthService::new(jwt_config),
            user_service: UserService::new(self.db.clone()),
            room_service: RoomService::new(self.db.clone()),
            message_service: MessageService::new(self.db.clone()),
            file_service: FileService::from_config(self.db.clone(), &upload_config)
                .expect("Failed to clone file service"),
            notification_service: Arc::clone(&self.notification_service),
            audit_service: Arc::clone(&self.audit_service),
            config: self.config.clone(),
            config_manager: self.config_manager.clone(),
        }
    }
}
