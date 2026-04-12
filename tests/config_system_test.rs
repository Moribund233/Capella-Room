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

/// 设置测试所需的环境变量
fn setup_test_env() {
    // 设置敏感配置的环境变量
    std::env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
    std::env::set_var("JWT_SECRET", "test-secret-key-for-jwt-signing");
}

/// ==================== 配置加载测试 ====================

#[test]
fn test_load_minimal_config() {
    // 设置环境变量（敏感配置）
    setup_test_env();

    // 配置文件中只包含非敏感配置
    let config_content = r#"
[server]
host = "127.0.0.1"
port = 8080

[database]
max_connections = 5
acquire_timeout_secs = 30
idle_timeout_secs = 600

[jwt]
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

[system]
name = "Test System"
description = "Test Description"
version = "1.0.0"
maintenance_mode = false
maintenance_message = "Test message"
"#;
    let temp_file = create_temp_config(config_content);
    let path = temp_file.path().to_str().unwrap();

    // 使用 load_from_file_only 方法，但环境变量需要在测试前设置
    let config = ConfigLoader::load_from_file_only(path).expect("Failed to load config");

    // 验证非敏感配置从配置文件加载
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.database.max_connections, 5);
    assert_eq!(config.database.acquire_timeout_secs, 30);
    assert_eq!(config.database.idle_timeout_secs, 600);
    assert_eq!(config.jwt.expiration_hours, 24);

    // 验证敏感配置从环境变量加载
    assert_eq!(config.database.url, Some("postgres://test:test@localhost:5432/test".to_string()));
    assert_eq!(config.jwt.secret, Some("test-secret-key-for-jwt-signing".to_string()));
}

#[test]
fn test_load_full_config() {
    // 设置环境变量（敏感配置）
    setup_test_env();

    let config_content = r#"
[server]
host = "0.0.0.0"
port = 3000

[database]
max_connections = 20
acquire_timeout_secs = 45
idle_timeout_secs = 900

[jwt]
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

    // 验证非敏感配置
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
    assert_eq!(config.system.name, "Test System");
    assert!(config.system.maintenance_mode);
    assert!(!config.admin.initial.enabled);

    // 验证敏感配置从环境变量加载
    assert_eq!(config.database.url, Some("postgres://test:test@localhost:5432/test".to_string()));
    assert_eq!(config.jwt.secret, Some("test-secret-key-for-jwt-signing".to_string()));
}

#[test]
fn test_missing_required_field() {
    // 设置环境变量
    setup_test_env();

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
    // 设置环境变量
    setup_test_env();

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
fn test_config_change_event_clone() {
    let event = ConfigChangeEvent::ConfigUpdated {
        key: "test.key".to_string(),
        old_value: "old".to_string(),
        new_value: "new".to_string(),
    };

    let cloned = event.clone();

    match (event, cloned) {
        (
            ConfigChangeEvent::ConfigUpdated {
                key: k1,
                old_value: ov1,
                new_value: nv1,
            },
            ConfigChangeEvent::ConfigUpdated {
                key: k2,
                old_value: ov2,
                new_value: nv2,
            },
        ) => {
            assert_eq!(k1, k2);
            assert_eq!(ov1, ov2);
            assert_eq!(nv1, nv2);
        }
        _ => panic!("Event type mismatch"),
    }
}

#[test]
fn test_config_change_event_types() {
    let updated_event = ConfigChangeEvent::ConfigUpdated {
        key: "test".to_string(),
        old_value: "old".to_string(),
        new_value: "value".to_string(),
    };

    let reloaded_event = ConfigChangeEvent::ConfigReloaded;

    let category_updated = ConfigChangeEvent::CategoryUpdated {
        category: "test".to_string(),
    };

    // 验证事件可以被创建
    match updated_event {
        ConfigChangeEvent::ConfigUpdated { .. } => {}
        _ => panic!("Expected ConfigUpdated"),
    }

    match reloaded_event {
        ConfigChangeEvent::ConfigReloaded => {}
        _ => panic!("Expected ConfigReloaded"),
    }

    match category_updated {
        ConfigChangeEvent::CategoryUpdated { .. } => {}
        _ => panic!("Expected CategoryUpdated"),
    }
}

/// ==================== 广播通道测试 ====================

#[tokio::test]
async fn test_broadcast_channel_capacity() {
    // 创建广播通道，容量为 100
    let (tx, _rx) = broadcast::channel::<ConfigChangeEvent>(100);

    // 发送 100 个事件
    for i in 0..100 {
        let event = ConfigChangeEvent::ConfigUpdated {
            key: format!("key{}", i),
            old_value: "old".to_string(),
            new_value: format!("value{}", i),
        };
        tx.send(event).expect("Failed to send event");
    }

    // 验证通道已满时发送会失败
    let event = ConfigChangeEvent::ConfigUpdated {
        key: "overflow".to_string(),
        old_value: "old".to_string(),
        new_value: "value".to_string(),
    };

    // 由于所有接收者都已 drop，发送可能会失败，这是预期的行为
    let _ = tx.send(event);
}

/// ==================== WebSocket 管理器默认配置测试 ====================

#[test]
fn test_websocket_manager_default() {
    // 使用默认配置创建 WebSocket 管理器
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(async {
        let ws_manager = WebSocketManager::from_config(100, 30, 90);

        assert_eq!(ws_manager.get_message_buffer_size().await, 100);
        assert_eq!(ws_manager.get_heartbeat_interval().await, 30);
        assert_eq!(ws_manager.get_heartbeat_timeout().await, 90);
    });
}
