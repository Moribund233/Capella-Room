use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tracing::{info, warn};

use seredeli_room::{
    config::{ConfigLoader, ConfigManager},
    db::Database,
    routes::create_router,
    state::AppState,
    utils::logging::MetricsCollector,
    websocket::manager::WebSocketManager,
};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    info!("Starting Seredeli Room server...");

    let config = ConfigLoader::load()?;
    info!("Configuration loaded successfully");
    info!(
        "Server will run on {}:{}",
        config.server.host, config.server.port
    );
    info!("Environment: {}", config.app.env);

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

    let ws_manager = WebSocketManager::new();
    info!("WebSocket manager initialized");

    let metrics_collector = Arc::new(MetricsCollector::new());
    info!("Metrics collector initialized");

    let config_manager = ConfigManager::new(db.clone(), config.clone());

    config_manager.initialize_default_configs().await?;

    config_manager.reload_from_database().await?;

    let config = config_manager.get_config().await;

    let state = AppState::new(db, ws_manager, config, Arc::clone(&metrics_collector))?;

    let app = create_router(state);

    let addr: SocketAddr = format!(
        "{}:{}",
        config_manager.get_config().await.server.host,
        config_manager.get_config().await.server.port
    )
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

fn init_logging() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
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
        init_logging();
    }
}
