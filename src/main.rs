use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use tracing::{info, warn};

use seredeli_room::{
    config::AppConfig,
    db::Database,
    routes::create_router,
    state::AppState,
    websocket::manager::WebSocketManager,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();

    info!("Starting Seredeli Room server...");

    // 加载配置
    let config = AppConfig::from_env()?;
    info!("Configuration loaded successfully");
    info!("Server will run on {}:{}", config.server.host, config.server.port);

    // 初始化数据库连接池
    let db = Database::new(&config.database).await?;

    // 运行数据库迁移
    db.migrate().await?;

    // 初始化WebSocket管理器
    let ws_manager = WebSocketManager::new();
    info!("WebSocket manager initialized");

    // 创建应用状态
    let state = AppState::new(db, ws_manager, config.jwt.clone());

    // 构建应用路由
    let app = create_router(state);

    // 绑定地址
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Failed to parse server address");

    info!("Server starting on http://{}", addr);

    // 启动HTTP服务器，支持优雅关闭
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // 创建关闭信号监听
    let shutdown_signal = create_shutdown_signal();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// 初始化日志系统
fn init_logging() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("info,seredeli_room=debug,tower_http=debug")
        });

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

/// 创建关闭信号监听
async fn create_shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C signal, starting graceful shutdown...");
        }
        _ = terminate => {
            warn!("Received terminate signal, starting graceful shutdown...");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        // 确保日志初始化不会panic
        init_logging();
    }
}
