-- ============================================
-- Phase 9.5: 性能细节优化 — 数据库索引优化
-- ============================================
-- 目的：针对 9.2 边界测试中发现的三类慢查询添加优化索引
-- 慢查询 ① 房间列表/详情 LEFT JOIN LATERAL (1.3s)
-- 慢查询 ② DELETE FROM messages WHERE room_id (53s)
-- 慢查询 ③ 消息查询 ORDER BY created_at DESC (隐含)
-- ============================================

-- --------------------------------------------------
-- 优化 1: 消息表 — 活跃消息的复合索引（加速 LATERAL 子查询）
-- 场景: LEFT JOIN LATERAL (...) WHERE is_deleted=false ORDER BY created_at DESC LIMIT 1
-- 覆盖: room_id 过滤 + is_deleted 过滤 + created_at 排序
-- --------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_messages_room_active_last
    ON messages(room_id, created_at DESC)
    WHERE is_deleted = false;

-- --------------------------------------------------
-- 优化 2: 消息表 — reply_to 自引用外键索引（加速级联操作）
-- 场景: DELETE FROM messages WHERE room_id = $1
--       触发 ON DELETE SET NULL 检查 messages.reply_to
-- --------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_messages_reply_to
    ON messages(reply_to);

-- --------------------------------------------------
-- 优化 3: 消息表 — 带 is_deleted 过滤的查询索引
-- 场景: get_room_messages 通常只查未删除消息
-- --------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_messages_room_not_deleted
    ON messages(room_id, created_at DESC)
    WHERE is_deleted = false;

-- --------------------------------------------------
-- 优化 4: 房间表 — 创建时间倒序索引（加速房间列表排序）
-- 场景: ORDER BY r.created_at DESC
-- --------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_rooms_created_desc
    ON rooms(created_at DESC);

-- --------------------------------------------------
-- 优化 5: 成员表 — 用户+房间复合索引
-- 场景: JOIN room_members rm ON r.id = rm.room_id WHERE rm.user_id = $1
-- --------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_room_members_user_room
    ON room_members(user_id, room_id);

-- --------------------------------------------------
-- 优化 6: 文件资源表 — room_id + message_id 复合索引（加速文件清理）
-- 场景: 删除房间时级联更新 file_resources
-- --------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_file_resources_room_message
    ON file_resources(room_id, message_id);

-- 性能监控视图（需启用 pg_stat_statements 扩展，默认不启用）
-- CREATE VIEW slow_query_stats AS ...
