-- Seredeli Room 数据库初始化脚本
-- 整合所有开发阶段的数据库迁移
-- 阶段 1-8.6 完整数据库结构

-- ============================================
-- 阶段 1: 基础架构 - 枚举类型定义
-- ============================================

-- 用户状态枚举
CREATE TYPE user_status AS ENUM ('online', 'offline', 'away');

-- 成员角色枚举
CREATE TYPE member_role AS ENUM ('owner', 'admin', 'member');

-- 用户角色枚举（管理员系统）
CREATE TYPE user_role AS ENUM ('user', 'admin', 'super_admin');

-- 消息类型枚举
CREATE TYPE message_type AS ENUM ('text', 'image', 'file', 'system');

-- 文件分类枚举
CREATE TYPE file_category AS ENUM ('image', 'document', 'video', 'audio', 'other');

-- 文件用途枚举
CREATE TYPE file_usage_type AS ENUM ('avatar', 'message', 'room_cover', 'general');

-- 通知类型枚举
CREATE TYPE notification_type AS ENUM (
    'private_message',
    'mentioned',
    'room_invitation',
    'system_notification',
    'file_upload_complete',
    'config_reload_required',
    'pending_action'
);

-- 待办操作状态枚举
CREATE TYPE action_status AS ENUM ('pending', 'approved', 'rejected', 'snoozed');

-- 审计事件类型枚举
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
    'system_rate_limit_triggered',
    -- 审计系统事件
    'audit_query',
    'audit_export',
    'audit_stats_query',
    'alert_query',
    'alert_rule_update',
    'audit_cleanup'
);

-- 审计严重级别枚举
CREATE TYPE audit_severity AS ENUM ('info', 'warning', 'error', 'critical');

-- 告警状态枚举
CREATE TYPE alert_status AS ENUM ('new', 'acknowledged', 'resolved', 'ignored');

-- ============================================
-- 阶段 1: 基础架构 - 核心表
-- ============================================

-- 用户表
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    status user_status DEFAULT 'offline',
    is_active BOOLEAN DEFAULT TRUE,
    role user_role DEFAULT 'user',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 聊天室表
CREATE TABLE rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_private BOOLEAN DEFAULT FALSE,
    max_members INTEGER DEFAULT 100,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 聊天室成员表
