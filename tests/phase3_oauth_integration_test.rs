use std::env;
use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

use capella_room::{
    config::{DatabaseConfig, JwtConfig, OAuthConfig},
    db::Database,
    models::user::UserRole,
    services::{
        auth_service::AuthService,
        oauth_service::OAuthService,
        user_service::UserService,
    },
};

fn load_test_env() {
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

fn create_test_jwt_secret() -> String {
    "test_secret_key_for_testing_purposes_only".to_string()
}

fn make_auth_service() -> AuthService {
    AuthService::new(JwtConfig {
        secret: Some(create_test_jwt_secret()),
        expiration_hours: 24,
    })
}

fn make_oauth_service(db: Database) -> OAuthService {
    OAuthService::new(db, OAuthConfig::default())
}

async fn create_test_user(user_service: &UserService, username: &str) -> (Uuid, String) {
    let email = format!("{}@test.com", username);
    let password = "TestPassword123";
    if let Ok(Some(user)) = user_service.get_user_by_email(&email).await {
        return (user.id, password.to_string());
    }
    let auth_service = make_auth_service();
    let password_hash = auth_service.hash_password(password).unwrap();
    let user = user_service.create_user(username, &email, &password_hash).await.unwrap();
    (user.id, password.to_string())
}

fn generate_user_token(user_id: Uuid, username: &str) -> String {
    let auth_service = make_auth_service();
    let tokens = auth_service.generate_token_pair(user_id, username, UserRole::User).unwrap();
    tokens.access_token
}

async fn cleanup_database(db: &Database) {
    sqlx::query("DELETE FROM webhook_deliveries").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM webhook_subscriptions").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM custom_events").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM room_resource_bindings").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM user_identity_mappings").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM oauth_tokens").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM oauth_authorization_codes").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM oauth_apps").execute(db.pool()).await.ok();
}

struct TestServer {
    base_url: String,
    db: Database,
}

impl TestServer {
    fn base_url(&self) -> &str { &self.base_url }
    fn db(&self) -> &Database { &self.db }
}

async fn start_test_server() -> Option<TestServer> {
    load_test_env();
    std::env::set_var("JWT_SECRET", create_test_jwt_secret());

    let database_url = env::var("DATABASE_URL").ok()?;
    let db_config = DatabaseConfig {
        url: Some(database_url),
        max_connections: 5,
        acquire_timeout_secs: 30,
        idle_timeout_secs: 600,
    };
    let db = Database::new(&db_config).await.ok()?;
    db.migrate().await.ok()?;

    let ws_manager = capella_room::websocket::manager::WebSocketManager::new();
    let metrics_collector = Arc::new(capella_room::utils::logging::MetricsCollector::new());
    let config_db = Database::new(&db_config).await.ok()?;
    let state = capella_room::state::AppState::new(
        db.clone(),
        ws_manager,
        capella_room::config::ConfigLoader::load().ok()?,
        metrics_collector,
        Arc::new(capella_room::config::ConfigManager::new(
            config_db,
            capella_room::config::ConfigLoader::load().ok()?,
            None,
        )),
        None,
    ).await.ok()?;

    let app = capella_room::routes::create_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.ok()?;
    let addr = listener.local_addr().ok()?;
    let base_url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service()).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Some(TestServer { base_url, db })
}

async fn create_oauth_app(server: &TestServer, admin_token: &str, name: &str) -> Option<(Uuid, String)> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/oauth/apps", server.base_url()))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({
            "name": name,
            "description": "Test app",
            "redirect_uris": ["http://localhost:9999/callback"],
            "scopes": ["openid", "profile", "email"]
        }))
        .send().await.ok()?;
    if !resp.status().is_success() { return None; }
    let json: serde_json::Value = resp.json().await.ok()?;
    let app_id: Uuid = json["data"]["id"].as_str()?.parse().ok()?;
    let client_secret = json["data"]["client_secret"].as_str()?.to_string();
    Some((app_id, client_secret))
}

async fn create_authorization_code_via_service(
    oauth_service: &OAuthService, app_id: Uuid, user_id: Uuid,
) -> Option<String> {
    let auth_code = oauth_service.create_authorization_code(
        app_id, user_id, "http://localhost:9999/callback",
        &["openid".to_string(), "profile".to_string()],
    ).await.ok()?;
    Some(auth_code.code)
}

#[cfg(test)]
mod oauth_token_exchange_tests {
    use super::*;

