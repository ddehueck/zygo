use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{
    JobRunId, JobRunStatus, SequenceId, WorkflowRunId, WorkflowRunStatus, WorkflowSchema,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct EngineState {
    pub id: WorkflowRunId,
    pub status: WorkflowRunStatus,
    pub schema: WorkflowSchema,
    cursor: RunCursor,
    status_by_job_run_id: HashMap<JobRunId, JobRunStatus>,
}

impl EngineState {
    pub fn new(id: &WorkflowRunId, schema: &WorkflowSchema) -> Self {
        Self {
            id: id.clone(),
            status: WorkflowRunStatus::Running,
            schema: schema.clone(),
            cursor: RunCursor::default(),
            status_by_job_run_id: HashMap::new(),
        }
    }

    pub fn set_job_status(&mut self, job_run_id: JobRunId, status: JobRunStatus) -> () {
        self.status_by_job_run_id.insert(job_run_id, status);
        self.status = compute_run_status(&self.status_by_job_run_id);
    }

    pub fn next_id(&self) -> SequenceId {
        self.cursor.next_id
    }

    pub fn increment_cursor(&mut self) {
        self.cursor.next_id = self.cursor.next_id.increment();
    }
}

fn compute_run_status(status_by_job_run_id: &HashMap<JobRunId, JobRunStatus>) -> WorkflowRunStatus {
    if status_by_job_run_id.is_empty() {
        return WorkflowRunStatus::Running;
    }

    if status_by_job_run_id
        .values()
        .any(|status| *status == JobRunStatus::Failed)
    {
        return WorkflowRunStatus::Failed;
    }

    if status_by_job_run_id
        .values()
        .all(|status| *status == JobRunStatus::Succeeded)
    {
        return WorkflowRunStatus::Succeeded;
    }

    WorkflowRunStatus::Running
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RunCursor {
    pub next_id: SequenceId,
}

impl RunCursor {
    pub fn default() -> Self {
        Self {
            next_id: SequenceId::new(0),
        }
    }
}
