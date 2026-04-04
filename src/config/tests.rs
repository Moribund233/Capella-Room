#[cfg(test)]
mod config_tests {
    use std::io::Write;
    use tempfile::NamedTempFile;

    use crate::config::ConfigLoader;

    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to temp file");
        file.flush().expect("Failed to flush temp file");
        file
    }

    fn create_valid_test_config() -> String {
        r#"
[app]
env = "test"

[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgres://test:test@localhost:5432/test"
max_connections = 5

[jwt]
secret = "test-secret-key-for-testing-only"
expiration_hours = 24

[upload]
max_file_size = 10485760
base_url = "/uploads"

[rate_limit]
enabled = true
default_requests = 100
default_window_secs = 60
auth_requests = 5
auth_window_secs = 60
message_requests = 30
message_window_secs = 60
room_requests = 20
room_window_secs = 60

[websocket]
heartbeat_interval_secs = 30
heartbeat_timeout_secs = 90
auth_timeout_secs = 30
message_buffer_size = 100

[reconnect]
base_delay_ms = 1000
max_delay_ms = 30000
max_attempts = 5
multiplier = 2

[logging]
level = "info"
structured = true

[cors]
allowed_origins = ["*"]
allowed_methods = ["GET", "POST"]
allowed_headers = ["*"]
allow_credentials = false
max_age = 3600

[system]
name = "Test System"
description = "Test Description"
version = "1.0.0"
maintenance_mode = false
maintenance_message = "Test message"

[admin.initial]
enabled = false
username = "admin"
email = "admin@test.com"
"#
        .to_string()
    }

    #[test]
    fn test_load_minimal_config() {
        let config_content = r#"
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgres://test:test@localhost:5432/test"
max_connections = 5

[jwt]
secret = "test-secret-key"
expiration_hours = 24

[upload]
max_file_size = 10485760
base_url = "/uploads"

[rate_limit]
enabled = true
default_requests = 100
default_window_secs = 60
auth_requests = 5
auth_window_secs = 60
message_requests = 30
message_window_secs = 60
room_requests = 20
room_window_secs = 60

[websocket]
heartbeat_interval_secs = 30
heartbeat_timeout_secs = 90
auth_timeout_secs = 30
message_buffer_size = 100

[reconnect]
base_delay_ms = 1000
max_delay_ms = 30000
max_attempts = 5
multiplier = 2

[logging]
level = "info"
structured = true

[cors]
allowed_origins = ["*"]
allowed_methods = ["GET", "POST"]
allowed_headers = ["*"]
allow_credentials = false
max_age = 3600

[system]
name = "Test System"
description = "Test Description"
version = "1.0.0"
maintenance_mode = false
maintenance_message = "Test message"
"#;
        let temp_file = create_temp_config(config_content);
        let path = temp_file.path().to_str().unwrap();

        let config = ConfigLoader::load_from_file_only(path).expect("Failed to load config");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 5);
        assert_eq!(config.jwt.expiration_hours, 24);
    }

    #[test]
    fn test_load_full_config() {
        let config_content = r#"
[app]
env = "production"

[server]
host = "0.0.0.0"
port = 3000

[database]
url = "postgres://user:pass@localhost:5432/prod"
max_connections = 20

[jwt]
secret = "production-secret-key"
expiration_hours = 12

[upload]
max_file_size = 20971520
base_url = "/files"

[rate_limit]
enabled = true
default_requests = 200
default_window_secs = 120
auth_requests = 10
auth_window_secs = 60
message_requests = 50
message_window_secs = 60
room_requests = 30
room_window_secs = 60

[websocket]
heartbeat_interval_secs = 45
heartbeat_timeout_secs = 120
auth_timeout_secs = 60
message_buffer_size = 200

[reconnect]
base_delay_ms = 2000
max_delay_ms = 60000
max_attempts = 10
multiplier = 3

[logging]
level = "debug"
structured = false

[cors]
allowed_origins = ["https://example.com"]
allowed_methods = ["GET", "POST"]
allowed_headers = ["*"]
allow_credentials = true
max_age = 7200

[system]
name = "Test System"
description = "Test Description"
version = "2.0.0"
maintenance_mode = true
maintenance_message = "Under maintenance"

[admin.initial]
enabled = false
username = "admin"
email = "admin@test.com"
"#;
        let temp_file = create_temp_config(config_content);
        let path = temp_file.path().to_str().unwrap();

        let config = ConfigLoader::load_from_file_only(path).expect("Failed to load config");

        assert_eq!(config.app.env, "production");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.jwt.expiration_hours, 12);
        assert_eq!(config.upload.max_file_size, 20971520);
        assert_eq!(config.upload.base_url, "/files");
        assert!(config.rate_limit.enabled);
        assert_eq!(config.rate_limit.default_requests, 200);
        assert_eq!(config.rate_limit.default_window_secs, 120);
        assert_eq!(config.websocket.heartbeat_interval_secs, 45);
        assert_eq!(config.websocket.heartbeat_timeout_secs, 120);
        assert_eq!(config.logging.level, "debug");
        assert!(!config.logging.structured);
        assert_eq!(config.cors.allowed_origins, vec!["https://example.com"]);
        assert!(config.cors.allow_credentials);
        assert_eq!(config.system.name, "Test System");
        assert!(config.system.maintenance_mode);
        assert!(!config.admin.initial.enabled);
    }

    #[test]
    fn test_config_clone() {
        let config_content = create_valid_test_config();
        let temp_file = create_temp_config(&config_content);
        let path = temp_file.path().to_str().unwrap();

        let config = ConfigLoader::load_from_file_only(path).expect("Failed to load config");
        let cloned = config.clone();

        assert_eq!(config.server.host, cloned.server.host);
        assert_eq!(config.server.port, cloned.server.port);
    }

    #[test]
    fn test_missing_required_field() {
        let config_content = r#"
[server]
host = "0.0.0.0"

[database]
max_connections = 10
"#;
        let temp_file = create_temp_config(config_content);
        let path = temp_file.path().to_str().unwrap();

        let result = ConfigLoader::load_from_file_only(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_toml() {
        let config_content = r#"
[server
host = "0.0.0.0"
"#;
        let temp_file = create_temp_config(config_content);
        let path = temp_file.path().to_str().unwrap();

        let result = ConfigLoader::load_from_file_only(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_jwt_secret_validation() {
        let config_content = r#"
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "postgres://test:test@localhost:5432/test"
max_connections = 10

[jwt]
expiration_hours = 24

[upload]
max_file_size = 10485760
base_url = "/uploads"

[rate_limit]
enabled = true
default_requests = 100
default_window_secs = 60
auth_requests = 5
auth_window_secs = 60
message_requests = 30
message_window_secs = 60
room_requests = 20
room_window_secs = 60

[websocket]
heartbeat_interval_secs = 30
heartbeat_timeout_secs = 90
auth_timeout_secs = 30
message_buffer_size = 100

[reconnect]
base_delay_ms = 1000
max_delay_ms = 30000
max_attempts = 5
multiplier = 2

[logging]
level = "info"
structured = true

[cors]
allowed_origins = ["*"]
allowed_methods = ["GET"]
allowed_headers = ["*"]
allow_credentials = false
max_age = 3600

[system]
name = "Test"
description = "Test"
version = "1.0.0"
maintenance_mode = false
maintenance_message = "Test"
"#;
        let temp_file = create_temp_config(config_content);
        let path = temp_file.path().to_str().unwrap();

        let result = ConfigLoader::load_from_file_only(path);
        assert!(result.is_err(), "Should fail because JWT_SECRET is not set");
    }
}