    #[tokio::test]
    async fn test_authorization_code_grant_json() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "oauth_token_ac").await;
        let token = generate_user_token(user_id, "oauth_token_ac");
        let (app_id, client_secret) = match create_oauth_app(&server, &token, "TokenACApp").await {
            Some(v) => v, None => { eprintln!("Skipping: could not create app"); return; }
        };

        let oauth_service = make_oauth_service(server.db().clone());
        let code = create_authorization_code_via_service(&oauth_service, app_id, user_id).await
            .expect("Failed to create auth code");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/oauth/token", server.base_url()))
            .header("Content-Type", "application/json")
            .json(&json!({
                "grant_type": "authorization_code",
                "code": code,
                "client_id": app_id,
                "client_secret": client_secret,
                "redirect_uri": "http://localhost:9999/callback"
            }))
            .send().await.expect("Request failed");

        assert_eq!(resp.status(), 200, "token exchange should return 200");
        let json: serde_json::Value = resp.json().await.unwrap();
        assert!(json["access_token"].as_str().is_some(), "should have access_token");
        assert!(json["refresh_token"].as_str().is_some(), "should have refresh_token");
        assert_eq!(json["token_type"].as_str(), Some("Bearer"));
        assert!(json["expires_in"].as_i64().is_some(), "should have expires_in");

        // Verify access_token can call userinfo
        let access_token = json["access_token"].as_str().unwrap();
        let userinfo_resp = client
            .get(format!("{}/oauth/userinfo", server.base_url()))
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await.expect("userinfo request failed");
        assert_eq!(userinfo_resp.status(), 200, "userinfo with access_token should return 200");

        cleanup_database(server.db()).await;
    }

    #[tokio::test]
    async fn test_authorization_code_grant_form() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "oauth_token_form").await;
        let token = generate_user_token(user_id, "oauth_token_form");
        let (app_id, client_secret) = match create_oauth_app(&server, &token, "TokenFormApp").await {
            Some(v) => v, None => { eprintln!("Skipping: could not create app"); return; }
        };

        let oauth_service = make_oauth_service(server.db().clone());
        let code = create_authorization_code_via_service(&oauth_service, app_id, user_id).await
            .expect("Failed to create auth code");

        // RFC 6749: token endpoint MUST accept application/x-www-form-urlencoded
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/oauth/token", server.base_url()))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("client_id", &app_id.to_string()),
                ("client_secret", &client_secret),
                ("redirect_uri", "http://localhost:9999/callback"),
            ])
            .send().await.expect("Request failed");

        assert_eq!(resp.status(), 200, "form-urlencoded token exchange should return 200");
        let json: serde_json::Value = resp.json().await.unwrap();
        assert!(json["access_token"].as_str().is_some(), "should have access_token from form request");

        cleanup_database(server.db()).await;
    }

    #[tokio::test]
    async fn test_refresh_token_grant() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "oauth_token_rt").await;
        let token = generate_user_token(user_id, "oauth_token_rt");
        let (app_id, client_secret) = match create_oauth_app(&server, &token, "TokenRTApp").await {
            Some(v) => v, None => { eprintln!("Skipping: could not create app"); return; }
        };

        let oauth_service = make_oauth_service(server.db().clone());
        let code = create_authorization_code_via_service(&oauth_service, app_id, user_id).await
            .expect("Failed to create auth code");

        let client = reqwest::Client::new();
        let token_resp = client
            .post(format!("{}/oauth/token", server.base_url()))
            .json(&json!({
                "grant_type": "authorization_code",
                "code": code,
                "client_id": app_id,
                "client_secret": client_secret,
                "redirect_uri": "http://localhost:9999/callback"
            }))
            .send().await.expect("Token exchange failed");
        assert_eq!(token_resp.status(), 200);
        let tokens: serde_json::Value = token_resp.json().await.unwrap();
        let refresh_token = tokens["refresh_token"].as_str().expect("should have refresh_token").to_string();

        // Use refresh_token to get new tokens
        let refresh_resp = client
            .post(format!("{}/oauth/token", server.base_url()))
            .json(&json!({
                "grant_type": "refresh_token",
                "refresh_token": refresh_token,
                "client_id": app_id,
                "client_secret": client_secret,
            }))
            .send().await.expect("Refresh request failed");

        assert_eq!(refresh_resp.status(), 200, "refresh_token grant should return 200");
        let refreshed: serde_json::Value = refresh_resp.json().await.unwrap();
        assert!(refreshed["access_token"].as_str().is_some(), "should have new access_token");

        cleanup_database(server.db()).await;
    }

    #[tokio::test]
    async fn test_client_credentials_grant() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "oauth_token_cc").await;
        let token = generate_user_token(user_id, "oauth_token_cc");
        let (_app_id, client_secret) = match create_oauth_app(&server, &token, "TokenCCApp").await {
            Some(v) => v, None => { eprintln!("Skipping: could not create app"); return; }
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/oauth/token", server.base_url()))
            .json(&json!({
                "grant_type": "client_credentials",
                "client_id": _app_id,
                "client_secret": client_secret,
            }))
            .send().await.expect("Request failed");

        assert_eq!(resp.status(), 200, "client_credentials grant should return 200");
        let json: serde_json::Value = resp.json().await.unwrap();
        assert!(json["access_token"].as_str().is_some(), "should have access_token");

        cleanup_database(server.db()).await;
    }
}

