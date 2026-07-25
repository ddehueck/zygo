//! Generation of jobs and their graph wiring.

use rand_chacha::ChaCha8Rng;
use zygo_core::models::{ChannelId, ContentHash, Job, JobEntrypoint, JobId, OrchestratorMode};

use crate::generators::Generate;
use crate::generators::entrypoint::EntrypointGenerator;

#[derive(Debug, Clone)]
pub struct JobBlueprint {
    pub id: JobId,
    pub content_hash: ContentHash,
    pub input_channel: ChannelId,
    pub output_channels: Vec<ChannelId>,
    pub entrypoint: JobEntrypoint,
}

impl From<&JobBlueprint> for Job {
    fn from(blueprint: &JobBlueprint) -> Self {
        Self {
            id: blueprint.id.clone(),
            content_hash: blueprint.content_hash.clone(),
            entrypoint: blueprint.entrypoint.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct JobGenerator {
    pub entrypoint: EntrypointGenerator,
}

#[derive(Debug, Clone)]
pub struct JobContext {
    pub id: JobId,
    pub input_channel: ChannelId,
    pub output_channels: Vec<ChannelId>,
    pub mode: OrchestratorMode,
}

impl Generate for JobGenerator {
    type Output = JobBlueprint;
    type Context = JobContext;

    fn generate(&self, rng: &mut ChaCha8Rng, context: JobContext) -> JobBlueprint {
        let entrypoint = self.entrypoint.generate(rng, context.mode);
        let content_hash = ContentHash::try_from(format!("job:{}:{entrypoint:?}", context.id))
            .expect("generated job content hash must be non-empty");

        JobBlueprint {
            id: context.id,
            content_hash,
            input_channel: context.input_channel,
            output_channels: context.output_channels,
            entrypoint,
        }
    }
}
