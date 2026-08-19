use serde::{Deserialize, Serialize};

use crate::models::{ChannelId, ContentHash, Job, JobEntrypoint, JobId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSchema {
    pub content_hash: ContentHash,
    pub input_channel_id: ChannelId,
    pub output_channel_id: ChannelId,
    pub jobs: Vec<Job>,
}

impl WorkflowSchema {
    pub fn get_jobs_by_input_channel_id(&self, channel_id: &ChannelId) -> Vec<Job> {
        self.jobs
            .iter()
            .filter(|job| &job.input_channel_id == channel_id)
            .cloned()
            .collect()
    }

    pub fn get_channels_for_job(&self, job_id: &JobId) -> Vec<&ChannelId> {
        self.jobs
            .iter()
            .find(|job| &job.id == job_id)
            .map(|job| vec![&job.input_channel_id, &job.output_channel_id])
            .unwrap_or_default()
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
