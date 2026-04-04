-- 阶段 8：配置体系和管理员系统
-- 创建用户角色枚举
CREATE TYPE user_role AS ENUM ('user', 'admin', 'super_admin');

-- 添加用户角色字段
ALTER TABLE users ADD COLUMN role user_role DEFAULT 'user';

-- 创建系统配置表
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

-- 创建索引
CREATE INDEX idx_system_configs_category ON system_configs(category);
CREATE INDEX idx_system_configs_editable ON system_configs(is_editable);

-- 创建更新时间戳触发器
CREATE TRIGGER update_system_configs_updated_at BEFORE UPDATE ON system_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 插入默认配置
INSERT INTO system_configs (key, value, value_type, description, category, is_editable, is_hot_reloadable) VALUES
    -- 服务器配置（不可热更新，需重启）
    ('server.host', '0.0.0.0', 'string', '服务器监听地址', 'server', false, false),
    ('server.port', '3000', 'int', '服务器端口', 'server', false, false),
    
    -- JWT 配置
    ('jwt.expiration_hours', '24', 'int', 'JWT Token 过期时间（小时）', 'security', true, true),
    
    -- 文件上传配置
    ('upload.max_file_size', '10485760', 'int', '最大文件大小（字节）', 'upload', true, true),
    ('upload.base_url', '/uploads', 'string', '文件访问基础URL路径', 'upload', true, true),
    
    -- 速率限制配置
    ('rate_limit.enabled', 'true', 'bool', '是否启用速率限制', 'rate_limit', true, true),
    ('rate_limit.default_requests', '100', 'int', '默认限制：时间窗口内的最大请求数', 'rate_limit', true, true),
    ('rate_limit.default_window_secs', '60', 'int', '默认限制：时间窗口（秒）', 'rate_limit', true, true),
    ('rate_limit.auth_requests', '5', 'int', '认证接口限制', 'rate_limit', true, true),
    ('rate_limit.auth_window_secs', '60', 'int', '认证接口时间窗口（秒）', 'rate_limit', true, true),
    ('rate_limit.message_requests', '30', 'int', '消息接口限制', 'rate_limit', true, true),
    ('rate_limit.message_window_secs', '60', 'int', '消息接口时间窗口（秒）', 'rate_limit', true, true),
    ('rate_limit.room_requests', '20', 'int', '房间接口限制', 'rate_limit', true, true),
    ('rate_limit.room_window_secs', '60', 'int', '房间接口时间窗口（秒）', 'rate_limit', true, true),
    
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

-- 为用户表添加角色索引
CREATE INDEX idx_users_role ON users(role);
