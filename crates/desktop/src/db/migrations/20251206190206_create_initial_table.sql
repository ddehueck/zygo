-- Enable foreign key enforcement (must be run per connection)
-- PRAGMA foreign_keys = ON;

-- Workflow table
CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);


-- Workflow versions table
CREATE TABLE IF NOT EXISTS workflow_versions (
    id TEXT NOT NULL UNIQUE,  -- Globally unique, allows single-column FK references
    workflow_id TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    content_hash TEXT NOT NULL,
    entrypoint_json TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Composite PK for consistency checks in other tables
    PRIMARY KEY (id, workflow_id),
    UNIQUE (workflow_id, content_hash)
);
CREATE INDEX IF NOT EXISTS idx_workflow_versions_workflow_id_created_at ON workflow_versions (workflow_id, created_at);


-- Channels table
CREATE TABLE IF NOT EXISTS channels (
    id TEXT NOT NULL,
    workflow_version_id TEXT NOT NULL REFERENCES workflow_versions(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id, workflow_version_id),
    UNIQUE (workflow_version_id, name)
);


-- Jobs table
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT NOT NULL,
    workflow_version_id TEXT NOT NULL REFERENCES workflow_versions(id) ON DELETE CASCADE,
    content_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    entrypoint_json TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id, workflow_version_id),
    UNIQUE (workflow_version_id, content_hash)
);


-- Job channel edges table
CREATE TABLE IF NOT EXISTS job_channel_edges (
    workflow_version_id TEXT NOT NULL REFERENCES workflow_versions(id) ON DELETE CASCADE,
    job_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('input', 'output')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (channel_id, job_id, workflow_version_id),

    FOREIGN KEY (job_id, workflow_version_id)
        REFERENCES jobs(id, workflow_version_id) ON DELETE CASCADE,

    FOREIGN KEY (channel_id, workflow_version_id)
        REFERENCES channels(id, workflow_version_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_job_channel_edges_job_id ON job_channel_edges (job_id);
CREATE INDEX IF NOT EXISTS idx_job_channel_edges_workflow_version_id ON job_channel_edges (workflow_version_id);

CREATE TABLE IF NOT EXISTS runs (
    id TEXT NOT NULL,
    workflow_version_id TEXT NOT NULL REFERENCES workflow_versions(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id, workflow_version_id)
);
CREATE INDEX IF NOT EXISTS idx_runs_workflow_version_id_created_at ON runs (workflow_version_id, created_at);


-- Run events table (immutable event log)
CREATE TABLE IF NOT EXISTS events (
    id TEXT NOT NULL,
    workflow_version_id TEXT NOT NULL,
    is_replay INTEGER NOT NULL DEFAULT 0,  -- Flag to indicate if this event is a replay (1) or original (0)
    timestamp TIMESTAMP NOT NULL,

    -- The run_id is unique on a per workflow.run() basis.
    -- The sequence_number is the order of the events within the run.
    run_id TEXT NOT NULL,
    sequence_number INTEGER NOT NULL,
    
    -- Event specific information
    kind TEXT NOT NULL CHECK (kind IN ('job_requested', 'job_started', 'job_succeeded', 'job_failed', 'channel_item_inserted')),
    
    -- Every event is scoped to a job + data_id as each job is considered a pure function.
    job_id TEXT,
    job_idempotency_key TEXT,  -- The idempotency key for the job, useful for determing if a job has already been run

    inserted_channel_id TEXT,
    inserted_channel_data_id TEXT,  -- External ID for the data item being inserted into the channel
    
    PRIMARY KEY (id, workflow_version_id),

    FOREIGN KEY (run_id, workflow_version_id)
        REFERENCES runs(id, workflow_version_id) ON DELETE CASCADE,

    FOREIGN KEY (workflow_version_id)
        REFERENCES workflow_versions(id) ON DELETE CASCADE,

    FOREIGN KEY (job_id, workflow_version_id)
        REFERENCES jobs(id, workflow_version_id) ON DELETE CASCADE,

    FOREIGN KEY (inserted_channel_id, workflow_version_id)
        REFERENCES channels(id, workflow_version_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_events_workflow_version_id_timestamp ON events (workflow_version_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_job_id_job_idempotency_key ON events (job_id, job_idempotency_key);  
