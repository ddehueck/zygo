CREATE TABLE IF NOT EXISTS logs (
    job_run_id INTEGER NOT NULL,
    "order" INTEGER NOT NULL CHECK ("order" >= 1),
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (job_run_id, "order"),
    FOREIGN KEY (job_run_id) REFERENCES job_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS logs_job_run_id ON logs (job_run_id, "order");

-- Temporarily disable to allow us to open the db file in outerbase.
-- CREATE INDEX logs_content_fts ON logs USING fts (content);
