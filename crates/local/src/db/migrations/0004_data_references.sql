CREATE TABLE IF NOT EXISTS data_references (
    id INTEGER PRIMARY KEY,
    workflow_run_id TEXT NOT NULL,
    job_run_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    uri TEXT NOT NULL,
    version TEXT NOT NULL,
    is_replay INTEGER NOT NULL DEFAULT 0 CHECK (is_replay IN (0, 1)),
    inserted_at TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (workflow_run_id, job_run_id, uri, version),
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS data_references_job_run_id
ON data_references (workflow_run_id, job_run_id, inserted_at, id);

CREATE INDEX IF NOT EXISTS data_references_workflow_run_id
ON data_references (workflow_run_id, inserted_at, id);

CREATE INDEX IF NOT EXISTS data_references_uri_version
ON data_references (uri, version);