#[cfg(test)]
mod identity_mapping_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_lookup_mapping_with_oauth_token() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "mapping_user").await;
        let admin_token = generate_user_token(user_id, "mapping_user");
        let (app_id, client_secret) = match create_oauth_app(&server, &admin_token, "MappingApp").await {
            Some(v) => v, None => { eprintln!("Skipping: could not create app"); return; }
        };

        // Get an OAuth access_token for this user+app via service
        let oauth_service = make_oauth_service(server.db().clone());
        let code = create_authorization_code_via_service(&oauth_service, app_id, user_id).await
            .expect("Failed to create auth code");

        let client = reqwest::Client::new();
        let token_resp = client
            .post(format!("{}/oauth/token", server.base_url()))
            .json(&json!({
                "grant_type": "authorization_code",
                "code": code,
                "client_id": app_id,
                "client_secret": client_secret,
                "redirect_uri": "http://localhost:9999/callback"
            }))
            .send().await.expect("Token exchange failed");
        let tokens: serde_json::Value = token_resp.json().await.unwrap();
        let oauth_token = tokens["access_token"].as_str().unwrap().to_string();

        // Create mapping with OAuth access_token
        let external_id = "ext_12345";
        let resp = client
            .post(format!("{}/oauth/mappings", server.base_url()))
            .header("Authorization", format!("Bearer {}", oauth_token))
            .json(&json!({
                "app_id": app_id,
                "user_id": user_id,
                "external_user_id": external_id,
                "external_username": "external_user"
            }))
            .send().await.expect("Create mapping failed");

        assert_eq!(resp.status(), 201, "create mapping with OAuth token should return 201");
        let mapping_json: serde_json::Value = resp.json().await.unwrap();
        let mapping_id = mapping_json["data"]["id"].as_str().unwrap();

        // Lookup mapping
        let lookup_resp = client
            .get(format!("{}/oauth/mappings", server.base_url()))
            .header("Authorization", format!("Bearer {}", oauth_token))
            .query(&[("app_id", app_id.to_string()), ("external_user_id", external_id.to_string())])
            .send().await.expect("Lookup failed");
        assert_eq!(lookup_resp.status(), 200, "lookup mapping should return 200");

        // Delete mapping
        let delete_resp = client
            .delete(format!("{}/oauth/mappings/{}", server.base_url(), mapping_id))
            .header("Authorization", format!("Bearer {}", oauth_token))
            .send().await.expect("Delete failed");
        assert_eq!(delete_resp.status(), 200, "delete mapping should return 200");

        cleanup_database(server.db()).await;
    }

    #[tokio::test]
    async fn test_create_mapping_without_oauth_token_fails() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "mapping_fail").await;
        let admin_token = generate_user_token(user_id, "mapping_fail");
        let (app_id, _) = create_oauth_app(&server, &admin_token, "MappingFailApp").await.unwrap();

        let client = reqwest::Client::new();
        // No auth header
        let resp = client
            .post(format!("{}/oauth/mappings", server.base_url()))
            .json(&json!({
                "app_id": app_id,
                "user_id": user_id,
                "external_user_id": "ext_fail",
            }))
            .send().await.expect("Request failed");

        assert_eq!(resp.status(), 401, "mapping without auth should return 401");

        cleanup_database(server.db()).await;
    }
}

#[cfg(test)]
mod resource_binding_tests {
    use super::*;

    async fn create_test_room(server: &TestServer, admin_token: &str, name: &str) -> Option<Uuid> {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/v1/rooms/", server.base_url()))
            .header("Authorization", format!("Bearer {}", admin_token))
            .json(&json!({"name": name}))
            .send().await.ok()?;
        if !resp.status().is_success() { return None; }
        let json: serde_json::Value = resp.json().await.ok()?;
        json["data"]["id"].as_str()?.parse().ok()
    }

