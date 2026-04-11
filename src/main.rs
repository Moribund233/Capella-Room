use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tracing::{info, warn};

use seredeli_room::{
    config::{ConfigLoader, ConfigManager},
    db::Database,
    redis::{ConfigSyncManager, RedisManager},
    routes::create_router,
    state::AppState,
    utils::logging::{init_logging, MetricsCollector},
    websocket::manager::WebSocketManager,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 先加载配置以确定是否在维护模式
    let config = ConfigLoader::load()?;

    // 初始化日志系统（维护模式下打印更详细的日志）
    init_logging(config.system.maintenance_mode);

    info!("Starting Seredeli Room server...");
    info!("Configuration loaded successfully");
    info!(
        "Server will run on {}:{}",
        config.server.host, config.server.port
    );
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    info!("Environment: {}", env);

    let db_url = config
        .database
        .url
        .clone()
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL is required"))?;

    let mut db_config = config.database.clone();
    db_config.url = Some(db_url);

    let db = Database::new(&db_config).await?;

    db.migrate().await?;

    let ws_manager = WebSocketManager::from_config(
        config.websocket.message_buffer_size,
        config.websocket.heartbeat_interval_secs,
        config.websocket.heartbeat_timeout_secs,
    );
    info!("WebSocket manager initialized with config: buffer_size={}, heartbeat_interval={}s, heartbeat_timeout={}s",
        config.websocket.message_buffer_size,
        config.websocket.heartbeat_interval_secs,
        config.websocket.heartbeat_timeout_secs
    );

    let metrics_collector = Arc::new(MetricsCollector::new());
    info!("Metrics collector initialized");

    // 初始化 Redis 连接（用于配置同步）
    let redis_manager = RedisManager::new(config.redis.clone()).await.ok().flatten();
    let config_sync_manager = if let Some(ref redis_mgr) = redis_manager {
        ConfigSyncManager::new(redis_mgr.clone())
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    let config_manager = ConfigManager::new(db.clone(), config.clone(), config_sync_manager);

    config_manager.initialize_default_configs().await?;

    config_manager.reload_from_database().await?;

    let config = config_manager.get_config().await;

    let shared_config_manager = Arc::new(config_manager);

    let state = AppState::new(
        db.clone(),
        ws_manager,
        config.clone(),
        Arc::clone(&metrics_collector),
        Arc::clone(&shared_config_manager),
    )
    .await?;

    initialize_super_admin(&state, &config.admin.initial).await?;

    let app = create_router(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Failed to parse server address");

    info!("Server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let shutdown_signal = create_shutdown_signal();

    let metrics_clone = Arc::clone(&metrics_collector);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            metrics_clone.log_periodic_metrics();
        }
    });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal)
    .await?;

    info!("Server shutdown complete");
    Ok(())
}

async fn initialize_super_admin(
    state: &AppState,
    admin_config: &seredeli_room::config::InitialAdminConfig,
) -> Result<()> {
    if !admin_config.enabled {
        info!("Initial super admin creation is disabled");
        return Ok(());
    }

    let has_super_admin = state.user_service().has_super_admin().await?;
    if has_super_admin {
        info!("Super admin already exists, skipping initialization");
        return Ok(());
    }

    let password = std::env::var("ADMIN_INITIAL_PASSWORD").or_else(|_| {
        if !admin_config.password.is_empty() {
            Ok(admin_config.password.clone())
        } else {
            Err(anyhow::anyhow!(
                "ADMIN_INITIAL_PASSWORD not set and no default password in config"
            ))
        }
    })?;

    if password.len() < 8 {
        return Err(anyhow::anyhow!(
            "Initial admin password must be at least 8 characters"
        ));
    }

    info!("Creating initial super admin: {}", admin_config.username);

    let password_hash = state.auth_service().hash_password(&password)?;

    let user = state
        .user_service()
        .create_super_admin(&admin_config.username, &admin_config.email, &password_hash)
        .await?;

    info!(
        "Super admin created successfully: {} ({})",
        user.username, user.email
    );
    warn!("⚠️  Please change the initial admin password immediately after first login!");

    Ok(())
}

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
        init_logging(false);
    }
}
