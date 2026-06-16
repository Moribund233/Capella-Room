use std::env;
use std::sync::Arc;

use uuid::Uuid;

use capella_room::{
    config::{DatabaseConfig, JwtConfig},
    db::Database,
    error::AppError,
    models::user::UserRole,
    services::{auth_service::AuthService, oauth_service::OAuthService, user_service::UserService},
};

fn load_test_env() {
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

async fn setup_test_db() -> Database {
    load_test_env();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test or environment");

    let max_connections = env::var("APP_DATABASE__MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let db_config = DatabaseConfig {
        url: Some(database_url),
        max_connections,
        acquire_timeout_secs: 30,
        idle_timeout_secs: 600,
    };

    let db = Database::new(&db_config)
        .await
        .expect("Failed to connect to test database");

    db.migrate().await.expect("Failed to run migrations");

    db
}

async fn create_test_user(user_service: &UserService, username: &str) -> (Uuid, String) {
    let email = format!("{}@test.com", username);
    let password = "TestPassword123";

    if let Ok(Some(user)) = user_service.get_user_by_email(&email).await {
        return (user.id, password.to_string());
    }

    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret_key_for_testing_purposes_only".to_string()),
        expiration_hours: 24,
    });

    let password_hash = auth_service.hash_password(password).unwrap();

    let user = user_service
        .create_user(username, &email, &password_hash)
        .await
        .unwrap();

    (user.id, password.to_string())
}

async fn cleanup_database(db: &Database) {
    sqlx::query("DELETE FROM oauth_authorization_codes").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM oauth_tokens").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM user_identity_mappings").execute(db.pool()).await.ok();
    sqlx::query("DELETE FROM oauth_apps").execute(db.pool()).await.ok();
}

fn setup_service(db: Database) -> OAuthService {
    OAuthService::new(db, "test_oauth_secret")
}

fn create_token_for_user(user_id: Uuid, username: &str) -> String {
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret_key_for_testing_purposes_only".to_string()),
        expiration_hours: 24,
    });
    let tokens = auth_service
        .generate_token_pair(user_id, username, UserRole::User)
        .unwrap();
    tokens.access_token
}

struct OAuthTestServer {
    base_url: String,
    db: Database,
}

impl OAuthTestServer {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn db(&self) -> &Database {
        &self.db
    }
}

async fn start_oauth_test_server() -> Option<OAuthTestServer> {
    load_test_env();
    std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes_only");

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
    let metrics_collector = Arc::new(
        capella_room::utils::logging::MetricsCollector::new(),
    );

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
    )
    .await
    .ok()?;

    let app = capella_room::routes::create_router(state);
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

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Some(OAuthTestServer { base_url, db })
}

async fn create_user_get_token(
    user_service: &UserService,
    username: &str,
) -> Option<(Uuid, String)> {
    let (user_id, _) = create_test_user(user_service, username).await;
    let token = create_token_for_user(user_id, username);
    Some((user_id, token))
}

#[cfg(test)]
mod oauth_app_api_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_app_returns_201_with_client_secret() {
        let server = match start_oauth_test_server().await {
            Some(s) => s,
            None => {
                eprintln!("Skipping test: DATABASE_URL not set or server failed to start");
                return;
            }
        };

        let user_service = UserService::new(server.db().clone());
        let (_user_id, token) = match create_user_get_token(&user_service, "oauth_api_create").await {
            Some(t) => t,
            None => {
                eprintln!("Skipping: could not create test user");
                return;
            }
        };

        let client = reqwest::Client::new();

