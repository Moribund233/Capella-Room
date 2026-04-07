//! Redis 集成测试
//!
//! 测试 Redis Pub/Sub 功能，包括：
//! - 配置加载
//! - 消息序列化/反序列化
//! - 跨节点消息广播（模拟）

use seredeli_room::config::RedisConfig;
use seredeli_room::redis::pubsub::RoomBroadcastMessage;
use uuid::Uuid;

/// 测试 Redis 配置默认值
#[test]
fn test_redis_config_defaults() {
    let config = RedisConfig::default();

    // URL 默认为空字符串（需通过环境变量 REDIS_URL 设置）
    assert_eq!(config.url, "");
    // 默认禁用 Redis
    assert!(!config.enabled);
    // 基础类型字段使用 serde default，数值类型默认为 0
    assert_eq!(config.pool_size, 0);
    assert_eq!(config.timeout_secs, 0);
    // String 类型默认为空
    assert_eq!(config.channel_prefix, "");
    assert_eq!(config.stream_max_len, 0);
    assert_eq!(config.consumer_batch_size, 0);
    assert_eq!(config.consumer_poll_interval_ms, 0);
    // bool 类型默认为 false
    assert!(!config.config_sync_enabled);
}

/// 测试 Redis 配置可通过环境变量覆盖
#[test]
fn test_redis_config_from_env() {
    // 注意：此测试验证配置结构体可以正确存储从环境变量读取的值
    // 实际的环境变量读取由配置加载器处理

    // 创建自定义配置（模拟从环境变量加载后的状态）
    let config = RedisConfig {
        url: "redis://192.168.1.100:6380".to_string(),
        enabled: true,
        pool_size: 20,
        timeout_secs: 10,
        channel_prefix: "myapp".to_string(),
        stream_max_len: 50000,
        consumer_batch_size: 50,
        consumer_poll_interval_ms: 500,
        config_sync_enabled: false,
    };

    assert!(config.enabled);
    assert_eq!(config.url, "redis://192.168.1.100:6380");
    assert_eq!(config.pool_size, 20);
    assert_eq!(config.timeout_secs, 10);
    assert_eq!(config.channel_prefix, "myapp");
    assert_eq!(config.stream_max_len, 50000);
    assert_eq!(config.consumer_batch_size, 50);
    assert_eq!(config.consumer_poll_interval_ms, 500);
    assert!(!config.config_sync_enabled);
}

/// 测试房间广播消息序列化
#[test]
fn test_room_broadcast_message_serialization() {
    let room_id = Uuid::new_v4();
    let exclude_user = Some(Uuid::new_v4());

    let msg = RoomBroadcastMessage::new(
        room_id,
        r#"{"type":"chat","content":"Hello"}"#.to_string(),
        exclude_user,
        "node-1".to_string(),
    );

    // 序列化
    let json = msg.to_json().expect("Failed to serialize message");

    // 反序列化
    let deserialized =
        RoomBroadcastMessage::from_json(&json).expect("Failed to deserialize message");

    assert_eq!(msg.room_id, deserialized.room_id);
    assert_eq!(msg.message, deserialized.message);
    assert_eq!(msg.exclude_user, deserialized.exclude_user);
    assert_eq!(msg.source_node, deserialized.source_node);
}

/// 测试房间广播消息（无排除用户）
#[test]
fn test_room_broadcast_message_without_exclude() {
    let room_id = Uuid::new_v4();

    let msg = RoomBroadcastMessage::new(
        room_id,
        r#"{"type":"notification","content":"System message"}"#.to_string(),
        None,
        "node-2".to_string(),
    );

    let json = msg.to_json().expect("Failed to serialize message");
    let deserialized =
        RoomBroadcastMessage::from_json(&json).expect("Failed to deserialize message");

    assert!(deserialized.exclude_user.is_none());
    assert_eq!(deserialized.source_node, "node-2");
}

/// 测试房间广播消息时间戳
#[test]
fn test_room_broadcast_message_timestamp() {
    let before = chrono::Utc::now();

    let msg = RoomBroadcastMessage::new(
        Uuid::new_v4(),
        "test".to_string(),
        None,
        "node-1".to_string(),
    );

    let after = chrono::Utc::now();

    assert!(msg.timestamp >= before);
    assert!(msg.timestamp <= after);
}

/// 测试消息 JSON 格式
#[test]
fn test_room_broadcast_message_json_format() {
    let room_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let exclude_user = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    let msg = RoomBroadcastMessage::new(
        room_id,
        "Hello, World!".to_string(),
        Some(exclude_user),
        "test-node".to_string(),
    );

    let json = msg.to_json().expect("Failed to serialize");

    // 验证 JSON 包含预期字段
    assert!(json.contains("\"room_id\""));
    assert!(json.contains("\"message\""));
    assert!(json.contains("\"exclude_user\""));
    assert!(json.contains("\"source_node\""));
    assert!(json.contains("\"timestamp\""));
    assert!(json.contains("550e8400-e29b-41d4-a716-446655440000"));
    assert!(json.contains("550e8400-e29b-41d4-a716-446655440001"));
    assert!(json.contains("test-node"));
}

/// 测试无效 JSON 反序列化失败
#[test]
fn test_invalid_json_deserialization() {
    let invalid_json = r#"{"invalid": "json"}"#;
    let result = RoomBroadcastMessage::from_json(invalid_json);

    // 应该失败，因为缺少必要字段
    assert!(result.is_err());
}

/// 测试损坏的 JSON
#[test]
fn test_malformed_json_deserialization() {
    let malformed_json = r#"{"room_id": "not-a-uuid", "message": "test"}"#;
    let result = RoomBroadcastMessage::from_json(malformed_json);

    assert!(result.is_err());
}
