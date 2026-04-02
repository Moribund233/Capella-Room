-- 文件资源表迁移
-- 创建文件分类枚举
CREATE TYPE file_category AS ENUM ('image', 'document', 'video', 'audio', 'other');

-- 创建文件用途枚举
CREATE TYPE file_usage_type AS ENUM ('avatar', 'message', 'room_cover', 'general');

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

-- 创建索引
CREATE INDEX idx_file_resources_uploader ON file_resources(uploader_id);
CREATE INDEX idx_file_resources_category ON file_resources(category);
CREATE INDEX idx_file_resources_hash ON file_resources(file_hash);
CREATE INDEX idx_file_resources_room ON file_resources(room_id);
CREATE INDEX idx_file_resources_message ON file_resources(message_id);
CREATE INDEX idx_file_resources_created_at ON file_resources(created_at);
CREATE INDEX idx_file_resources_usage_type ON file_resources(usage_type);

-- 更新时间戳触发器
CREATE TRIGGER update_file_resources_updated_at BEFORE UPDATE ON file_resources
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
