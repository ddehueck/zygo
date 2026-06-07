use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{Channel, ChannelId, ChannelName, Edge, Job, JobEntrypoint, JobId};

use super::ids::{ContentHash, WorkflowId, WorkflowVersionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersion {
    pub id: WorkflowVersionId,
    pub workflow_id: WorkflowId,
    pub content_hash: ContentHash,
    pub schema: WorkflowVersionSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersionSchema {
    pub jobs: Vec<Job>,
    pub channels: Vec<Channel>,
    pub edges: Vec<Edge>,
}

impl WorkflowVersionSchema {
    pub fn new(jobs: Vec<Job>, channels: Vec<Channel>, edges: Vec<Edge>) -> Self {
        Self {
            jobs,
            channels,
            edges,
        }
    }

    pub fn get_jobs_by_input_channel_id(&self, channel_id: &ChannelId) -> Vec<Job> {
        self.edges
            .iter()
            .filter(|edge| edge.channel_id == *channel_id)
            .map(|edge| self.jobs.iter().find(|job| job.id == edge.job_id).unwrap())
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
        self.jobs
            .iter()
            .find(|j| &j.id == job_id)
            .map(|j| j.entrypoint.clone())
    }

    pub fn get_channel_ids_by_name(&self) -> HashMap<ChannelName, ChannelId> {
        self.channels
            .iter()
            .map(|c| (c.name.clone(), c.id.clone()))
            .collect()
    }
}
