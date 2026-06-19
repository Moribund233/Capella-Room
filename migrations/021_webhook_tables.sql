-- 021_webhook_tables.sql
CREATE TABLE webhook_subscriptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    url             TEXT NOT NULL,
    secret          VARCHAR(128) NOT NULL,
    events          TEXT[] NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    max_retries     INT NOT NULL DEFAULT 3,
    retry_interval_secs INT NOT NULL DEFAULT 10,
    timeout_ms      INT NOT NULL DEFAULT 5000,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE webhook_deliveries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES webhook_subscriptions(id) ON DELETE CASCADE,
    event_type      VARCHAR(64) NOT NULL,
    event_id        UUID NOT NULL,
    payload         JSONB NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'success', 'failed', 'cancelled')),
    http_status     INT,
    response_body   TEXT,
    attempt_count   INT NOT NULL DEFAULT 0,
    next_retry_at   TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deliveries_pending ON webhook_deliveries(status, next_retry_at)
    WHERE status = 'pending' OR status = 'failed';
CREATE INDEX idx_deliveries_sub ON webhook_deliveries(subscription_id, created_at DESC);
