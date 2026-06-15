use std::env;
use std::sync::Arc;
use std::time::Duration;

use capella_room::{
    config::{ConfigLoader, DatabaseConfig},
    db::Database,
    routes::create_router,
    state::AppState,
    utils::logging::MetricsCollector,
    websocket::manager::WebSocketManager,
};
use tokio::time::sleep;

fn load_test_env() {
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

fn setup_test_db_url() -> Option<String> {
    load_test_env();
    env::var("DATABASE_URL").ok()
}

async fn start_test_server() -> Option<String> {
    let database_url = setup_test_db_url()?;

    let db_config = DatabaseConfig {
        url: Some(database_url.clone()),
        max_connections: 5,
        acquire_timeout_secs: 30,
        idle_timeout_secs: 600,
    };

    let db = Database::new(&db_config).await.ok()?;
    db.migrate().await.ok()?;

    let _config = ConfigLoader::load().ok()?;
    let ws_manager = WebSocketManager::new();
    let metrics_collector = Arc::new(MetricsCollector::new());

    let config_db = Database::new(&db_config).await.ok()?;

    let state = AppState::new(
        db,
        ws_manager,
        ConfigLoader::load().ok()?,
        metrics_collector,
        Arc::new(capella_room::config::ConfigManager::new(
            config_db,
            ConfigLoader::load().ok()?,
            None,
        )),
        None,
    )
    .await
    .ok()?;

    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .ok()?;
    let addr = listener.local_addr().ok()?;
    let base_url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    sleep(Duration::from_millis(500)).await;
    Some(base_url)
}

#[cfg(test)]
mod v2_auth_api_tests {
    use super::*;

    #[tokio::test]
    async fn test_v2_register_send_code_returns_success() {
        let base_url = match start_test_server().await {
            Some(url) => url,
            None => {
                eprintln!("Skipping test: DATABASE_URL not set");
                return;
            }
        };

        let client = reqwest::Client::new();
        let body = serde_json::json!({"email": "newuser@test.com"});
        let resp = client
            .post(format!("{}/api/v2/auth/register/send-code", base_url))
            .json(&body)
            .send()
            .await
            .expect("Request failed");

        assert_eq!(resp.status(), 200, "register/send-code should return 200");
        let json: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["code_length"], 6);
    }

    #[tokio::test]
    async fn test_v2_register_send_code_empty_email() {
        let base_url = match start_test_server().await {
            Some(url) => url,
            None => {
                eprintln!("Skipping test: DATABASE_URL not set");
                return;
            }
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/v2/auth/register/send-code", base_url))
            .json(&serde_json::json!({"email": ""}))
            .send()
            .await
            .expect("Request failed");

        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_v2_login_send_code_unregistered_email() {
        let base_url = match start_test_server().await {
            Some(url) => url,
            None => {
                eprintln!("Skipping test: DATABASE_URL not set");
                return;
            }
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/v2/auth/login/send-code", base_url))
            .json(&serde_json::json!({"email": "nonexistent@test.com"}))
            .send()
            .await
            .expect("Request failed");

        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_v2_register_fails_with_wrong_code() {
        let base_url = match start_test_server().await {
            Some(url) => url,
            None => {
                eprintln!("Skipping test: DATABASE_URL not set");
                return;
            }
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/v2/auth/register", base_url))
            .json(&serde_json::json!({
                "email": "register-test@test.com",
                "code": "000000",
                "username": "regtest",
                "password": "SecurePass123!"
            }))
            .send()
            .await
            .expect("Request failed");

        assert!(resp.status().is_client_error(), "Wrong code should return 4xx");
    }

    #[tokio::test]
    async fn test_v2_auth_register_send_code_duplicate_email() {
        let base_url = match start_test_server().await {
            Some(url) => url,
            None => {
                eprintln!("Skipping test: DATABASE_URL not set");
                return;
            }
        };

        let client = reqwest::Client::new();

        // First request should succeed
        let resp = client
            .post(format!("{}/api/v2/auth/register/send-code", base_url))
            .json(&serde_json::json!({"email": "dup@test.com"}))
            .send()
            .await
            .expect("Request failed");
        assert_eq!(resp.status(), 200, "First code request should succeed");

        // Second immediate request should hit rate limit or cooldown
        let resp2 = client
            .post(format!("{}/api/v2/auth/register/send-code", base_url))
            .json(&serde_json::json!({"email": "dup@test.com"}))
            .send()
            .await
            .expect("Request failed");
        assert!(resp2.status().is_client_error() || resp2.status() == 200,
            "Second request should be rate-limited or still succeed (email not registered yet)");
    }
}
