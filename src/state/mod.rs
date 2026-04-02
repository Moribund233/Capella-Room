use std::fmt;
use std::sync::Arc;

use crate::config::JwtConfig;
use crate::db::Database;
use crate::services::auth_service::AuthService;
use crate::services::room_service::RoomService;
use crate::services::user_service::UserService;
use crate::websocket::manager::WebSocketManager;

/// 应用状态
/// 在Axum处理函数中通过State提取器访问
pub struct AppState {
    pub db: Database,
    pub ws_manager: Arc<WebSocketManager>,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub room_service: RoomService,
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("db", &self.db)
            .field("ws_manager", &self.ws_manager)
            .field("auth_service", &"<AuthService>")
            .field("user_service", &"<UserService>")
            .field("room_service", &"<RoomService>")
            .finish()
    }
}

impl AppState {
    /// 创建应用状态
    pub fn new(
        db: Database,
        ws_manager: Arc<WebSocketManager>,
        jwt_config: JwtConfig,
    ) -> Arc<Self> {
        let auth_service = AuthService::new(jwt_config);
        let user_service = UserService::new(db.clone());
        let room_service = RoomService::new(db.clone());

        Arc::new(Self {
            db,
            ws_manager,
            auth_service,
            user_service,
            room_service,
        })
    }

    /// 获取数据库连接池
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// 获取WebSocket管理器
    pub fn ws_manager(&self) -> &WebSocketManager {
        &self.ws_manager
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
}

// 为Arc<AppState>实现Clone
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            ws_manager: Arc::clone(&self.ws_manager),
            auth_service: AuthService::new(self.auth_service.jwt_config.clone()),
            user_service: UserService::new(self.db.clone()),
            room_service: RoomService::new(self.db.clone()),
        }
    }
}
