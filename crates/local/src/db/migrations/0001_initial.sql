-- Workflow Runs Table
CREATE TABLE IF NOT EXISTS workflow_runs (
    id INTEGER PRIMARY KEY,
    public_id TEXT NOT NULL UNIQUE,
    workflow_id TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    active_job_count INTEGER NOT NULL DEFAULT 0,
    succeeded_job_count INTEGER NOT NULL DEFAULT 0,
    errored_job_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Job Runs Table
CREATE TABLE IF NOT EXISTS job_runs (
    id INTEGER PRIMARY KEY,
    public_id TEXT NOT NULL UNIQUE,
    workflow_run_id INTEGER NOT NULL,
    job_id TEXT NOT NULL,
    status TEXT NOT NULL,
    duration_ms INTEGER,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS job_runs_workflow_run_id ON job_runs (workflow_run_id, created_at);

-- Data References Table
CREATE TABLE IF NOT EXISTS data_references (
    id INTEGER PRIMARY KEY,
    workflow_run_id INTEGER NOT NULL,
    job_run_id INTEGER NOT NULL,
    uri TEXT NOT NULL,
    is_replay INTEGER NOT NULL DEFAULT 0 CHECK (is_replay IN (0, 1)),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (workflow_run_id, job_run_id, uri),
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (job_run_id) REFERENCES job_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS data_references_job_run_id
ON data_references (workflow_run_id, job_run_id, inserted_at, id);

CREATE INDEX IF NOT EXISTS data_references_workflow_run_id
ON data_references (workflow_run_id, inserted_at, id);

CREATE INDEX IF NOT EXISTS data_references_uri_version
ON data_references (uri, version);

-- Tags Table
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY,
    value TEXT NOT NULL,
    workflow_run_id INTEGER NOT NULL,
    job_run_id INTEGER,
    data_reference_id INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (workflow_run_id, job_run_id, data_reference_id, value),
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (job_run_id) REFERENCES job_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (data_reference_id) REFERENCES data_references(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS tags_workflow_run_id ON tags (workflow_run_id, value, id);
CREATE INDEX IF NOT EXISTS tags_job_run_id ON tags (job_run_id, id);
CREATE INDEX IF NOT EXISTS tags_data_reference_id ON tags (data_reference_id, id);