        let resp = client
            .post(format!("{}/api/v2/oauth/apps", server.base_url()))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "name": "My API App",
                "description": "Created via API",
                "redirect_uris": ["http://localhost:3000/callback"],
                "scopes": ["read", "write"]
            }))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(
            resp.status(),
            201,
            "create app should return 201"
        );
        let json: serde_json::Value = resp.json().await.unwrap();
        assert!(json.get("client_secret").is_some(), "response should include client_secret");
        assert_eq!(json["name"], "My API App");
    }

    #[tokio::test]
    async fn test_list_apps_returns_200_with_user_apps() {
        let server = match start_oauth_test_server().await {
            Some(s) => s,
            None => {
                eprintln!("Skipping test: DATABASE_URL not set or server failed to start");
                return;
            }
        };

        let user_service = UserService::new(server.db().clone());
        let (_user_id, token) = match create_user_get_token(&user_service, "oauth_api_list").await {
            Some(t) => t,
            None => {
                eprintln!("Skipping: could not create test user");
                return;
            }
        };

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/api/v2/oauth/apps", server.base_url()))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(resp.status(), 200, "list apps should return 200");
        let apps: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert!(apps.is_empty(), "new user should have no apps");
    }

    #[tokio::test]
    async fn test_delete_app_by_non_owner_returns_404() {
        let server = match start_oauth_test_server().await {
            Some(s) => s,
            None => {
                eprintln!("Skipping test: DATABASE_URL not set or server failed to start");
                return;
            }
        };

        let user_service = UserService::new(server.db().clone());
        let (_owner_id, owner_token) = match create_user_get_token(&user_service, "oauth_del_owner").await {
            Some(t) => t,
            None => {
                eprintln!("Skipping: could not create owner user");
                return;
            }
        };
        let (_non_owner_id, non_owner_token) = match create_user_get_token(&user_service, "oauth_del_nonowner").await {
            Some(t) => t,
            None => {
                eprintln!("Skipping: could not create non-owner user");
                return;
            }
        };

        let client = reqwest::Client::new();
        let create_resp = client
            .post(format!("{}/api/v2/oauth/apps", server.base_url()))
            .header("Authorization", format!("Bearer {}", owner_token))
            .json(&serde_json::json!({
                "name": "App to Delete",
                "redirect_uris": ["http://localhost:3000/callback"],
                "scopes": ["read"]
            }))
            .send()
            .await
            .expect("Failed to create app");
        assert_eq!(create_resp.status(), 201, "create must succeed first");
        let app: serde_json::Value = create_resp.json().await.unwrap();
        let app_id = app["id"].as_str().unwrap().to_string();

        let del_resp = client
            .delete(format!("{}/api/v2/oauth/apps/{}", server.base_url(), app_id))
            .header("Authorization", format!("Bearer {}", non_owner_token))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(
            del_resp.status(),
            404,
            "non-owner delete should return 404"
        );
    }
}

#[cfg(test)]
mod oauth_app_service_tests {
    use super::*;

    #[tokio::test]
    async fn test_register_app() {
        let db = setup_test_db().await;
        cleanup_database(&db).await;
        let _user_service = UserService::new(db.clone());
        let (user_id, _) = create_test_user(&_user_service, "oauth_register_app").await;
        let service = setup_service(db.clone());

        let app = service
            .register_app(
                user_id,
                "My Test App",
                Some("A test OAuth application"),
                &["http://localhost:3000/callback"],
                &["read", "write"],
            )
            .await
            .unwrap();

        assert_eq!(app.name, "My Test App");
        assert_eq!(
            app.description,
            Some("A test OAuth application".to_string())
        );
        assert_eq!(app.owner_id, user_id);
        assert!(app.is_active);
        assert!(app.client_secret.starts_with("capella_sk_"));
        assert_eq!(
            app.redirect_uris,
            vec!["http://localhost:3000/callback".to_string()]
        );
        assert_eq!(app.scopes, vec!["read".to_string(), "write".to_string()]);
    }

    #[tokio::test]
    async fn test_list_apps() {
        let db = setup_test_db().await;
        cleanup_database(&db).await;
        let user_service = UserService::new(db.clone());
        let (user1_id, _) = create_test_user(&user_service, "oauth_list_user1").await;
        let (user2_id, _) = create_test_user(&user_service, "oauth_list_user2").await;
        let service = setup_service(db.clone());

        service
            .register_app(
                user1_id,
                "App 1",
                None,
                &["http://localhost:3000/callback"],
                &["read"],
            )
            .await
            .unwrap();
        service
            .register_app(
                user1_id,
                "App 2",
                None,
                &["http://localhost:3000/callback"],
                &["read"],
            )
            .await
            .unwrap();
        service
            .register_app(
                user2_id,
                "App 3",
                None,
                &["http://localhost:3000/callback"],
                &["read"],
            )
            .await
            .unwrap();

        let user1_apps = service.list_apps(user1_id).await.unwrap();
        assert_eq!(user1_apps.len(), 2);
        assert!(user1_apps.iter().all(|a| a.owner_id == user1_id));

        let user2_apps = service.list_apps(user2_id).await.unwrap();
        assert_eq!(user2_apps.len(), 1);
        assert!(user2_apps.iter().all(|a| a.owner_id == user2_id));
    }

