//! 配置系统集成测试
//!
//! 测试配置加载、热重载机制、配置变更事件等功能

use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::sync::broadcast;

use seredeli_room::config::{ConfigChangeEvent, ConfigLoader};
use seredeli_room::websocket::manager::WebSocketManager;

/// 创建临时配置文件
fn create_temp_config(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    file.flush().expect("Failed to flush temp file");
    file
}

/// ==================== 配置加载测试 ====================

#[test]
fn test_load_minimal_config() {
    let config_content = r#"
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgres://test:test@localhost:5432/test"
max_connections = 5
acquire_timeout_secs = 30
idle_timeout_secs = 600

[jwt]
secret = "test-secret-key"
expiration_hours = 24

[upload]
max_file_size = 10485760
base_url = "/uploads"

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
    assert_eq!(config.database.acquire_timeout_secs, 30);
    assert_eq!(config.database.idle_timeout_secs, 600);
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
acquire_timeout_secs = 45
idle_timeout_secs = 900

[jwt]
secret = "production-secret-key"
expiration_hours = 12

[upload]
max_file_size = 20971520
base_url = "/files"

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
    assert_eq!(config.database.acquire_timeout_secs, 45);
    assert_eq!(config.database.idle_timeout_secs, 900);
    assert_eq!(config.jwt.expiration_hours, 12);
    assert_eq!(config.upload.max_file_size, 20971520);
    assert_eq!(config.upload.base_url, "/files");
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

/// ==================== 热重载机制测试 ====================

#[tokio::test]
async fn test_websocket_config_update() {
    // 创建 WebSocket 管理器
    let ws_manager = Arc::new(WebSocketManager::from_config(100, 30, 90));

    // 验证初始配置
    assert_eq!(ws_manager.get_message_buffer_size().await, 100);
    assert_eq!(ws_manager.get_heartbeat_interval().await, 30);
    assert_eq!(ws_manager.get_heartbeat_timeout().await, 90);

    // 更新配置
    ws_manager.set_message_buffer_size(200).await;
    ws_manager.set_heartbeat_interval(45).await;
    ws_manager.set_heartbeat_timeout(120).await;

    // 验证更新后的配置
    assert_eq!(ws_manager.get_message_buffer_size().await, 200);
    assert_eq!(ws_manager.get_heartbeat_interval().await, 45);
    assert_eq!(ws_manager.get_heartbeat_timeout().await, 120);
}

/// ==================== 配置变更事件测试 ====================

#[test]
fn test_config_change_event_types() {
    // 测试 ConfigUpdated 事件
    let event = ConfigChangeEvent::ConfigUpdated {
        key: "websocket.heartbeat_interval_secs".to_string(),
        old_value: "30".to_string(),
        new_value: "45".to_string(),
    };

    match event {
        ConfigChangeEvent::ConfigUpdated {
            key,
            old_value,
            new_value,
        } => {
            assert_eq!(key, "websocket.heartbeat_interval_secs");
            assert_eq!(old_value, "30");
            assert_eq!(new_value, "45");
        }
        _ => panic!("Expected ConfigUpdated event"),
    }

    // 测试 CategoryUpdated 事件
    let event = ConfigChangeEvent::CategoryUpdated {
        category: "websocket".to_string(),
    };

    match event {
        ConfigChangeEvent::CategoryUpdated { category } => {
            assert_eq!(category, "websocket");
        }
        _ => panic!("Expected CategoryUpdated event"),
    }

    // 测试 ConfigReloaded 事件
    let event = ConfigChangeEvent::ConfigReloaded;
    assert!(matches!(event, ConfigChangeEvent::ConfigReloaded));
}

#[test]
fn test_config_change_event_clone() {
    let event = ConfigChangeEvent::ConfigUpdated {
        key: "test.key".to_string(),
        old_value: "old".to_string(),
        new_value: "new".to_string(),
    };

    let cloned = event.clone();

    match cloned {
        ConfigChangeEvent::ConfigUpdated {
            key,
            old_value,
            new_value,
        } => {
            assert_eq!(key, "test.key");
            assert_eq!(old_value, "old");
            assert_eq!(new_value, "new");
        }
        _ => panic!("Expected ConfigUpdated event"),
    }
}

#[test]
fn test_broadcast_channel_capacity() {
    // 创建广播通道，容量为 100
    let (tx, _rx) = broadcast::channel::<ConfigChangeEvent>(100);

    // 发送多个事件
    for i in 0..50 {
        let event = ConfigChangeEvent::ConfigUpdated {
            key: format!("key_{}", i),
            old_value: "old".to_string(),
            new_value: "new".to_string(),
        };
        assert!(tx.send(event).is_ok());
    }

    // 验证可以创建多个订阅者
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();

    // 新订阅者不会收到历史消息（这是 broadcast 的特性）
    // 但应该能接收新消息
    let new_event = ConfigChangeEvent::ConfigReloaded;
    assert!(tx.send(new_event.clone()).is_ok());

    // 两个订阅者都应该收到消息
    assert!(rx1.try_recv().is_ok());
    assert!(rx2.try_recv().is_ok());
}

/// ==================== 组件默认值测试 ====================

#[test]
fn test_websocket_manager_default() {
    let manager = WebSocketManager::default();

    // 验证默认值
    assert_eq!(manager.get_total_connections(), 0);
}
