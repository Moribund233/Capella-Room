use std::sync::Arc;

use crate::db::Database;
use crate::websocket::manager::WebSocketManager;

/// 应用状态
/// 在Axum处理函数中通过State提取器访问
#[derive(Debug)]
pub struct AppState {
    pub db: Database,
    pub ws_manager: Arc<WebSocketManager>,
}

impl AppState {
    /// 创建应用状态
    pub fn new(db: Database, ws_manager: Arc<WebSocketManager>) -> Arc<Self> {
        Arc::new(Self { db, ws_manager })
    }

    /// 获取数据库连接池
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// 获取WebSocket管理器
    pub fn ws_manager(&self) -> &WebSocketManager {
        &self.ws_manager
    }
}

// 为Arc<AppState>实现Clone
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            ws_manager: Arc::clone(&self.ws_manager),
        }
    }
}
