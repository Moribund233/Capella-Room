-- 创建审计事件类型枚举
CREATE TYPE audit_event_type AS ENUM (
    -- 用户事件
    'user_login',
    'user_logout',
    'user_register',
    'user_password_change',
    'user_profile_update',
    -- 房间事件
    'room_create',
    'room_delete',
    'room_member_add',
    'room_member_remove',
    'room_member_role_change',
    -- 消息事件
    'message_send',
    'message_edit',
    'message_delete',
    'message_report',
    -- 管理员事件
    'admin_user_disable',
    'admin_user_role_change',
    'admin_user_delete',
    'admin_room_delete',
    'admin_message_delete',
    'admin_config_update',
    -- 系统事件
    'system_login_failure',
    'system_unauthorized_access',
    'system_rate_limit_triggered'
);

-- 创建审计严重级别枚举
CREATE TYPE audit_severity AS ENUM (
    'info',
    'warning',
    'error',
    'critical'
);

-- 创建告警状态枚举
CREATE TYPE alert_status AS ENUM (
    'new',
    'acknowledged',
    'resolved',
    'ignored'
);

-- 创建审计日志表（按月份分区）
-- 注意：分区表的主键必须包含分区列，因此使用 (id, created_at) 作为复合主键
CREATE TABLE audit_logs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    event_type audit_event_type NOT NULL,
    severity audit_severity NOT NULL DEFAULT 'info',
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_role user_role,
    target_type VARCHAR(50),
    target_id UUID,
    action VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    metadata JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'success',
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- 创建当前月份的分区表
CREATE TABLE audit_logs_current PARTITION OF audit_logs
    FOR VALUES FROM ('2026-01-01') TO ('2027-01-01');

-- 创建审计告警表
CREATE TABLE audit_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID,
    alert_type VARCHAR(100) NOT NULL,
    severity audit_severity NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    related_logs UUID[],
    source_ip INET,
    affected_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status alert_status NOT NULL DEFAULT 'new',
    acknowledged_by UUID REFERENCES users(id) ON DELETE SET NULL,
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 创建告警规则表
CREATE TABLE audit_alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    event_type audit_event_type,
    condition JSONB NOT NULL,
    severity audit_severity NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    cooldown_minutes INT NOT NULL DEFAULT 60,
    notify_admins BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 创建审计日志索引
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_target_id ON audit_logs(target_id);
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity);
CREATE INDEX idx_audit_logs_status ON audit_logs(status);

-- 创建审计告警索引
CREATE INDEX idx_audit_alerts_status ON audit_alerts(status);
CREATE INDEX idx_audit_alerts_created_at ON audit_alerts(created_at DESC);
CREATE INDEX idx_audit_alerts_severity ON audit_alerts(severity);
CREATE INDEX idx_audit_alerts_affected_user ON audit_alerts(affected_user_id);

-- 创建告警规则索引
CREATE INDEX idx_alert_rules_enabled ON audit_alert_rules(enabled);
CREATE INDEX idx_alert_rules_event_type ON audit_alert_rules(event_type);

-- 添加注释
COMMENT ON TABLE audit_logs IS '审计日志表，记录所有关键操作';
COMMENT ON COLUMN audit_logs.event_type IS '事件类型';
COMMENT ON COLUMN audit_logs.severity IS '严重级别';
COMMENT ON COLUMN audit_logs.actor_id IS '操作者用户ID';
COMMENT ON COLUMN audit_logs.actor_role IS '操作者角色';
COMMENT ON COLUMN audit_logs.target_type IS '目标类型（user/room/message/file）';
COMMENT ON COLUMN audit_logs.target_id IS '目标ID';
COMMENT ON COLUMN audit_logs.action IS '操作类型';
COMMENT ON COLUMN audit_logs.metadata IS '额外元数据（IP、User-Agent等）';

COMMENT ON TABLE audit_alerts IS '安全告警表';
COMMENT ON COLUMN audit_alerts.rule_id IS '触发规则的ID';
COMMENT ON COLUMN audit_alerts.related_logs IS '关联的审计日志ID列表';

COMMENT ON TABLE audit_alert_rules IS '告警规则配置表';
COMMENT ON COLUMN audit_alert_rules.condition IS '规则条件（JSON格式）';
COMMENT ON COLUMN audit_alert_rules.cooldown_minutes IS '告警冷却时间（分钟）';

-- 创建更新时间触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 为告警表添加更新时间触发器
CREATE TRIGGER update_audit_alerts_updated_at
    BEFORE UPDATE ON audit_alerts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 为告警规则表添加更新时间触发器
CREATE TRIGGER update_audit_alert_rules_updated_at
    BEFORE UPDATE ON audit_alert_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 插入默认告警规则
INSERT INTO audit_alert_rules (name, description, event_type, condition, severity, enabled, cooldown_minutes, notify_admins)
VALUES
    (
        '暴力破解检测',
        '检测短时间内多次登录失败',
        'system_login_failure',
        '{"condition_type": "frequency", "threshold": 5, "time_window_minutes": 5}'::jsonb,
        'critical',
        true,
        30,
        true
    ),
    (
        '异常登录检测',
        '检测来自异常IP的登录',
        'user_login',
        '{"condition_type": "pattern", "pattern": "suspicious_ip"}'::jsonb,
        'warning',
        true,
        60,
        true
    ),
    (
        '越权访问检测',
        '检测未授权访问尝试',
        'system_unauthorized_access',
        '{"condition_type": "threshold", "threshold": 1}'::jsonb,
        'error',
        true,
        15,
        true
    ),
    (
        '敏感操作监控',
        '监控管理员敏感操作',
        'admin_user_delete',
        '{"condition_type": "threshold", "threshold": 1}'::jsonb,
        'warning',
        true,
        5,
        true
    ),
    (
        '频率限制触发',
        '检测API频率限制触发',
        'system_rate_limit_triggered',
        '{"condition_type": "frequency", "threshold": 10, "time_window_minutes": 1}'::jsonb,
        'info',
        true,
        10,
        false
    );
