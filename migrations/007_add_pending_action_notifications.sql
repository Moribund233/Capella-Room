-- 扩展通知类型枚举，添加待办通知类型
ALTER TYPE notification_type ADD VALUE IF NOT EXISTS 'config_reload_required';
ALTER TYPE notification_type ADD VALUE IF NOT EXISTS 'pending_action';

-- 创建待办操作状态枚举（如果不存在）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'action_status') THEN
        CREATE TYPE action_status AS ENUM ('pending', 'approved', 'rejected', 'snoozed');
    END IF;
END
$$;

-- 扩展 notifications 表，添加待办相关字段
ALTER TABLE notifications 
    ADD COLUMN IF NOT EXISTS requires_action BOOLEAN DEFAULT false,
    ADD COLUMN IF NOT EXISTS action_type VARCHAR(50),
    ADD COLUMN IF NOT EXISTS action_status action_status DEFAULT 'pending',
    ADD COLUMN IF NOT EXISTS action_deadline TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS action_result JSONB,
    ADD COLUMN IF NOT EXISTS action_by UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS action_at TIMESTAMP WITH TIME ZONE;

-- 创建待办通知索引
CREATE INDEX IF NOT EXISTS idx_notifications_action 
    ON notifications(requires_action, action_status) 
    WHERE requires_action = true AND action_status = 'pending';

CREATE INDEX IF NOT EXISTS idx_notifications_action_type 
    ON notifications(action_type) 
    WHERE requires_action = true;

CREATE INDEX IF NOT EXISTS idx_notifications_action_deadline 
    ON notifications(action_deadline) 
    WHERE requires_action = true AND action_status = 'pending';

-- 添加约束（使用枚举类型后不需要CHECK约束，但保留以确保数据完整性）
-- 注意：枚举类型本身限制了取值范围

-- 添加注释
COMMENT ON COLUMN notifications.requires_action IS '是否需要用户操作';
COMMENT ON COLUMN notifications.action_type IS '操作类型：config_reload, alert_ack 等';
COMMENT ON COLUMN notifications.action_status IS '操作状态：pending, approved, rejected, snoozed';
COMMENT ON COLUMN notifications.action_deadline IS '操作截止时间';
COMMENT ON COLUMN notifications.action_result IS '操作结果数据（JSON格式）';
COMMENT ON COLUMN notifications.action_by IS '执行操作的用户ID';
COMMENT ON COLUMN notifications.action_at IS '操作执行时间';
