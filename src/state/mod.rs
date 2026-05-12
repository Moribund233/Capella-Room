use std::fmt;
use std::sync::Arc;

use tracing::info;

use crate::config::{start_config_listeners, AppConfig, ConfigManager};
use crate::db::Database;
use crate::redis::{
    pubsub::RedisPubSub, ConsumerGroupConfig, RedisManager, StreamConsumer, StreamManager,
};
use crate::services::account_security_service::AccountSecurityService;
use crate::services::audit_log_consumer::AuditLogConsumerHandler;
use crate::services::audit_service::AuditService;
use crate::services::auth_service::AuthService;
use crate::services::file_service::FileService;
use crate::services::ip_security_service::IpSecurityService;
use crate::services::message_service::MessageService;
use crate::services::monitor_service::MonitorService;
use crate::services::notification_service::NotificationService;
use crate::services::room_service::RoomService;
use crate::services::user_service::UserService;
use crate::services::user_settings_service::UserSettingsService;
use crate::utils::logging::{
    init_global_log_broadcaster, LogBroadcaster, MetricsCollector, StructuredLogger,
};
use crate::websocket::manager::WebSocketManager;

pub struct AppState {
    pub db: Database,
    pub ws_manager: Arc<WebSocketManager>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub log_broadcaster: Arc<LogBroadcaster>,
    pub logger: Arc<StructuredLogger>,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub room_service: RoomService,
    pub message_service: MessageService,
    pub file_service: Arc<FileService>,
    pub notification_service: Arc<NotificationService>,
    pub audit_service: Arc<AuditService>,
    pub ip_security_service: Arc<IpSecurityService>,
    pub user_settings_service: UserSettingsService,
    pub account_security_service: AccountSecurityService,
    pub config: Arc<tokio::sync::RwLock<AppConfig>>,
    pub config_manager: Arc<ConfigManager>,
    pub redis_manager: Option<Arc<RedisManager>>,
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("db", &self.db)
            .field("ws_manager", &self.ws_manager)
            .field("metrics_collector", &"<MetricsCollector>")
            .field("log_broadcaster", &"<LogBroadcaster>")
            .field("logger", &"<StructuredLogger>")
            .field("auth_service", &"<AuthService>")
            .field("user_service", &"<UserService>")
            .field("room_service", &"<RoomService>")
            .field("message_service", &"<MessageService>")
            .field("file_service", &"<FileService>")
            .field("notification_service", &"<NotificationService>")
            .field("audit_service", &"<AuditService>")
            .field("ip_security_service", &"<IpSecurityService>")
            .field("redis_manager", &self.redis_manager)
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
        redis_manager: Option<Arc<RedisManager>>,
    ) -> anyhow::Result<Arc<Self>> {
        let log_broadcaster = Arc::new(LogBroadcaster::new(1000));
        // 初始化全局日志广播器
        init_global_log_broadcaster((*log_broadcaster).clone());
        let logger = Arc::new(StructuredLogger);

        let shared_config = Arc::new(tokio::sync::RwLock::new(config));

        let auth_service = AuthService::with_shared_config(shared_config.clone());
        let user_service = UserService::new(db.clone());
        let room_service = RoomService::new(db.clone());
        let message_service = MessageService::new(db.clone());

        // 先创建用户设置服务（通知服务依赖它）
        let user_settings_service = UserSettingsService::new(db.clone().pool().clone());
        let user_settings_service_arc = Arc::new(user_settings_service);

        let notification_service = Arc::new(NotificationService::new(
            db.clone(),
            ws_manager.clone(),
            user_settings_service_arc.clone(),
        ));

        // 初始化 Stream 管理器（如果 Redis 可用）
        let stream_manager = if let Some(ref redis_mgr) = redis_manager {
            StreamManager::new(redis_mgr.clone(), 10000).await? // 最大 10000 条消息
        } else {
            None
        };

        let audit_service = Arc::new(
            AuditService::new(
                db.clone(),
                notification_service.clone(),
                config_manager.clone(),
                stream_manager.clone(),
            )
            .await,
        );

        let ip_security_service =
            Arc::new(IpSecurityService::new(db.clone(), audit_service.clone()).await);

        let file_service = Arc::new(FileService::with_shared_config(
            db.clone(),
            shared_config.clone(),
        )?);

        let account_security_service = AccountSecurityService::new(db.clone().pool().clone());

        // 如果 Redis 启用，设置 WebSocketManager 的 Redis Pub/Sub
        if let Some(ref redis_mgr) = redis_manager {
            if let Some(redis_pubsub) = RedisPubSub::new(redis_mgr.clone()).await? {
                ws_manager.set_redis_pubsub(redis_pubsub).await;
            }
        }

        // 如果 Redis 启用，启动审计日志 Stream 消费者
        if let Some(ref redis_mgr) = redis_manager {
            let consumer = Arc::new(StreamConsumer::new(
                redis_mgr.clone(),
                ConsumerGroupConfig::default(),
            ));
            let handler = Arc::new(AuditLogConsumerHandler::new(db.clone()));

            tokio::spawn(async move {
                info!("Starting audit log stream consumer...");
                consumer.start(handler, "capella:stream:audit_logs").await;
            });
        }

        let state = Arc::new(Self {
            db,
            ws_manager: ws_manager.clone(),
            metrics_collector,
            log_broadcaster,
            logger,
            auth_service,
            user_service,
            room_service,
            message_service,
            file_service,
            notification_service,
            audit_service,
            ip_security_service,
            user_settings_service: (*user_settings_service_arc).clone(),
            account_security_service,
            config: shared_config,
            config_manager: config_manager.clone(),
            redis_manager,
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

    pub fn log_broadcaster(&self) -> &LogBroadcaster {
        &self.log_broadcaster
    }

    pub fn logger(&self) -> &StructuredLogger {
        &self.logger
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

    pub fn ip_security_service(&self) -> &IpSecurityService {
        &self.ip_security_service
    }

    pub fn user_settings_service(&self) -> &UserSettingsService {
        &self.user_settings_service
    }

    pub fn account_security_service(&self) -> &AccountSecurityService {
        &self.account_security_service
    }

    pub fn monitor_service(&self) -> MonitorService {
        MonitorService::new(self.db.clone())
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
        Self {
            db: self.db.clone(),
            ws_manager: Arc::clone(&self.ws_manager),
            metrics_collector: Arc::clone(&self.metrics_collector),
            log_broadcaster: Arc::clone(&self.log_broadcaster),
            logger: Arc::clone(&self.logger),
            auth_service: AuthService::with_shared_config(self.config.clone()),
            user_service: UserService::new(self.db.clone()),
            room_service: RoomService::new(self.db.clone()),
            message_service: MessageService::new(self.db.clone()),
            file_service: Arc::clone(&self.file_service),
            notification_service: Arc::clone(&self.notification_service),
            audit_service: Arc::clone(&self.audit_service),
            ip_security_service: Arc::clone(&self.ip_security_service),
            user_settings_service: UserSettingsService::new(self.db.clone().pool().clone()),
            account_security_service: AccountSecurityService::new(self.db.clone().pool().clone()),
            config: self.config.clone(),
            config_manager: self.config_manager.clone(),
            redis_manager: self.redis_manager.clone(),
        }
    }
}
