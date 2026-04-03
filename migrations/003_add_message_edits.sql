-- 添加消息编辑历史表
-- 用于存储消息编辑记录

-- 创建消息编辑历史表
CREATE TABLE IF NOT EXISTS message_edits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    editor_id UUID NOT NULL REFERENCES users(id),
    old_content TEXT NOT NULL,
    new_content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 添加索引
CREATE INDEX IF NOT EXISTS idx_message_edits_message_id ON message_edits(message_id);
CREATE INDEX IF NOT EXISTS idx_message_edits_created_at ON message_edits(created_at);

-- 添加消息编辑次数字段到messages表
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS edit_count INTEGER NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS edited_at TIMESTAMPTZ;

-- 添加索引
CREATE INDEX IF NOT EXISTS idx_messages_edited_at ON messages(edited_at) WHERE edited_at IS NOT NULL;

-- 添加全文搜索支持
-- 创建消息内容的全文搜索向量（中文支持）
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS content_tsv TSVECTOR;

-- 创建全文搜索索引
CREATE INDEX IF NOT EXISTS idx_messages_content_tsv ON messages USING GIN(content_tsv);

-- 创建触发器函数来自动更新全文搜索向量
CREATE OR REPLACE FUNCTION update_message_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    -- 对于英文使用 to_tsvector，中文需要额外处理
    NEW.content_tsv := to_tsvector('simple', NEW.content);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器
DROP TRIGGER IF EXISTS trigger_update_message_search ON messages;
CREATE TRIGGER trigger_update_message_search
    BEFORE INSERT OR UPDATE OF content ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_message_search_vector();

-- 为现有消息更新全文搜索向量
UPDATE messages SET content_tsv = to_tsvector('simple', content) WHERE content_tsv IS NULL;
