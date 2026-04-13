-- ============================================
-- 阶段 8.7: IP 白名单/黑名单安全系统
-- ============================================

-- IP 列表类型枚举
CREATE TYPE ip_list_type AS ENUM ('whitelist', 'blacklist');

-- ============================================
-- IP 白名单/黑名单表
-- ============================================
CREATE TABLE ip_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL,
    ip_range_cidr CIDR,              -- 支持 CIDR 格式（如 10.0.0.0/8），NULL 表示单个 IP
    list_type ip_list_type NOT NULL,
    description TEXT,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    expires_at TIMESTAMPTZ,          -- 过期时间，NULL 表示永久
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- 索引
-- ============================================
-- 按列表类型索引（查询白名单或黑名单时使用）
CREATE INDEX idx_ip_lists_type ON ip_lists(list_type);

-- 按 IP 地址索引（快速查找特定 IP）
CREATE INDEX idx_ip_lists_address ON ip_lists(ip_address);

-- 按过期时间索引（清理过期条目时使用）
CREATE INDEX idx_ip_lists_expires ON ip_lists(expires_at) WHERE expires_at IS NOT NULL;

-- 组合索引：类型 + IP（最常用的查询场景）
CREATE INDEX idx_ip_lists_type_address ON ip_lists(list_type, ip_address);

-- ============================================
-- 触发器
-- ============================================
-- 更新时间戳触发器
CREATE TRIGGER update_ip_lists_updated_at
    BEFORE UPDATE ON ip_lists
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- 扩展审计事件类型枚举
-- ============================================
-- 注意：PostgreSQL 不支持直接修改 ENUM 类型添加值
-- 需要在应用层处理新的事件类型，或使用以下方式：

-- 如果需要在数据库层面强制约束，可以创建新的枚举类型并迁移数据
-- 但为简化部署，建议在应用层处理新事件类型

-- ============================================
-- 默认数据
-- ============================================
-- 插入一些示例数据（可选，生产环境建议清空）
-- 本地开发环境白名单示例
-- INSERT INTO ip_lists (ip_address, list_type, description) VALUES
--     ('127.0.0.1', 'whitelist', '本地开发环境'),
--     ('::1', 'whitelist', 'IPv6 本地地址');

-- ============================================
-- 表注释
-- ============================================
COMMENT ON TABLE ip_lists IS 'IP 白名单/黑名单表，用于控制访问权限';
COMMENT ON COLUMN ip_lists.ip_address IS 'IP 地址（IPv4 或 IPv6）';
COMMENT ON COLUMN ip_lists.ip_range_cidr IS 'CIDR 格式的 IP 范围，NULL 表示单个 IP';
COMMENT ON COLUMN ip_lists.list_type IS '列表类型：whitelist 白名单，blacklist 黑名单';
COMMENT ON COLUMN ip_lists.expires_at IS '过期时间，NULL 表示永久有效';
