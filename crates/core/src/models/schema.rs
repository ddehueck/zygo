use serde::{Deserialize, Serialize};

use crate::models::{Channel, ChannelId, ContentHash, Job, JobEntrypoint, JobId, WorkflowId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSchema {
    pub id: WorkflowId, // todo: call this a name.
    pub content_hash: ContentHash,
    pub input_channel_id: ChannelId,
    pub output_channel_id: ChannelId,
    pub jobs: Vec<Job>,
    pub channels: Vec<Channel>,
}

impl WorkflowSchema {
    pub fn get_jobs_by_input_channel_id(&self, channel_id: &ChannelId) -> Vec<Job> {
        self.jobs
            .iter()
            .filter(|job| &job.input_channel_id == channel_id)
            .cloned()
            .collect()
    }

    pub fn get_job_by_id(&self, job_id: &JobId) -> Option<&Job> {
        self.jobs.iter().find(|j| &j.id == job_id)
    }

    pub fn get_job_entrypoint(&self, job_id: &JobId) -> Option<JobEntrypoint> {
        self.jobs
            .iter()
            .find(|j| &j.id == job_id)
            .map(|j| j.entrypoint.clone())
    }
}
