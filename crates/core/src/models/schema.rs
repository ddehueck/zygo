use serde::{Deserialize, Serialize};

use crate::models::{Channel, ChannelId, ContentHash, Edge, EdgeKind, Job, JobEntrypoint, JobId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSchema {
    pub content_hash: ContentHash,
    pub input_channel_id: ChannelId,
    pub entrypoint: JobEntrypoint, // Only one per workflow as we call the python module run with the job/ref as args
    pub jobs: Vec<Job>,
    pub channels: Vec<Channel>,
    pub edges: Vec<Edge>,
}

impl WorkflowSchema {
    pub fn get_jobs_by_input_channel_id(&self, channel_id: &ChannelId) -> Vec<Job> {
        self.edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::Input && edge.channel_id == *channel_id)
            .filter_map(|edge| self.jobs.iter().find(|job| job.id == edge.job_id))
            .cloned()
            .collect()
    }

    pub fn get_channels_for_job(&self, job_id: &JobId) -> Vec<&Channel> {
        self.edges
            .iter()
            .filter(|edge| &edge.job_id == job_id)
            .filter_map(|edge| self.channels.iter().find(|c| c.id == edge.channel_id))
            .collect()
    }

    pub fn get_job_by_id(&self, job_id: &JobId) -> Option<&Job> {
        self.jobs.iter().find(|j| &j.id == job_id)
    }

    pub fn get_job_entrypoint(&self, job_id: &JobId) -> Option<JobEntrypoint> {
        // Fake one for now while we refactor entrpoints
        None
        // self.jobs
        //     .iter()
        //     .find(|j| &j.id == job_id)
        //     .map(|j| j.entrypoint.clone())
    }
}
