-- 移除外键约束：审计日志的 actor_id 不需要引用 users(id)
-- 审计日志是历史记录，即使对应的用户被删除，日志条目本身仍需保留
-- 用 ON DELETE SET NULL 清理历史行的 actor_id 实际很少用到，
-- 但该约束会阻止写入 actor_id 引用已删除用户的新日志行
ALTER TABLE audit_logs DROP CONSTRAINT IF EXISTS audit_logs_actor_id_fkey;
