-- The event log is independent of the workflow_runs projection.
CREATE TABLE event_stream (
    workflow_run_id TEXT NOT NULL CHECK (length(trim(workflow_run_id)) > 0),
    sequence_id INTEGER NOT NULL CHECK (typeof(sequence_id) = 'integer' AND sequence_id >= 0),
    event TEXT NOT NULL,
    PRIMARY KEY (workflow_run_id, sequence_id)
);
