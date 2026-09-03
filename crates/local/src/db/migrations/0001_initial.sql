-- Workflow Runs Table
CREATE TABLE IF NOT EXISTS workflow_runs (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    active_job_count INTEGER NOT NULL DEFAULT 0,
    succeeded_job_count INTEGER NOT NULL DEFAULT 0,
    errored_job_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER IF NOT EXISTS workflow_runs_updated_at
AFTER UPDATE ON workflow_runs
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE workflow_runs
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- Job Runs Table
CREATE TABLE IF NOT EXISTS job_runs (
    id TEXT PRIMARY KEY,
    workflow_run_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    status TEXT NOT NULL,
    duration_ms INTEGER,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (workflow_run_id, id),
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS job_runs_workflow_run_id
ON job_runs (workflow_run_id, created_at);

CREATE TRIGGER IF NOT EXISTS job_runs_updated_at
AFTER UPDATE ON job_runs
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE job_runs
    SET updated_at = CURRENT_TIMESTAMP
    WHERE workflow_run_id = NEW.workflow_run_id
      AND id = NEW.id;
END;

-- Tags Table
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS tag_associations (
    id INTEGER PRIMARY KEY,
    tag_id INTEGER NOT NULL,
    value TEXT NOT NULL,
    workflow_run_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS tag_associations_workflow_run_id
ON tag_associations (workflow_run_id, tag_id);
