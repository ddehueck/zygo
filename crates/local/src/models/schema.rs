use serde::{Deserialize, Serialize};

use crate::models::{Channel, ChannelId, ContentHash, Entrypoint, Job, JobId, WorkflowId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSchema {
    pub id: WorkflowId,         // todo: call this a name.
    pub entrypoint: Entrypoint, // this doesn't really belong here?
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

    pub fn get_job_entrypoint(&self, job_id: &JobId) -> Option<Entrypoint> {
        self.jobs
            .iter()
            .find(|j| &j.id == job_id)
            .map(|j| j.entrypoint.clone())
    }

    // Creates a workflow schema that would only run the specified job.
    pub fn to_job_run(&self, job_id: &JobId) -> Option<WorkflowSchema> {
        let job = self.get_job_by_id(job_id)?.clone();
        let channels = self
            .channels
            .iter()
            .filter(|channel| {
                channel.id == job.input_channel_id || channel.id == job.output_channel_id
            })
            .cloned()
            .collect();

        Some(WorkflowSchema {
            id: self.id.clone(),
            entrypoint: self.entrypoint.clone(),
            content_hash: job.content_hash.clone(),
            input_channel_id: job.input_channel_id.clone(),
            output_channel_id: job.output_channel_id.clone(), // todo: this is a lil awk but workable and may be a rabbit hole I don't want to deal with rn.
            jobs: vec![job],
            channels,
        })
    }
}
