-- ============================================
-- 迁移: 更新审计事件类型枚举
-- 说明: 添加好友事件和IP安全事件类型到 audit_event_type 枚举
-- ============================================

-- PostgreSQL 不支持直接修改 ENUM 类型添加值
-- 需要使用 ALTER TYPE ... ADD VALUE 语法（PostgreSQL 9.1+）

-- 添加好友相关事件类型
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'user_friend_request_send';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'user_friend_request_accept';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'user_friend_request_reject';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'user_friend_request_cancel';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'user_friend_remove';

-- 添加IP安全相关事件类型
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'ip_blocked';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'ip_whitelist_denied';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'ip_rate_limited';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'ip_list_added';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'ip_list_removed';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'ip_list_updated';

-- 验证更新
-- 查询当前枚举类型的所有值
-- SELECT enumlabel FROM pg_enum WHERE enumtypid = 'audit_event_type'::regtype ORDER BY enumsortorder;