    #[tokio::test]
    async fn test_get_app() {
        let db = setup_test_db().await;
        cleanup_database(&db).await;
        let user_service = UserService::new(db.clone());
        let (user_id, _) = create_test_user(&user_service, "oauth_get_app").await;
        let service = setup_service(db.clone());

        let app = service
            .register_app(
                user_id,
                "Get App Test",
                Some("Get app description"),
                &["http://localhost:3000/callback"],
                &["read"],
            )
            .await
            .unwrap();

        let fetched = service.get_app(app.id).await.unwrap();

        assert_eq!(fetched.id, app.id);
        assert_eq!(fetched.name, "Get App Test");
        assert_eq!(fetched.description, Some("Get app description".to_string()));
        assert_eq!(fetched.owner_id, user_id);
        assert!(fetched.is_active);
    }

    #[tokio::test]
    async fn test_update_app() {
        let db = setup_test_db().await;
        cleanup_database(&db).await;
        let user_service = UserService::new(db.clone());
        let (user_id, _) = create_test_user(&user_service, "oauth_update_app").await;
        let service = setup_service(db.clone());

        let app = service
            .register_app(
                user_id,
                "Original Name",
                Some("Original description"),
                &["http://localhost:3000/callback"],
                &["read"],
            )
            .await
            .unwrap();

        let updated = service
            .update_app(
                app.id,
                user_id,
                Some("Updated Name"),
                Some("Updated description"),
                Some(&["http://localhost:3000/new-callback"]),
                Some(&["read", "write"]),
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(
            updated.redirect_uris,
            vec!["http://localhost:3000/new-callback".to_string()]
        );
        assert_eq!(updated.scopes, vec!["read".to_string(), "write".to_string()]);
    }

    #[tokio::test]
    async fn test_delete_app() {
        let db = setup_test_db().await;
        cleanup_database(&db).await;
        let user_service = UserService::new(db.clone());
        let (owner_id, _) = create_test_user(&user_service, "oauth_del_owner").await;
        let (non_owner_id, _) = create_test_user(&user_service, "oauth_del_nonowner").await;
        let service = setup_service(db.clone());

        let app = service
            .register_app(
                owner_id,
                "App to Delete",
                None,
                &["http://localhost:3000/callback"],
                &["read"],
            )
            .await
            .unwrap();

        let result = service.delete_app(app.id, non_owner_id).await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound));

        service.delete_app(app.id, owner_id).await.unwrap();

        let result = service.get_app(app.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rotate_secret() {
        let db = setup_test_db().await;
        cleanup_database(&db).await;
        let user_service = UserService::new(db.clone());
        let (owner_id, _) = create_test_user(&user_service, "oauth_rotate").await;
        let (non_owner_id, _) = create_test_user(&user_service, "oauth_rotate_nonowner").await;
        let service = setup_service(db.clone());

        let app = service
            .register_app(
                owner_id,
                "Rotate Test",
                None,
                &["http://localhost:3000/callback"],
                &["read"],
            )
            .await
            .unwrap();
        let old_secret = app.client_secret.clone();

        let result = service.rotate_secret(app.id, non_owner_id).await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound));

        let new_secret = service.rotate_secret(app.id, owner_id).await.unwrap();
        assert_ne!(new_secret, old_secret);
        assert!(new_secret.starts_with("capella_sk_"));

        let refreshed = service.get_app(app.id).await.unwrap();
        assert!(
            !OAuthService::verify_client_secret(&old_secret, &refreshed.client_secret),
            "old secret should not verify against new hash"
        );
        assert!(
            OAuthService::verify_client_secret(&new_secret, &refreshed.client_secret),
            "new secret should verify against stored hash"
        );
    }
}
