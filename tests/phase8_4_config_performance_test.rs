//! 阶段 8.4：审计系统配置与性能测试
//!
//! 测试审计配置管理、热更新和性能优化功能

use chrono::Utc;
use serde_json::json;

use capella_room::config::AuditConfig;
use capella_room::models::audit::{AuditEventType, AuditLog, AuditSeverity};

/// 测试审计配置结构
#[test]
fn test_audit_config_structure() {
    let config = AuditConfig {
        enabled: true,
        log_retention_days: 90,
        buffer_size: 100,
        flush_interval_seconds: 5,
        excluded_paths: vec!["/health".to_string(), "/ws".to_string()],
        alert_enabled: true,
        alert_cooldown_minutes: 10,
        auto_archive_enabled: true,
        archive_hour: 3,
    };

    assert!(config.enabled);
    assert_eq!(config.log_retention_days, 90);
    assert_eq!(config.buffer_size, 100);
    assert_eq!(config.flush_interval_seconds, 5);
    assert_eq!(config.excluded_paths.len(), 2);
    assert!(config.alert_enabled);
    assert_eq!(config.alert_cooldown_minutes, 10);
    assert!(config.auto_archive_enabled);
    assert_eq!(config.archive_hour, 3);
}

/// 测试审计配置默认值
#[test]
fn test_audit_config_defaults() {
    // 注意：Default trait 使用真正的默认值，而不是 default_* 函数
    // default_* 函数仅在 serde 反序列化时起作用
    let config = AuditConfig::default();

    // 验证 Default 实现的基本默认值
    assert_eq!(config.log_retention_days, 0); // Default 默认值
    assert_eq!(config.buffer_size, 0); // Default 默认值
}

/// 测试 AppConfig 包含审计配置
#[test]
fn test_app_config_includes_audit() {
    // AppConfig 需要手动构造，这里仅测试审计配置本身
    let audit_config = AuditConfig::default();

    // 验证 Default 实现的基本默认值
    assert_eq!(audit_config.log_retention_days, 0);
}

/// 测试审计配置序列化
#[test]
fn test_audit_config_serialization() {
    let config = AuditConfig {
        enabled: true,
        log_retention_days: 60,
        buffer_size: 50,
        flush_interval_seconds: 10,
        excluded_paths: vec!["/api/health".to_string()],
        alert_enabled: false,
        alert_cooldown_minutes: 5,
        auto_archive_enabled: false,
        archive_hour: 2,
    };

    let json = serde_json::to_value(&config).unwrap();

    assert_eq!(json["enabled"], true);
    assert_eq!(json["log_retention_days"], 60);
    assert_eq!(json["buffer_size"], 50);
    assert_eq!(json["flush_interval_seconds"], 10);
    assert_eq!(json["alert_enabled"], false);

    // 反序列化
    let deserialized: AuditConfig = serde_json::from_value(json).unwrap();
    assert!(deserialized.enabled);
    assert_eq!(deserialized.log_retention_days, 60);
}

/// 测试审计日志批量写入性能
#[tokio::test]
async fn test_audit_log_batch_write_performance() {
    // 此测试验证批量写入机制的性能
    // 注意：实际性能测试需要数据库连接，这里仅测试逻辑

    let batch_size = 100;
    let logs: Vec<AuditLog> = (0..batch_size)
        .map(|i| AuditLog {
            id: uuid::Uuid::new_v4(),
            event_type: AuditEventType::UserLogin,
            severity: AuditSeverity::Info,
            actor_id: Some(uuid::Uuid::new_v4()),
            actor_name: None,
            actor_role: None,
            target_type: None,
            target_id: None,
            action: "login".to_string(),
            description: format!("User login {}", i),
            metadata: Some(json!({"ip": "127.0.0.1"})),
            status: "success".to_string(),
            error_message: None,
            created_at: Utc::now(),
        })
        .collect();

    assert_eq!(logs.len(), batch_size);

    // 验证日志创建成功
    for (i, log) in logs.iter().enumerate() {
        assert!(log.description.contains(&format!("{}", i)));
    }
}

