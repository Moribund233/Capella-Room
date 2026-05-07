-- 房间邀请系统
-- 支持生成邀请链接/邀请码加入私有房间

-- 房间邀请表
CREATE TABLE room_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    inviter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invite_code VARCHAR(20) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ,
    max_uses INT,
    used_count INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 邀请使用记录表（用于追踪谁使用了邀请）
CREATE TABLE room_invitation_uses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invitation_id UUID NOT NULL REFERENCES room_invitations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    used_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(invitation_id, user_id)
);

-- 索引
CREATE INDEX idx_room_invitations_room ON room_invitations(room_id);
CREATE INDEX idx_room_invitations_code ON room_invitations(invite_code);
CREATE INDEX idx_room_invitations_active ON room_invitations(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_room_invitation_uses_invitation ON room_invitation_uses(invitation_id);
