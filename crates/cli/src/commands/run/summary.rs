use std::time::SystemTime;

use zygo_core::{
    engine::EngineSnapshot,
    models::{Event, EventKind, WorkflowRunStatus},
};

pub struct WorkflowRunSummary {
    pub workflow_id: String,
    pub workflow_status: String,
    pub job_runs: Vec<JobRunSummary>,
}

pub struct JobRunSummary {
    pub job_id: String,
    pub job_run_id: String,
    pub status: String,
    pub started_at: Option<SystemTime>,
    pub ended_at: Option<SystemTime>,
}

impl WorkflowRunSummary {
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            workflow_status: WorkflowRunStatus::Running.to_string(),
            job_runs: vec![],
        }
    }

    pub fn update_by_snapshot(&mut self, snapshot: &EngineSnapshot) {
        self.workflow_status = snapshot.state.status.to_string();
    }

    pub fn update_by_event(&mut self, event: Event) {
        let timestamp = event.timestamp;

        match event.kind {
            EventKind::JobStarted(data) => {
                let job_run_id = data.job_run_id.to_string();

                if let Some(job_run) = self
                    .job_runs
                    .iter_mut()
                    .find(|job_run| job_run.job_run_id == job_run_id)
                {
                    job_run.job_id = data.job_id.to_string();
                    job_run.status = "running".to_owned();
                    job_run.started_at = Some(timestamp);
                    job_run.ended_at = None;
                } else {
                    self.job_runs.push(JobRunSummary {
                        job_id: data.job_id.to_string(),
                        job_run_id,
                        status: "running".to_owned(),
                        started_at: Some(timestamp),
                        ended_at: None,
                    });
                }
            }
            EventKind::JobSucceeded(data) => self.complete_job_run(
                data.job_id.to_string(),
                data.job_run_id.to_string(),
                "succeeded",
                timestamp,
            ),
            EventKind::JobFailed(data) => self.complete_job_run(
                data.job_id.to_string(),
                data.job_run_id.to_string(),
                "failed",
                timestamp,
            ),
            EventKind::DataReferenceInserted(_) | EventKind::ChannelItemInserted(_) => {}
        }
    }

    fn complete_job_run(
        &mut self,
        job_id: String,
        job_run_id: String,
        status: &str,
        ended_at: SystemTime,
    ) {
        if let Some(job_run) = self
            .job_runs
            .iter_mut()
            .find(|job_run| job_run.job_run_id == job_run_id)
        {
            job_run.job_id = job_id;
            job_run.status = status.to_owned();
            job_run.ended_at = Some(ended_at);
        } else {
            self.job_runs.push(JobRunSummary {
                job_id,
                job_run_id,
                status: status.to_owned(),
                started_at: None,
                ended_at: Some(ended_at),
            });
        }
    }
}
