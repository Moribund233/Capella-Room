//! 阶段 8.4：告警系统测试
//!
//! 测试告警规则引擎、告警通知和告警管理功能

use chrono::Timelike;
use chrono::Utc;

use seredeli_room::models::audit::{AuditEventType, AuditSeverity};
use seredeli_room::services::alert_engine::{
    create_abnormal_login_rule, create_brute_force_rule, create_unauthorized_access_rule,
    AlertCondition, ConditionType,
};

/// 测试告警条件创建
#[test]
fn test_alert_condition_creation() {
    let condition = create_brute_force_rule();
    assert_eq!(condition.condition_type, ConditionType::Frequency);
    assert_eq!(condition.threshold, Some(5));
    assert_eq!(condition.time_window_minutes, Some(5));
}

/// 测试告警条件序列化
#[test]
fn test_alert_condition_serialization() {
    let condition = AlertCondition {
        condition_type: ConditionType::Frequency,
        threshold: Some(5),
        time_window_minutes: Some(5),
        pattern: None,
        event_types: Some(vec![AuditEventType::SystemLoginFailure]),
        severity: Some(AuditSeverity::Warning),
    };

    let json = serde_json::to_value(&condition).unwrap();
    assert!(json.get("condition_type").is_some());
    assert!(json.get("threshold").is_some());

    let deserialized: AlertCondition = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.condition_type, ConditionType::Frequency);
    assert_eq!(deserialized.threshold, Some(5));
}

/// 测试模式匹配 - 异常时间检测
#[test]
fn test_abnormal_time_pattern() {
    // 创建凌晨时间的日志
    let early_morning = Utc::now()
        .with_hour(3)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    let hour = early_morning.hour();
    let is_abnormal = !(6..=23).contains(&hour);
    assert!(is_abnormal, "3 AM should be considered abnormal time");

    // 创建正常时间的日志
    let normal_time = Utc::now()
        .with_hour(10)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    let hour = normal_time.hour();
    let is_abnormal = !(6..=23).contains(&hour);
    assert!(!is_abnormal, "10 AM should be considered normal time");
}

/// 测试告警条件类型
#[test]
fn test_condition_types() {
    // 阈值条件
    let threshold_condition = AlertCondition {
        condition_type: ConditionType::Threshold,
        threshold: Some(10),
        time_window_minutes: Some(60),
        pattern: None,
        event_types: Some(vec![AuditEventType::UserLogin]),
        severity: None,
    };
    assert_eq!(threshold_condition.condition_type, ConditionType::Threshold);

    // 频率条件
    let frequency_condition = create_brute_force_rule();
    assert_eq!(frequency_condition.condition_type, ConditionType::Frequency);

    // 模式条件
    let pattern_condition = create_abnormal_login_rule();
    assert_eq!(pattern_condition.condition_type, ConditionType::Pattern);
}

/// 测试告警严重级别优先级
#[test]
fn test_severity_priority() {
    assert_eq!(AuditSeverity::Info.priority(), 0);
    assert_eq!(AuditSeverity::Warning.priority(), 1);
    assert_eq!(AuditSeverity::Error.priority(), 2);
    assert_eq!(AuditSeverity::Critical.priority(), 3);

    // 验证优先级排序
    assert!(AuditSeverity::Critical.priority() > AuditSeverity::Error.priority());
    assert!(AuditSeverity::Error.priority() > AuditSeverity::Warning.priority());
    assert!(AuditSeverity::Warning.priority() > AuditSeverity::Info.priority());
}

/// 测试事件类型分类
#[test]
fn test_event_type_categories() {
    assert_eq!(AuditEventType::UserLogin.category(), "user");
    assert_eq!(AuditEventType::RoomCreate.category(), "room");
    assert_eq!(AuditEventType::MessageSend.category(), "message");
    assert_eq!(AuditEventType::AdminUserDelete.category(), "admin");
    assert_eq!(AuditEventType::SystemLoginFailure.category(), "system");
}

/// 测试事件类型默认严重级别
#[test]
fn test_event_type_default_severity() {
    assert_eq!(
        AuditEventType::SystemLoginFailure.default_severity(),
        AuditSeverity::Warning
    );
    assert_eq!(
        AuditEventType::SystemUnauthorizedAccess.default_severity(),
        AuditSeverity::Error
    );
    assert_eq!(
        AuditEventType::UserLogin.default_severity(),
        AuditSeverity::Info
    );
}

/// 测试预设规则创建
#[test]
fn test_preset_rules() {
    // 暴力破解检测规则
    let brute_force = create_brute_force_rule();
    assert_eq!(brute_force.condition_type, ConditionType::Frequency);
    assert_eq!(brute_force.threshold, Some(5));
    assert_eq!(brute_force.time_window_minutes, Some(5));
    assert!(brute_force.event_types.is_some());

    // 异常登录检测规则
    let abnormal_login = create_abnormal_login_rule();
    assert_eq!(abnormal_login.condition_type, ConditionType::Pattern);
    assert_eq!(abnormal_login.pattern, Some("abnormal_time".to_string()));

    // 越权访问检测规则
    let unauthorized = create_unauthorized_access_rule();
    assert_eq!(unauthorized.condition_type, ConditionType::Threshold);
    assert_eq!(unauthorized.threshold, Some(1));
}

/// 测试 AlertStatus 枚举
#[test]
fn test_alert_status() {
    use seredeli_room::models::audit::AlertStatus;

    let new_status = AlertStatus::New;
    let acknowledged_status = AlertStatus::Acknowledged;
    let resolved_status = AlertStatus::Resolved;
    let ignored_status = AlertStatus::Ignored;

    // 验证序列化
    assert_eq!(serde_json::to_string(&new_status).unwrap(), "\"new\"");
    assert_eq!(
        serde_json::to_string(&acknowledged_status).unwrap(),
        "\"acknowledged\""
    );
    assert_eq!(
        serde_json::to_string(&resolved_status).unwrap(),
        "\"resolved\""
    );
    assert_eq!(
        serde_json::to_string(&ignored_status).unwrap(),
        "\"ignored\""
    );
}
