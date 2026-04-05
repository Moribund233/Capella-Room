-- 创建通知类型枚举
CREATE TYPE notification_type AS ENUM (
    'private_message',
    'mentioned',
    'room_invitation',
    'system_notification',
    'file_upload_complete'
);

-- 创建通知表
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type notification_type NOT NULL,
    title VARCHAR(200),
    content TEXT NOT NULL,
    data JSONB,
    is_read BOOLEAN DEFAULT false,
    read_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE -- 可选：通知过期时间
);

-- 创建索引优化查询性能
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, is_read) WHERE is_read = false;
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_type ON notifications(notification_type);

-- 添加注释
COMMENT ON TABLE notifications IS '用户通知表，存储离线通知和通知历史';
COMMENT ON COLUMN notifications.user_id IS '接收通知的用户ID';
COMMENT ON COLUMN notifications.notification_type IS '通知类型：私信、@提及、房间邀请、系统通知、文件上传完成';
COMMENT ON COLUMN notifications.title IS '通知标题';
COMMENT ON COLUMN notifications.content IS '通知内容';
COMMENT ON COLUMN notifications.data IS '额外的通知数据（JSON格式）';
COMMENT ON COLUMN notifications.is_read IS '是否已读';
COMMENT ON COLUMN notifications.read_at IS '阅读时间';
COMMENT ON COLUMN notifications.expires_at IS '过期时间，用于自动清理过期通知';