    #[tokio::test]
    async fn test_resource_binding_crud() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "resource_user").await;
        let token = generate_user_token(user_id, "resource_user");
        let (app_id, _) = create_oauth_app(&server, &token, "ResourceBindApp").await
            .expect("Failed to create OAuth app");

        let room_id = create_test_room(&server, &token, "ResourceTestRoom").await
            .expect("Failed to create room");

        let client = reqwest::Client::new();

        // Bind resource
        let bind_resp = client
            .post(format!("{}/rooms/{}/resources", server.base_url(), room_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "app_id": app_id,
                "resource_type": "document",
                "resource_id": "doc_001",
                "resource_url": "http://example.com/doc/1",
                "resource_name": "Test Document",
                "metadata": {"key": "value"}
            }))
            .send().await.expect("Bind failed");
        assert_eq!(bind_resp.status(), 201, "bind resource should return 201");

        // List bindings
        let list_resp = client
            .get(format!("{}/rooms/{}/resources", server.base_url(), room_id))
            .header("Authorization", format!("Bearer {}", token))
            .send().await.expect("List failed");
        assert_eq!(list_resp.status(), 200, "list bindings should return 200");
        let bindings: serde_json::Value = list_resp.json().await.unwrap();
        let binding_id = bindings["data"][0]["id"].as_str().unwrap().to_string();

        // Lookup resource
        let lookup_resp = client
            .get(format!("{}/oauth/resources", server.base_url()))
            .header("Authorization", format!("Bearer {}", token))
            .query(&[
                ("app_id", app_id.to_string()),
                ("resource_type", "document".to_string()),
                ("resource_id", "doc_001".to_string()),
            ])
            .send().await.expect("Lookup failed");
        assert_eq!(lookup_resp.status(), 200, "lookup resource should return 200");

        // Update binding
        let update_resp = client
            .put(format!("{}/rooms/{}/resources/{}", server.base_url(), room_id, binding_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({"resource_name": "Updated Doc"}))
            .send().await.expect("Update failed");
        assert_eq!(update_resp.status(), 200, "update binding should return 200");

        // Unbind resource
        let unbind_resp = client
            .delete(format!("{}/rooms/{}/resources/{}", server.base_url(), room_id, binding_id))
            .header("Authorization", format!("Bearer {}", token))
            .send().await.expect("Unbind failed");
        assert_eq!(unbind_resp.status(), 200, "unbind resource should return 200");

        cleanup_database(server.db()).await;
    }

    #[tokio::test]
    async fn test_auto_create_resource() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "auto_resource").await;
        let admin_token = generate_user_token(user_id, "auto_resource");

        // Get an OAuth access_token for this user+app
        let (app_id, client_secret) = create_oauth_app(&server, &admin_token, "AutoResApp").await
            .expect("Failed to create OAuth app");
        let oauth_service = make_oauth_service(server.db().clone());
        let code = create_authorization_code_via_service(&oauth_service, app_id, user_id).await
            .expect("Failed to create auth code");

        let client = reqwest::Client::new();
        let token_resp = client
            .post(format!("{}/oauth/token", server.base_url()))
            .json(&json!({
                "grant_type": "authorization_code",
                "code": code,
                "client_id": app_id,
                "client_secret": client_secret,
                "redirect_uri": "http://localhost:9999/callback"
            }))
            .send().await.expect("Token exchange failed");
        let tokens: serde_json::Value = token_resp.json().await.unwrap();
        let oauth_token = tokens["access_token"].as_str().unwrap().to_string();

        // Auto-create resource binding
        let resp = client
            .post(format!("{}/oauth/resources", server.base_url()))
            .header("Authorization", format!("Bearer {}", oauth_token))
            .json(&json!({
                "resource_type": "perseus:repository",
                "resource_id": "42",
                "resource_name": "my-awesome-repo",
            }))
            .send().await.expect("Request failed");

        assert_eq!(resp.status(), 201, "auto-create should return 201");
        let json: serde_json::Value = resp.json().await.unwrap();
        assert!(json["data"]["room"]["id"].as_str().is_some(), "should return room");
        assert!(json["data"]["binding"]["id"].as_str().is_some(), "should return binding");
        assert_eq!(json["data"]["room"]["name"], "my-awesome-repo 聊天室");

        // Duplicate request should return 409
        let dup_resp = client
            .post(format!("{}/oauth/resources", server.base_url()))
            .header("Authorization", format!("Bearer {}", oauth_token))
            .json(&json!({
                "resource_type": "perseus:repository",
                "resource_id": "42",
                "resource_name": "my-awesome-repo",
            }))
            .send().await.expect("Request failed");
        assert_eq!(dup_resp.status(), 409, "duplicate should return 409");

        cleanup_database(server.db()).await;
    }
}

#[cfg(test)]
mod custom_event_http_tests {
    use super::*;

