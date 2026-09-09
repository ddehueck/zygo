CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_run_id TEXT NOT NULL,
    job_run_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS logs_workflow_run_id ON logs (workflow_run_id, id);
CREATE INDEX IF NOT EXISTS logs_job_run_id ON logs (job_run_id, id);
CREATE INDEX logs_content_fts ON logs USING fts (content);