CREATE TABLE room_members (
    room_id UUID REFERENCES rooms(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role member_role DEFAULT 'member',
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (room_id, user_id)
);

-- 消息表
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    message_type message_type DEFAULT 'text',
    reply_to UUID REFERENCES messages(id) ON DELETE SET NULL,
    is_deleted BOOLEAN DEFAULT FALSE,
    edit_count INTEGER NOT NULL DEFAULT 0,
    edited_at TIMESTAMPTZ,
    content_tsv TSVECTOR,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- 阶段 6.5: 文件上传与资源管理
-- ============================================

-- 文件资源表
CREATE TABLE file_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uploader_id UUID REFERENCES users(id) ON DELETE SET NULL,
    original_name VARCHAR(255) NOT NULL,
    storage_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_url VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_hash VARCHAR(64),
    category file_category NOT NULL,
    usage_type file_usage_type DEFAULT 'general',
    room_id UUID REFERENCES rooms(id) ON DELETE SET NULL,
    message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    is_deleted BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- 阶段 6: 消息编辑历史
-- ============================================

-- 消息编辑历史表
CREATE TABLE message_edits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    editor_id UUID NOT NULL REFERENCES users(id),
    old_content TEXT NOT NULL,
    new_content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 阶段 8: 配置体系和管理员系统
-- ============================================

-- 系统配置表
CREATE TABLE system_configs (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    value_type VARCHAR(20) DEFAULT 'string',
    description TEXT,
    category VARCHAR(50),
    is_editable BOOLEAN DEFAULT true,
    is_hot_reloadable BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- 阶段 4.6: 通知系统
-- ============================================

-- 通知表
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type notification_type NOT NULL,
    title VARCHAR(200),
    content TEXT NOT NULL,
    data JSONB,
    is_read BOOLEAN DEFAULT false,
    read_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    -- 待办通知相关字段
    requires_action BOOLEAN DEFAULT false,
    action_type VARCHAR(50),
    action_status action_status DEFAULT 'pending',
    action_deadline TIMESTAMPTZ,
    action_result JSONB,
    action_by UUID REFERENCES users(id),
    action_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- 阶段 8.4: 审计系统
-- ============================================

-- 审计日志表（按月份分区）
CREATE TABLE audit_logs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    event_type audit_event_type NOT NULL,
    severity audit_severity NOT NULL DEFAULT 'info',
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_name VARCHAR(50),
    actor_role user_role,
    target_type VARCHAR(50),
    target_id UUID,
    action VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    metadata JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'success',
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- 创建当前年份的分区表
CREATE TABLE audit_logs_current PARTITION OF audit_logs
    FOR VALUES FROM ('2026-01-01') TO ('2027-01-01');

-- 审计告警表
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
    acknowledged_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 告警规则表
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 索引创建
-- ============================================

-- 用户表索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_role ON users(role);

-- 聊天室表索引
CREATE INDEX idx_rooms_owner ON rooms(owner_id);

-- 聊天室成员表索引
CREATE INDEX idx_room_members_room ON room_members(room_id);
CREATE INDEX idx_room_members_user ON room_members(user_id);

-- 消息表索引
CREATE INDEX idx_messages_room ON messages(room_id);
CREATE INDEX idx_messages_sender ON messages(sender_id);
CREATE INDEX idx_messages_created_at ON messages(created_at);
CREATE INDEX idx_messages_room_created ON messages(room_id, created_at);
CREATE INDEX idx_messages_edited_at ON messages(edited_at) WHERE edited_at IS NOT NULL;
CREATE INDEX idx_messages_content_tsv ON messages USING GIN(content_tsv);

-- 文件资源表索引
CREATE INDEX idx_file_resources_uploader ON file_resources(uploader_id);
CREATE INDEX idx_file_resources_category ON file_resources(category);
CREATE INDEX idx_file_resources_hash ON file_resources(file_hash);
CREATE INDEX idx_file_resources_room ON file_resources(room_id);
CREATE INDEX idx_file_resources_message ON file_resources(message_id);
CREATE INDEX idx_file_resources_created_at ON file_resources(created_at);
CREATE INDEX idx_file_resources_usage_type ON file_resources(usage_type);

-- 消息编辑历史表索引
CREATE INDEX idx_message_edits_message_id ON message_edits(message_id);
CREATE INDEX idx_message_edits_created_at ON message_edits(created_at);

-- 系统配置表索引
CREATE INDEX idx_system_configs_category ON system_configs(category);
CREATE INDEX idx_system_configs_editable ON system_configs(is_editable);

-- 通知表索引
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, is_read) WHERE is_read = false;
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_type ON notifications(notification_type);
CREATE INDEX idx_notifications_action ON notifications(requires_action, action_status) WHERE requires_action = true AND action_status = 'pending';
CREATE INDEX idx_notifications_action_type ON notifications(action_type) WHERE requires_action = true;
CREATE INDEX idx_notifications_action_deadline ON notifications(action_deadline) WHERE requires_action = true AND action_status = 'pending';

-- 审计日志表索引
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_target_id ON audit_logs(target_id);
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity);
CREATE INDEX idx_audit_logs_status ON audit_logs(status);

-- 审计告警表索引
CREATE INDEX idx_audit_alerts_status ON audit_alerts(status);
CREATE INDEX idx_audit_alerts_created_at ON audit_alerts(created_at DESC);
CREATE INDEX idx_audit_alerts_severity ON audit_alerts(severity);
CREATE INDEX idx_audit_alerts_affected_user ON audit_alerts(affected_user_id);

-- 告警规则表索引
CREATE INDEX idx_alert_rules_enabled ON audit_alert_rules(enabled);
CREATE INDEX idx_alert_rules_event_type ON audit_alert_rules(event_type);

-- ============================================
-- 触发器函数和触发器
-- ============================================

