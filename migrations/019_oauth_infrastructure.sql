-- oauth_apps
CREATE TABLE IF NOT EXISTS oauth_apps (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL,
    description     TEXT,
    client_secret   VARCHAR(256) NOT NULL,
    redirect_uris   TEXT[] NOT NULL DEFAULT '{}',
    scopes          TEXT[] NOT NULL DEFAULT '{}',
    owner_id        UUID NOT NULL REFERENCES users(id),
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- oauth_authorization_codes
CREATE TABLE IF NOT EXISTS oauth_authorization_codes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id),
    code            VARCHAR(64) NOT NULL UNIQUE,
    redirect_uri    TEXT,
    scopes          TEXT[],
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- oauth_tokens
CREATE TABLE IF NOT EXISTS oauth_tokens (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id              UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES users(id),
    access_token        VARCHAR(512) NOT NULL UNIQUE,
    refresh_token_hash  VARCHAR(256),
    scopes              TEXT[],
    expires_at          TIMESTAMPTZ NOT NULL,
    refresh_expires_at  TIMESTAMPTZ,
    revoked_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- user_identity_mappings
CREATE TABLE IF NOT EXISTS user_identity_mappings (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users(id),
    app_id            UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    external_user_id  VARCHAR(255) NOT NULL,
    external_username VARCHAR(255),
    mapped_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, external_user_id)
);

-- 索引
CREATE INDEX idx_oauth_apps_owner_id ON oauth_apps(owner_id);
CREATE INDEX idx_oauth_auth_codes_app_id ON oauth_authorization_codes(app_id);
CREATE INDEX idx_oauth_auth_codes_user_id ON oauth_authorization_codes(user_id);
CREATE INDEX idx_oauth_tokens_app_id ON oauth_tokens(app_id);
CREATE INDEX idx_oauth_tokens_user_id ON oauth_tokens(user_id);
CREATE INDEX idx_oauth_mappings_user_id ON user_identity_mappings(user_id);
CREATE INDEX idx_oauth_mappings_app_id ON user_identity_mappings(app_id);
