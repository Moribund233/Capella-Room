-- 迁移: 用户设置体系与账号安全
-- 日期: 2026-05-06
-- 说明:
--   1. 弃用 user_ui_configs 表中的 app_config 字段（应用配置改为从 ui.ts 读取）
--   2. 创建用户设置表，支持通知、隐私、消息等个性化设置
--   3. 创建账号安全相关表（登录会话、登录历史）

-- ============================================
-- 1. 删除 user_ui_configs 表中的 app_config 字段
-- ============================================

-- 删除 app_config 字段（应用配置改为从 ui.ts 配置文件读取）
ALTER TABLE user_ui_configs DROP COLUMN IF EXISTS app_config;

-- ============================================
-- 2. 创建用户设置表
-- ============================================

CREATE TABLE IF NOT EXISTS user_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- 通知设置 (JSONB)
    notification_settings JSONB DEFAULT '{
        "private_message": true,
        "mentioned": true,
        "room_invitation": true,
        "system_notification": true,
        "file_upload_complete": true,
        "sound_enabled": true,
        "desktop_notification": true
    }'::jsonb,

    -- 隐私设置 (JSONB)
    privacy_settings JSONB DEFAULT '{
        "online_status_visibility": "everyone",
        "profile_visibility": "everyone",
        "allow_stranger_message": true,
        "allow_room_invitation": true
    }'::jsonb,

    -- 消息设置 (JSONB)
    message_settings JSONB DEFAULT '{
        "message_preview": true,
        "read_receipt": true,
        "typing_indicator": true,
        "do_not_disturb": false
    }'::jsonb,

    -- 元数据
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- 每个用户只有一条设置记录
    CONSTRAINT unique_user_settings UNIQUE (user_id)
);

-- 创建更新时间触发器
CREATE OR REPLACE FUNCTION update_user_settings_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_user_settings_updated_at
    BEFORE UPDATE ON user_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_user_settings_updated_at();

-- 创建索引
CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);

-- 添加注释
COMMENT ON TABLE user_settings IS '用户个性化设置表，存储通知、隐私、消息等偏好设置';
COMMENT ON COLUMN user_settings.notification_settings IS '通知设置：私信、@提及、房间邀请、系统通知等开关';
COMMENT ON COLUMN user_settings.privacy_settings IS '隐私设置：在线状态可见性、资料可见性、陌生人消息等';
COMMENT ON COLUMN user_settings.message_settings IS '消息设置：消息预览、已读回执、输入状态、免打扰等';

-- ============================================
-- 3. 创建用户登录会话表（账号安全）
-- ============================================

CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- 会话信息
    session_token_hash VARCHAR(64) NOT NULL,  -- SHA-256 哈希值，用于验证
    device_name VARCHAR(100),                  -- 设备名称（如 "iPhone 15", "Windows PC"）
    device_type VARCHAR(20),                   -- 设备类型：mobile, tablet, desktop, unknown
    ip_address INET,                           -- IP 地址
    user_agent TEXT,                           -- User-Agent 字符串
    location_info JSONB,                       -- 位置信息（国家、城市等）

    -- 状态
    is_current BOOLEAN DEFAULT false,          -- 是否为当前会话
    is_active BOOLEAN DEFAULT true,            -- 是否活跃

    -- 时间戳
    last_active_at TIMESTAMPTZ DEFAULT NOW(),  -- 最后活跃时间
    expires_at TIMESTAMPTZ NOT NULL,           -- 过期时间
    created_at TIMESTAMPTZ DEFAULT NOW(),      -- 创建时间

    -- 约束
    CONSTRAINT unique_session_token_hash UNIQUE (session_token_hash)
);

-- 创建索引
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_user_active ON user_sessions(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);

-- 添加注释
COMMENT ON TABLE user_sessions IS '用户登录会话表，存储活跃的设备登录信息，用于账号安全管理';
COMMENT ON COLUMN user_sessions.session_token_hash IS '会话令牌哈希值，用于验证会话有效性';
COMMENT ON COLUMN user_sessions.device_type IS '设备类型：mobile, tablet, desktop, unknown';
COMMENT ON COLUMN user_sessions.is_current IS '标记当前正在使用的会话';

-- ============================================
-- 4. 创建登录历史表（账号安全）
-- ============================================

CREATE TABLE IF NOT EXISTS login_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- 登录信息
    ip_address INET NOT NULL,                  -- IP 地址
    device_info JSONB,                         -- 设备信息（名称、类型、User-Agent等）
    location_info JSONB,                       -- 位置信息

    -- 登录结果
    login_status VARCHAR(20) NOT NULL,         -- 状态：success, failed, blocked
    failure_reason VARCHAR(100),               -- 失败原因（如 invalid_password, account_locked）

    -- 安全相关
    is_suspicious BOOLEAN DEFAULT false,       -- 是否可疑登录
    risk_level VARCHAR(20) DEFAULT 'low',      -- 风险等级：low, medium, high

    -- 时间戳
    created_at TIMESTAMPTZ DEFAULT NOW(),      -- 登录时间

    -- 约束
    CONSTRAINT valid_login_status CHECK (login_status IN ('success', 'failed', 'blocked')),
    CONSTRAINT valid_risk_level CHECK (risk_level IN ('low', 'medium', 'high'))
);

-- 创建索引
CREATE INDEX idx_login_history_user_id ON login_history(user_id);
CREATE INDEX idx_login_history_user_created ON login_history(user_id, created_at DESC);
CREATE INDEX idx_login_history_created_at ON login_history(created_at DESC);
CREATE INDEX idx_login_history_suspicious ON login_history(user_id, is_suspicious) WHERE is_suspicious = true;

-- 添加注释
COMMENT ON TABLE login_history IS '用户登录历史表，记录所有登录尝试，用于安全审计和异常检测';
COMMENT ON COLUMN login_history.login_status IS '登录状态：success(成功), failed(失败), blocked(被阻止)';
COMMENT ON COLUMN login_history.is_suspicious IS '标记是否为可疑登录（异地、异常设备等）';
COMMENT ON COLUMN login_history.risk_level IS '风险等级：low(低), medium(中), high(高)';

-- ============================================
-- 5. 创建自动清理过期会话的函数和任务
-- ============================================

-- 清理过期会话的函数
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM user_sessions
    WHERE expires_at < NOW() OR is_active = false;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_expired_sessions() IS '清理过期的用户会话记录';

-- ============================================
-- 6. 创建登录历史归档函数（可选，用于性能优化）
-- ============================================

-- 将旧登录历史归档到历史表（保留最近90天的记录）
CREATE OR REPLACE FUNCTION archive_old_login_history()
RETURNS INTEGER AS $$
DECLARE
    archived_count INTEGER;
BEGIN
    -- 注意：这里只是示例，实际项目中可能需要创建 login_history_archive 表
    -- 或者使用分区表来管理历史数据

    -- 删除90天前的非可疑登录记录（保留可疑记录用于审计）
    DELETE FROM login_history
    WHERE created_at < NOW() - INTERVAL '90 days'
      AND is_suspicious = false;

    GET DIAGNOSTICS archived_count = ROW_COUNT;
    RETURN archived_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION archive_old_login_history() IS '归档90天前的非可疑登录历史记录';

-- ============================================
-- 7. 验证迁移
-- ============================================

-- 检查表是否创建成功
SELECT 'user_settings table created' AS status WHERE EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'user_settings'
);

SELECT 'user_sessions table created' AS status WHERE EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'user_sessions'
);

SELECT 'login_history table created' AS status WHERE EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'login_history'
);