-- 更新时间戳触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 消息全文搜索向量更新函数
CREATE OR REPLACE FUNCTION update_message_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.content_tsv := to_tsvector('simple', NEW.content);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 用户表更新时间戳触发器
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 聊天室表更新时间戳触发器
CREATE TRIGGER update_rooms_updated_at
    BEFORE UPDATE ON rooms
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 文件资源表更新时间戳触发器
CREATE TRIGGER update_file_resources_updated_at
    BEFORE UPDATE ON file_resources
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 系统配置表更新时间戳触发器
CREATE TRIGGER update_system_configs_updated_at
    BEFORE UPDATE ON system_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 消息全文搜索触发器
CREATE TRIGGER trigger_update_message_search
    BEFORE INSERT OR UPDATE OF content ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_message_search_vector();

-- 审计告警表更新时间戳触发器
CREATE TRIGGER update_audit_alerts_updated_at
    BEFORE UPDATE ON audit_alerts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 告警规则表更新时间戳触发器
CREATE TRIGGER update_audit_alert_rules_updated_at
    BEFORE UPDATE ON audit_alert_rules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- 默认数据插入
-- ============================================

-- 系统默认配置
INSERT INTO system_configs (key, value, value_type, description, category, is_editable, is_hot_reloadable) VALUES
    -- 服务器配置（不可热更新，需重启）
    ('server.host', '0.0.0.0', 'string', '服务器监听地址', 'server', false, false),
    ('server.port', '3000', 'int', '服务器端口', 'server', false, false),
    
    -- JWT 配置
    ('jwt.expiration_hours', '24', 'int', 'JWT Token 过期时间（小时）', 'security', true, true),
    
    -- 文件上传配置
    ('upload.max_file_size', '10485760', 'int', '最大文件大小（字节）', 'upload', true, true),
    ('upload.base_url', '/uploads', 'string', '文件访问基础URL路径', 'upload', true, true),
    
    -- WebSocket 配置
    ('websocket.heartbeat_interval_secs', '30', 'int', 'WebSocket 心跳间隔（秒）', 'websocket', true, true),
    ('websocket.heartbeat_timeout_secs', '90', 'int', 'WebSocket 心跳超时（秒）', 'websocket', true, true),
    ('websocket.auth_timeout_secs', '30', 'int', 'WebSocket 认证超时（秒）', 'websocket', true, true),
    ('websocket.message_buffer_size', '100', 'int', 'WebSocket 消息缓冲区大小', 'websocket', true, true),
    
    -- 日志配置
    ('logging.level', 'info', 'string', '日志级别', 'logging', true, true),
    ('logging.structured', 'true', 'bool', '是否启用结构化日志', 'logging', true, true),
    
    -- 系统配置
    ('system.name', 'Seredeli Room', 'string', '系统名称', 'system', true, true),
    ('system.version', '1.0.0', 'string', '系统版本', 'system', false, false),
    ('system.maintenance_mode', 'false', 'bool', '维护模式', 'system', true, true),
    ('system.maintenance_message', 'System is under maintenance, please try again later.', 'string', '维护模式提示信息', 'system', true, true)
ON CONFLICT (key) DO NOTHING;

-- 默认告警规则
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

-- ============================================
-- 表注释
-- ============================================

COMMENT ON TABLE users IS '用户表，存储用户基本信息';
COMMENT ON TABLE rooms IS '聊天室表，存储房间信息';
COMMENT ON TABLE room_members IS '聊天室成员表，存储用户与房间的关联关系';
COMMENT ON TABLE messages IS '消息表，存储聊天消息';
COMMENT ON TABLE file_resources IS '文件资源表，存储上传文件的元数据';
COMMENT ON TABLE message_edits IS '消息编辑历史表，存储消息编辑记录';
COMMENT ON TABLE system_configs IS '系统配置表，存储可动态配置的系统参数';
COMMENT ON TABLE notifications IS '用户通知表，存储离线通知和通知历史';
COMMENT ON TABLE audit_logs IS '审计日志表，记录所有关键操作';
COMMENT ON TABLE audit_alerts IS '安全告警表';
COMMENT ON TABLE audit_alert_rules IS '告警规则配置表';
