use serde::{Deserialize, Serialize};

use crate::models::ids::{ChannelId, JobId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub job_id: JobId,
    pub channel_id: ChannelId,
    pub kind: EdgeKind,
}

impl Edge {
    pub fn new(job_id: JobId, channel_id: ChannelId, kind: EdgeKind) -> Self {
        Self {
            job_id,
            channel_id,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    Input,
    Output,
}
