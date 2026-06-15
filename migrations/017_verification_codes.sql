-- ============================================
-- Phase: Verification Codes
-- ============================================
-- 验证码系统：注册/登录/密码重置的邮箱验证码

CREATE TABLE IF NOT EXISTS verification_codes (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email      VARCHAR(255) NOT NULL,
    code       VARCHAR(8) NOT NULL,
    purpose    VARCHAR(32) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at    TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vc_email_purpose
    ON verification_codes(email, purpose, created_at DESC);
