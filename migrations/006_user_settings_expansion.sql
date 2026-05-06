-- 迁移: 用户设置扩展 + 房间级设置
-- 日期: 2026-05-06
-- 说明:
--   1. 为 user_settings 表添加语言、无障碍、媒体设置字段
--   2. 创建用户房间设置表，支持独立通知偏好和置顶

-- ============================================
-- 1. 为 user_settings 添加语言与地区设置
-- ============================================

ALTER TABLE user_settings ADD COLUMN IF NOT EXISTS language_settings JSONB DEFAULT '{
    "language": "zh-CN",
    "timezone": "Asia/Shanghai",
    "time_format": "24h",
    "date_format": "YYYY-MM-DD",
    "first_day_of_week": "monday"
}'::jsonb;

COMMENT ON COLUMN user_settings.language_settings IS '语言与地区设置：界面语言、时区、时间日期格式';

-- ============================================
-- 2. 为 user_settings 添加无障碍设置
-- ============================================

ALTER TABLE user_settings ADD COLUMN IF NOT EXISTS accessibility_settings JSONB DEFAULT '{
    "font_size": "medium",
    "reduce_motion": false,
    "high_contrast": false,
    "dense_mode": false
}'::jsonb;

COMMENT ON COLUMN user_settings.accessibility_settings IS '无障碍设置：字体大小、减少动效、高对比度、紧凑模式';

-- ============================================
-- 3. 为 user_settings 添加媒体与存储设置
-- ============================================

ALTER TABLE user_settings ADD COLUMN IF NOT EXISTS media_settings JSONB DEFAULT '{
    "auto_download_media": true,
    "save_media_gallery": false,
    "image_quality": "high",
    "auto_play_video": true,
    "auto_play_audio": false
}'::jsonb;

COMMENT ON COLUMN user_settings.media_settings IS '媒体与存储设置：自动下载、保存到相册、图片质量、自动播放';

-- ============================================
-- 4. 创建用户房间设置表
-- ============================================

CREATE TABLE IF NOT EXISTS user_room_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,

    -- 通知偏好
    is_muted BOOLEAN DEFAULT false,                  -- 静音（不推送该房间的通知）
    notification_preference VARCHAR(20) DEFAULT 'all', -- all / mention_only / muted

    -- 列表偏好
    is_pinned BOOLEAN DEFAULT false,                 -- 置顶

    -- 自定义
    custom_name VARCHAR(100),                        -- 用户侧自定义房间名称
    custom_color VARCHAR(7),                         -- 自定义颜色（十六进制）

    -- 元数据
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- 每个用户每个房间只有一条配置
    CONSTRAINT unique_user_room UNIQUE (user_id, room_id),
    -- 通知偏好取值约束
    CONSTRAINT check_notification_preference CHECK (
        notification_preference IN ('all', 'mention_only', 'muted')
    )
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_user_room_settings_user ON user_room_settings(user_id);
CREATE INDEX IF NOT EXISTS idx_user_room_settings_room ON user_room_settings(room_id);
CREATE INDEX IF NOT EXISTS idx_user_room_settings_pinned ON user_room_settings(user_id, is_pinned)
    WHERE is_pinned = true;
CREATE INDEX IF NOT EXISTS idx_user_room_settings_muted ON user_room_settings(user_id, is_muted)
    WHERE is_muted = true;

-- 更新时间触发器
CREATE OR REPLACE FUNCTION update_user_room_settings_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_update_user_room_settings_updated_at ON user_room_settings;
CREATE TRIGGER trigger_update_user_room_settings_updated_at
    BEFORE UPDATE ON user_room_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_user_room_settings_updated_at();

-- 添加注释
COMMENT ON TABLE user_room_settings IS '用户房间设置表，每个用户对每个房间的独立偏好设置';
COMMENT ON COLUMN user_room_settings.is_muted IS '是否静音该房间（静音后不推送通知）';
COMMENT ON COLUMN user_room_settings.notification_preference IS '通知偏好：all(全部通知), mention_only(仅@提及), muted(不通知)';
COMMENT ON COLUMN user_room_settings.is_pinned IS '是否置顶该房间';
COMMENT ON COLUMN user_room_settings.custom_name IS '用户侧自定义房间显示名称';
COMMENT ON COLUMN user_room_settings.custom_color IS '房间自定义颜色标记';

-- ============================================
-- 5. 验证迁移
-- ============================================

SELECT 'language_settings column added' AS status
WHERE EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'user_settings' AND column_name = 'language_settings'
);

SELECT 'accessibility_settings column added' AS status
WHERE EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'user_settings' AND column_name = 'accessibility_settings'
);

SELECT 'media_settings column added' AS status
WHERE EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'user_settings' AND column_name = 'media_settings'
);

SELECT 'user_room_settings table created' AS status
WHERE EXISTS (
    SELECT 1 FROM information_schema.tables WHERE table_name = 'user_room_settings'
);
