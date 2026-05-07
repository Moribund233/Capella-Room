-- 私聊功能支持
-- 在 rooms 表添加 room_type 字段区分群聊和私聊

-- 房间类型枚举
CREATE TYPE room_type AS ENUM ('group', 'direct');

-- 在 rooms 表添加 room_type 字段
ALTER TABLE rooms ADD COLUMN room_type room_type DEFAULT 'group';

-- 更新现有房间为群聊类型
UPDATE rooms SET room_type = 'group' WHERE room_type IS NULL;

-- 设置非空约束
ALTER TABLE rooms ALTER COLUMN room_type SET NOT NULL;

-- 创建索引加速查询
CREATE INDEX idx_rooms_type ON rooms(room_type);

-- 创建唯一索引确保两个用户之间只有一个私聊房间
-- 通过 room_members 表来确保
CREATE UNIQUE INDEX idx_direct_room_members ON room_members (room_id, user_id);