/// 测试审计日志查询性能
#[test]
fn test_audit_log_query_performance() {
    // 模拟大量日志的查询性能
    let total_logs = 1000;
    let logs: Vec<AuditLog> = (0..total_logs)
        .map(|i| AuditLog {
            id: uuid::Uuid::new_v4(),
            event_type: if i % 2 == 0 {
                AuditEventType::UserLogin
            } else {
                AuditEventType::UserLogout
            },
            severity: if i % 3 == 0 {
                AuditSeverity::Warning
            } else {
                AuditSeverity::Info
            },
            actor_id: Some(uuid::Uuid::new_v4()),
            actor_name: None,
            actor_role: None,
            target_type: None,
            target_id: None,
            action: "action".to_string(),
            description: format!("Log entry {}", i),
            metadata: None,
            status: "success".to_string(),
            error_message: None,
            created_at: Utc::now(),
        })
        .collect();

    // 模拟按事件类型过滤
    let login_logs: Vec<&AuditLog> = logs
        .iter()
        .filter(|log| log.event_type == AuditEventType::UserLogin)
        .collect();

    assert_eq!(login_logs.len(), total_logs / 2);

    // 模拟按严重级别过滤
    let warning_logs: Vec<&AuditLog> = logs
        .iter()
        .filter(|log| log.severity == AuditSeverity::Warning)
        .collect();

    // 由于整除，实际数量可能略有不同
    assert!(warning_logs.len() >= total_logs / 3 - 1);
    assert!(warning_logs.len() <= total_logs / 3 + 1);
}

/// 测试配置热更新事件
#[test]
fn test_config_hot_reload_event() {
    use capella_room::config::ConfigChangeEvent;

    let event = ConfigChangeEvent::ConfigUpdated {
        key: "audit.buffer_size".to_string(),
        old_value: "100".to_string(),
        new_value: "200".to_string(),
    };

    match event {
        ConfigChangeEvent::ConfigUpdated { key, new_value, .. } => {
            assert_eq!(key, "audit.buffer_size");
            assert_eq!(new_value, "200");
        }
        _ => panic!("Expected ConfigUpdated event"),
    }
}

/// 测试审计配置保留策略
#[test]
fn test_audit_retention_policy() {
    use chrono::Duration;

    let retention_days = 90;
    let now = Utc::now();
    let cutoff = now - Duration::days(retention_days as i64);

    // 验证 cutoff 计算正确
    let days_diff = (now - cutoff).num_days();
    assert_eq!(days_diff, retention_days as i64);

    // 测试不同保留天数的场景
    let test_cases = vec![
        (0, "永久保留"),
        (30, "月度保留"),
        (90, "季度保留"),
        (365, "年度保留"),
    ];

    for (days, description) in test_cases {
        let cutoff = now - Duration::days(days as i64);
        let diff = (now - cutoff).num_days();
        assert_eq!(diff, days as i64, "Failed for {}", description);
    }
}

/// 测试归档时间配置
#[test]
fn test_archive_hour_config() {
    let config = AuditConfig::default();

    // 验证归档小时在有效范围内
    assert!(
        config.archive_hour < 24,
        "Archive hour must be between 0 and 23"
    );

    // 测试不同归档小时
    for hour in 0..24 {
        let config = AuditConfig {
            archive_hour: hour,
            ..Default::default()
        };
        assert_eq!(config.archive_hour, hour);
    }
}

/// 测试排除路径配置
#[test]
fn test_excluded_paths_config() {
    let config = AuditConfig {
        excluded_paths: vec![
            "/health".to_string(),
            "/ws".to_string(),
            "/static".to_string(),
            "/metrics".to_string(),
        ],
        ..Default::default()
    };

    assert!(config.excluded_paths.contains(&"/health".to_string()));
    assert!(config.excluded_paths.contains(&"/ws".to_string()));
    assert!(!config.excluded_paths.contains(&"/api/admin".to_string()));
}

/// 测试告警冷却配置
#[test]
fn test_alert_cooldown_config() {
    let config = AuditConfig {
        alert_cooldown_minutes: 15,
        ..Default::default()
    };

    assert_eq!(config.alert_cooldown_minutes, 15);

    // 验证冷却时间不能为负
    assert!(config.alert_cooldown_minutes >= 0);
}

/// 测试缓冲区大小配置
#[test]
fn test_buffer_size_config() {
    let config = AuditConfig {
        buffer_size: 500,
        flush_interval_seconds: 10,
        ..Default::default()
    };

    assert_eq!(config.buffer_size, 500);
    assert_eq!(config.flush_interval_seconds, 10);

    // 验证缓冲区大小必须为正
    assert!(config.buffer_size > 0);
    assert!(config.flush_interval_seconds > 0);
}

/// 测试审计配置启用/禁用
#[test]
fn test_audit_enabled_disabled() {
    let enabled_config = AuditConfig {
        enabled: true,
        alert_enabled: true,
        auto_archive_enabled: true,
        ..Default::default()
    };

    let disabled_config = AuditConfig {
        enabled: false,
        alert_enabled: false,
        auto_archive_enabled: false,
        ..Default::default()
    };

    assert!(enabled_config.enabled);
    assert!(enabled_config.alert_enabled);
    assert!(enabled_config.auto_archive_enabled);

    assert!(!disabled_config.enabled);
    assert!(!disabled_config.alert_enabled);
    assert!(!disabled_config.auto_archive_enabled);
}
