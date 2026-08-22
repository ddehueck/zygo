CREATE TABLE IF NOT EXISTS workflow_run_summary (
    workflow_run_id TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    active_job_count INTEGER NOT NULL DEFAULT 0,
    succeeded_job_count INTEGER NOT NULL DEFAULT 0,
    errored_job_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE
);

CREATE TRIGGER IF NOT EXISTS workflow_run_summary_updated_at
AFTER UPDATE ON workflow_run_summary
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE workflow_run_summary
    SET updated_at = CURRENT_TIMESTAMP
    WHERE workflow_run_id = NEW.workflow_run_id;
END;

CREATE TABLE IF NOT EXISTS job_run_summary (
    workflow_run_id TEXT NOT NULL,
    job_run_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    status TEXT NOT NULL,
    duration_ms INTEGER,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (workflow_run_id, job_run_id),
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS job_run_summary_workflow_run_id
ON job_run_summary (workflow_run_id, created_at);

CREATE TRIGGER IF NOT EXISTS job_run_summary_updated_at
AFTER UPDATE ON job_run_summary
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE job_run_summary
    SET updated_at = CURRENT_TIMESTAMP
    WHERE workflow_run_id = NEW.workflow_run_id
      AND job_run_id = NEW.job_run_id;
END;
