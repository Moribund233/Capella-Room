-- 020_room_resource_bindings.sql
CREATE TABLE room_resource_bindings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id         UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    resource_type   VARCHAR(64) NOT NULL,
    resource_id     VARCHAR(255) NOT NULL,
    resource_url    TEXT,
    resource_name   VARCHAR(255),
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, resource_type, resource_id)
);

CREATE INDEX idx_rrb_room_id ON room_resource_bindings(room_id);
CREATE INDEX idx_rrb_app_id ON room_resource_bindings(app_id);
CREATE INDEX idx_rrb_lookup ON room_resource_bindings(app_id, resource_type, resource_id);
