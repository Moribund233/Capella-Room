-- 迁移: 设备禁用功能
-- 日期: 2026-05-06
-- 说明:
--   1. 为 user_sessions 表添加 is_blocked 字段，支持用户禁用特定设备
--   2. 被禁用的设备无法使用旧 Token 登录，需要重新认证
--   3. 补充 user_settings 表中 privacy_settings 缺失的 single_device_login 默认值

-- ============================================
-- 1. 补充迁移5缺失的默认值（针对已执行旧版迁移5的数据库）
-- ============================================

-- 为现有用户的 privacy_settings 添加 single_device_login 字段（如果不存在）
UPDATE user_settings
SET privacy_settings = jsonb_set(
    privacy_settings,
    '{single_device_login}',
    'false'::jsonb,
    true  -- 如果字段已存在则不覆盖
)
WHERE privacy_settings->>'single_device_login' IS NULL;

COMMENT ON COLUMN user_settings.privacy_settings IS '隐私设置：在线状态可见性、资料可见性、陌生人消息、单设备登录等';

-- ============================================
-- 2. 为 user_sessions 添加设备禁用标记
-- ============================================

ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS is_blocked BOOLEAN DEFAULT false;

COMMENT ON COLUMN user_sessions.is_blocked IS '设备是否被用户禁用：true 表示该设备被禁用，无法使用旧 Token 登录';

-- ============================================
-- 3. 创建索引优化查询
-- ============================================

-- 查询用户被禁用的设备
CREATE INDEX IF NOT EXISTS idx_user_sessions_blocked ON user_sessions(user_id, is_blocked) WHERE is_blocked = true;

-- ============================================
-- 4. 更新现有数据
-- ============================================

-- 将现有数据的 is_blocked 设置为 false（默认不禁用）
UPDATE user_sessions SET is_blocked = false WHERE is_blocked IS NULL;

-- ============================================
-- 5. 添加约束确保数据一致性
-- ============================================

-- 被禁用的会话不能是当前会话
-- 注意：这是一个软约束，应用层需要确保逻辑正确
COMMENT ON TABLE user_sessions IS '用户登录会话表，存储活跃的设备登录信息，用于账号安全管理。is_blocked 字段用于标记被用户禁用的设备';
