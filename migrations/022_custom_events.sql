-- 022_custom_events.sql
CREATE TABLE custom_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_name  VARCHAR(128) NOT NULL,
    room_id     UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    source_app  VARCHAR(64) NOT NULL,
    data        JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_custom_events_room_created ON custom_events(room_id, created_at DESC);
