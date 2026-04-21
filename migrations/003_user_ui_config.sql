-- 用户 UI 配置表
-- 支持云端同步用户界面偏好设置

-- 创建用户 UI 配置表
CREATE TABLE user_ui_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- 应用配置 (JSON 格式，存储 name, logo, version)
    app_config JSONB DEFAULT NULL,
    
    -- 主题配置 (JSON 格式，存储 name)
    theme_config JSONB DEFAULT NULL,
    
    -- 侧边栏配置 (JSON 格式，存储 items 数组)
    sidebar_config JSONB DEFAULT NULL,
    
    -- QuickBar 配置 (JSON 格式，存储 items 数组)
    quickbar_config JSONB DEFAULT NULL,
    
    -- Dock 配置 (JSON 格式)
    dock_config JSONB DEFAULT NULL,
    
    -- 元数据
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- 每个用户只有一条配置记录
    CONSTRAINT unique_user_ui_config UNIQUE (user_id)
);

-- 创建更新时间触发器
CREATE OR REPLACE FUNCTION update_user_ui_config_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_user_ui_config_updated_at
    BEFORE UPDATE ON user_ui_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_user_ui_config_updated_at();

-- 创建索引
CREATE INDEX idx_user_ui_configs_user_id ON user_ui_configs(user_id);

-- 添加注释
COMMENT ON TABLE user_ui_configs IS '用户 UI 配置表，存储用户界面偏好设置';
COMMENT ON COLUMN user_ui_configs.app_config IS '应用配置，包含 name, logo, version';
COMMENT ON COLUMN user_ui_configs.theme_config IS '主题配置，包含 name (light/dark)';
COMMENT ON COLUMN user_ui_configs.sidebar_config IS '侧边栏配置，包含 items 数组';
COMMENT ON COLUMN user_ui_configs.quickbar_config IS 'QuickBar 配置，包含 items 数组';
COMMENT ON COLUMN user_ui_configs.dock_config IS 'Dock 栏配置';
