ALTER TABLE export_jobs
    ADD COLUMN IF NOT EXISTS object_key TEXT,
    ADD COLUMN IF NOT EXISTS content_type TEXT,
    ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ;

UPDATE export_jobs
SET status = 'queued'
WHERE status = 'pending';

ALTER TABLE export_jobs
    DROP CONSTRAINT IF EXISTS export_jobs_status_check;

ALTER TABLE export_jobs
    ADD CONSTRAINT export_jobs_status_check CHECK (
        status IN ('queued', 'processing', 'completed', 'failed')
    );

CREATE TABLE IF NOT EXISTS outbox_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    event_version INTEGER NOT NULL DEFAULT 1,
    payload_json JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    publish_attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_outbox_events_unpublished
    ON outbox_events (occurred_at)
    WHERE published_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_outbox_events_event_type
    ON outbox_events (event_type, occurred_at DESC);

CREATE TABLE IF NOT EXISTS processed_messages (
    consumer_name TEXT NOT NULL,
    message_id UUID NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (consumer_name, message_id)
);

CREATE TABLE IF NOT EXISTS event_list_projection (
    event_id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category_id UUID NOT NULL,
    category_name TEXT NOT NULL,
    category_color TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    location TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    budget DOUBLE PRECISION NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_event_list_projection_user_starts_at
    ON event_list_projection (user_id, starts_at);

CREATE INDEX IF NOT EXISTS idx_event_list_projection_user_status
    ON event_list_projection (user_id, status);

CREATE INDEX IF NOT EXISTS idx_event_list_projection_user_category
    ON event_list_projection (user_id, category_id);

CREATE TABLE IF NOT EXISTS calendar_projection (
    event_id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    date_bucket DATE NOT NULL,
    title TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL,
    category_color TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_calendar_projection_user_date
    ON calendar_projection (user_id, date_bucket);

CREATE TABLE IF NOT EXISTS dashboard_projection (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    total_events BIGINT NOT NULL DEFAULT 0,
    upcoming_events BIGINT NOT NULL DEFAULT 0,
    completed_events BIGINT NOT NULL DEFAULT 0,
    cancelled_events BIGINT NOT NULL DEFAULT 0,
    total_budget DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS report_projection (
    event_id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category_id UUID NOT NULL,
    category_name TEXT NOT NULL,
    category_color TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    location TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    budget DOUBLE PRECISION NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_report_projection_user_starts_at
    ON report_projection (user_id, starts_at);

CREATE INDEX IF NOT EXISTS idx_report_projection_user_status
    ON report_projection (user_id, status);

CREATE TABLE IF NOT EXISTS recent_activity_projection (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_message_id UUID NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    action TEXT NOT NULL,
    title TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_recent_activity_projection_user_time
    ON recent_activity_projection (user_id, occurred_at DESC);

INSERT INTO event_list_projection (
    event_id,
    user_id,
    category_id,
    category_name,
    category_color,
    title,
    description,
    location,
    starts_at,
    ends_at,
    budget,
    status,
    created_at,
    updated_at
)
SELECT
    e.id,
    e.user_id,
    e.category_id,
    c.name,
    c.color,
    e.title,
    e.description,
    e.location,
    e.starts_at,
    e.ends_at,
    e.budget,
    e.status,
    e.created_at,
    e.updated_at
FROM events e
INNER JOIN categories c ON c.id = e.category_id
ON CONFLICT (event_id) DO UPDATE
SET
    user_id = EXCLUDED.user_id,
    category_id = EXCLUDED.category_id,
    category_name = EXCLUDED.category_name,
    category_color = EXCLUDED.category_color,
    title = EXCLUDED.title,
    description = EXCLUDED.description,
    location = EXCLUDED.location,
    starts_at = EXCLUDED.starts_at,
    ends_at = EXCLUDED.ends_at,
    budget = EXCLUDED.budget,
    status = EXCLUDED.status,
    created_at = EXCLUDED.created_at,
    updated_at = EXCLUDED.updated_at;

INSERT INTO calendar_projection (
    event_id,
    user_id,
    date_bucket,
    title,
    starts_at,
    ends_at,
    status,
    category_color,
    updated_at
)
SELECT
    e.id,
    e.user_id,
    DATE(e.starts_at AT TIME ZONE 'UTC'),
    e.title,
    e.starts_at,
    e.ends_at,
    e.status,
    c.color,
    e.updated_at
FROM events e
INNER JOIN categories c ON c.id = e.category_id
ON CONFLICT (event_id) DO UPDATE
SET
    user_id = EXCLUDED.user_id,
    date_bucket = EXCLUDED.date_bucket,
    title = EXCLUDED.title,
    starts_at = EXCLUDED.starts_at,
    ends_at = EXCLUDED.ends_at,
    status = EXCLUDED.status,
    category_color = EXCLUDED.category_color,
    updated_at = EXCLUDED.updated_at;

INSERT INTO report_projection (
    event_id,
    user_id,
    category_id,
    category_name,
    category_color,
    title,
    description,
    location,
    starts_at,
    ends_at,
    budget,
    status,
    created_at,
    updated_at
)
SELECT
    e.id,
    e.user_id,
    e.category_id,
    c.name,
    c.color,
    e.title,
    e.description,
    e.location,
    e.starts_at,
    e.ends_at,
    e.budget,
    e.status,
    e.created_at,
    e.updated_at
FROM events e
INNER JOIN categories c ON c.id = e.category_id
ON CONFLICT (event_id) DO UPDATE
SET
    user_id = EXCLUDED.user_id,
    category_id = EXCLUDED.category_id,
    category_name = EXCLUDED.category_name,
    category_color = EXCLUDED.category_color,
    title = EXCLUDED.title,
    description = EXCLUDED.description,
    location = EXCLUDED.location,
    starts_at = EXCLUDED.starts_at,
    ends_at = EXCLUDED.ends_at,
    budget = EXCLUDED.budget,
    status = EXCLUDED.status,
    created_at = EXCLUDED.created_at,
    updated_at = EXCLUDED.updated_at;

INSERT INTO dashboard_projection (
    user_id,
    total_events,
    upcoming_events,
    completed_events,
    cancelled_events,
    total_budget,
    updated_at
)
SELECT
    u.id,
    COUNT(e.id)::BIGINT,
    COUNT(*) FILTER (WHERE e.starts_at >= NOW() AND e.status <> 'cancelled')::BIGINT,
    COUNT(*) FILTER (WHERE e.status = 'completed')::BIGINT,
    COUNT(*) FILTER (WHERE e.status = 'cancelled')::BIGINT,
    COALESCE(SUM(e.budget), 0)::DOUBLE PRECISION,
    NOW()
FROM users u
LEFT JOIN events e ON e.user_id = u.id
GROUP BY u.id
ON CONFLICT (user_id) DO UPDATE
SET
    total_events = EXCLUDED.total_events,
    upcoming_events = EXCLUDED.upcoming_events,
    completed_events = EXCLUDED.completed_events,
    cancelled_events = EXCLUDED.cancelled_events,
    total_budget = EXCLUDED.total_budget,
    updated_at = EXCLUDED.updated_at;

INSERT INTO recent_activity_projection (
    source_message_id,
    user_id,
    entity_type,
    entity_id,
    action,
    title,
    occurred_at
)
SELECT
    gen_random_uuid(),
    e.user_id,
    'event',
    e.id,
    'updated',
    e.title,
    e.updated_at
FROM events e
ON CONFLICT (source_message_id) DO NOTHING;
