-- 迁移: 全文搜索优化和活动统计查询优化
-- 日期: 2026-04-29

-- ============================================
-- 1. 确保现有消息的 content_tsv 已被正确填充
-- ============================================

-- 更新所有现有消息的 content_tsv（如果为NULL或空）
UPDATE messages 
SET content_tsv = to_tsvector('simple', content)
WHERE content_tsv IS NULL OR content_tsv = ''::tsvector;

-- ============================================
-- 2. 创建复合索引优化活动统计查询
-- ============================================

-- 用于日/周/月活跃用户统计的复合索引
CREATE INDEX IF NOT EXISTS idx_messages_created_at_sender 
ON messages (created_at, sender_id) 
WHERE is_deleted = false;

-- 用于日/周/月消息量统计的复合索引
CREATE INDEX IF NOT EXISTS idx_messages_created_at_only 
ON messages (created_at) 
WHERE is_deleted = false;

-- ============================================
-- 3. 创建活动统计物化视图（可选，用于高频查询场景）
-- ============================================

-- 如果活动统计查询非常频繁，可以取消注释以下代码创建物化视图
/*
CREATE MATERIALIZED VIEW IF NOT EXISTS activity_stats_summary AS
SELECT
    COUNT(DISTINCT sender_id) FILTER (WHERE created_at > NOW() - INTERVAL '1 day') AS daily_active_users,
    COUNT(DISTINCT sender_id) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') AS weekly_active_users,
    COUNT(DISTINCT sender_id) FILTER (WHERE created_at > NOW() - INTERVAL '30 days') AS monthly_active_users,
    COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '1 day') AS daily_messages,
    COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') AS weekly_messages,
    COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '30 days') AS monthly_messages
FROM messages
WHERE is_deleted = false;

-- 创建唯一索引用于并发刷新
CREATE UNIQUE INDEX idx_activity_stats_summary_singleton ON activity_stats_summary (daily_active_users);

-- 创建刷新函数
CREATE OR REPLACE FUNCTION refresh_activity_stats_summary()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY activity_stats_summary;
END;
$$ LANGUAGE plpgsql;
*/

-- ============================================
-- 4. 验证全文搜索功能
-- ============================================

-- 测试查询（可选，执行后应返回结果）
-- SELECT * FROM messages WHERE content_tsv @@ to_tsquery('simple', 'test:*') LIMIT 1;