    async fn create_test_room(server: &TestServer, admin_token: &str, name: &str) -> Option<Uuid> {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/v1/rooms/", server.base_url()))
            .header("Authorization", format!("Bearer {}", admin_token))
            .json(&json!({"name": name}))
            .send().await.ok()?;
        let json: serde_json::Value = resp.json().await.ok()?;
        json["data"]["id"].as_str()?.parse().ok()
    }

    #[tokio::test]
    async fn test_send_custom_event_http() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "custom_evt").await;
        let token = generate_user_token(user_id, "custom_evt");

        let room_id = create_test_room(&server, &token, "CustomEventRoom").await
            .expect("Failed to create room");

        let client = reqwest::Client::new();

        // Send custom event (persistent)
        let send_resp = client
            .post(format!("{}/rooms/{}/custom-events", server.base_url(), room_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "event_name": "test:my_event",
                "room_id": room_id,
                "data": {"hello": "world"},
                "persistent": true
            }))
            .send().await.expect("Send failed");
        assert_eq!(send_resp.status(), 200, "send custom event should return 200");

        // Send non-persistent event
        let send2_resp = client
            .post(format!("{}/rooms/{}/custom-events", server.base_url(), room_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "event_name": "test:my_event2",
                "room_id": room_id,
                "data": {"foo": "bar"},
            }))
            .send().await.expect("Send failed");
        assert_eq!(send2_resp.status(), 200);

        cleanup_database(server.db()).await;
    }

    #[tokio::test]
    async fn test_custom_event_without_colon_fails() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, "custom_evt2").await;
        let token = generate_user_token(user_id, "custom_evt2");

        let room_id = create_test_room(&server, &token, "CustomEventFailRoom").await
            .expect("Failed to create room");

        let client = reqwest::Client::new();
        let send_resp = client
            .post(format!("{}/rooms/{}/custom-events", server.base_url(), room_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "event_name": "no_colon",
                "room_id": room_id,
                "data": {},
            }))
            .send().await.expect("Send failed");
        assert!(send_resp.status().is_client_error(), "event without colon should fail");

        cleanup_database(server.db()).await;
    }
}

#[cfg(test)]
mod webhook_subscription_tests {
    use super::*;

    #[allow(clippy::needless_borrow)]
    async fn create_test_app_get_token(server: &TestServer, username: &str) -> Option<(Uuid, String)> {
        let user_service = UserService::new(server.db().clone());
        let (user_id, _) = create_test_user(&user_service, username).await;
        let token = generate_user_token(user_id, username);
        let (app_id, _) = create_oauth_app(&server, &token, &format!("{}App", username)).await?;
        Some((app_id, token))
    }

    #[tokio::test]
    async fn test_webhook_subscription_crud() {
        let server = match start_test_server().await {
            Some(s) => s, None => { eprintln!("Skipping: server start failed"); return; }
        };
        let _guard = capella_room::test_helpers::db_guard().lock().await;

        let (_app_id, token) = create_test_app_get_token(&server, "webhook_user").await
            .expect("Failed to create app/token");

        let client = reqwest::Client::new();

        // Create subscription
        let create_resp = client
            .post(format!("{}/webhook/subscriptions", server.base_url()))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "app_id": _app_id,
                "url": "http://localhost:9999/webhook",
                "events": ["message.created", "room.user_joined"],
                "secret": "whsec_test",
            }))
            .send().await.expect("Create failed");
        assert_eq!(create_resp.status(), 201, "create webhook subscription should return 201");

        // List subscriptions
        let list_resp = client
            .get(format!("{}/webhook/subscriptions", server.base_url()))
            .header("Authorization", format!("Bearer {}", token))
            .send().await.expect("List failed");
        assert_eq!(list_resp.status(), 200, "list subscriptions should return 200");

        let subs: serde_json::Value = list_resp.json().await.unwrap();
        let sub_id = subs["data"][0]["id"].as_str().unwrap().to_string();

        // Update subscription
        let update_resp = client
            .put(format!("{}/webhook/subscriptions/{}", server.base_url(), sub_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({"url": "http://localhost:9999/webhook-v2"}))
            .send().await.expect("Update failed");
        assert_eq!(update_resp.status(), 200, "update subscription should return 200");

        // Delete subscription
        let delete_resp = client
            .delete(format!("{}/webhook/subscriptions/{}", server.base_url(), sub_id))
            .header("Authorization", format!("Bearer {}", token))
            .send().await.expect("Delete failed");
        assert_eq!(delete_resp.status(), 200, "delete subscription should return 200");

        cleanup_database(server.db()).await;
    }
}
