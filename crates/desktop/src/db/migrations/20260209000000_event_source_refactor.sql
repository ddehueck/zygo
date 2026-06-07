-- Data references table
CREATE TABLE IF NOT EXISTS data_references (
    id          TEXT PRIMARY KEY,
    uri         TEXT NOT NULL,
    etag        TEXT NOT NULL,
    content_type TEXT,
    size_bytes  INTEGER,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (uri, etag)
);
CREATE INDEX IF NOT EXISTS idx_data_references_uri ON data_references (uri);


-- Recreate events table with new source columns and data_reference_id
-- SQLite requires table recreation for CHECK constraint and column changes.

-- 1. Drop old indexes
DROP INDEX IF EXISTS idx_events_workflow_version_id_timestamp;
DROP INDEX IF EXISTS idx_events_job_id_job_idempotency_key;

-- 2. Rename old table
ALTER TABLE events RENAME TO events_old;

-- 3. Create new events table
CREATE TABLE events (
    id TEXT NOT NULL,
    workflow_version_id TEXT NOT NULL,
    is_replay INTEGER NOT NULL DEFAULT 0,
    timestamp TIMESTAMP NOT NULL,

    -- The workflow_run_id is unique on a per workflow.run() basis.
    -- The sequence_number is the order of the events within the run.
    workflow_run_id TEXT NOT NULL,
    sequence_number INTEGER NOT NULL,

    -- Source information: where this event came from
    source_type TEXT NOT NULL CHECK (source_type IN ('input', 'job_run')),
    source_job_id TEXT,       -- Only set for job_run source
    source_job_run_id TEXT,   -- Only set for job_run source (was job_idempotency_key)

    -- Event specific information
    kind TEXT NOT NULL CHECK (kind IN ('job_requested', 'job_started', 'job_succeeded', 'job_failed', 'channel_item_inserted', 'data_reference_inserted')),

    -- For job_requested events: the job being requested
    job_id TEXT,

    -- For channel_item_inserted events: the channel being inserted into
    inserted_channel_id TEXT,

    -- For events that carry data: reference to the data_references table
    data_reference_id TEXT,

    PRIMARY KEY (id, workflow_version_id),

    FOREIGN KEY (workflow_run_id, workflow_version_id)
        REFERENCES runs(id, workflow_version_id) ON DELETE CASCADE,

    FOREIGN KEY (workflow_version_id)
        REFERENCES workflow_versions(id) ON DELETE CASCADE,

    FOREIGN KEY (job_id, workflow_version_id)
        REFERENCES jobs(id, workflow_version_id) ON DELETE CASCADE,

    FOREIGN KEY (inserted_channel_id, workflow_version_id)
        REFERENCES channels(id, workflow_version_id) ON DELETE CASCADE,

    FOREIGN KEY (data_reference_id)
        REFERENCES data_references(id) ON DELETE SET NULL
);

-- 4. Migrate data from old table
-- Old events without source info get source_type='input', with job_id events get source_type='job_run'
INSERT INTO events (
    id, workflow_version_id, is_replay, timestamp,
    workflow_run_id, sequence_number,
    source_type, source_job_id, source_job_run_id,
    kind, job_id, inserted_channel_id, data_reference_id
)
SELECT
    id, workflow_version_id, is_replay, timestamp,
    run_id, sequence_number,
    CASE WHEN job_id IS NOT NULL AND kind != 'job_requested' THEN 'job_run' ELSE 'input' END,
    CASE WHEN kind != 'job_requested' THEN job_id ELSE NULL END,
    CASE WHEN kind != 'job_requested' THEN job_idempotency_key ELSE NULL END,
    kind,
    CASE WHEN kind = 'job_requested' THEN job_id ELSE NULL END,
    inserted_channel_id,
    NULL  -- No data_reference_id for old events
FROM events_old;

-- 5. Drop old table
DROP TABLE events_old;

-- 6. Recreate indexes
CREATE INDEX IF NOT EXISTS idx_events_workflow_version_id_timestamp ON events (workflow_version_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_source_job_id_source_job_run_id ON events (source_job_id, source_job_run_id);
CREATE INDEX IF NOT EXISTS idx_events_workflow_run_id ON events (workflow_run_id);
