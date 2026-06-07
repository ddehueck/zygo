//! Generation of [`JobBlueprint`]s: the units of work in a workflow, described
//! purely by the intent the orchestrator's `RegisterWorkflow` request accepts.
//!
//! A job blueprint carries *no* id. The orchestrator is the source of truth for
//! ids; it mints them at registration time and returns them in the response. So
//! the generator only owns the parts that are genuinely the client's to choose:
//! a name and wiring (supplied by the parent
//! [`workflow`](crate::generators::workflow) generator), a content hash, and the
//! [`entrypoint`](crate::generators::entrypoint) that decides how the job runs.

use rand_chacha::ChaCha8Rng;
use zygo_core::models::{ChannelName, ContentHash, JobEntrypoint, JobName, OrchestratorMode};

use crate::generators::Generate;
use crate::generators::entrypoint::EntrypointGenerator;

/// A single job as the orchestrator's registration request describes it: a name,
/// a content hash, the channels it is wired to (by name), and how it executes.
///
/// Wiring is expressed by *channel name* (not id) because that is exactly what
/// the `RegisterWorkflow` request speaks; the orchestrator resolves names to the
/// ids it mints.
#[derive(Debug, Clone)]
pub struct JobBlueprint {
    pub name: JobName,
    pub content_hash: ContentHash,
    pub input_channel: ChannelName,
    pub output_channels: Vec<ChannelName>,
    pub entrypoint: JobEntrypoint,
}

/// Describes how an individual [`JobBlueprint`] is generated.
#[derive(Debug, Clone, Default)]
pub struct JobGenerator {
    /// How the generated job will be executed.
    pub entrypoint: EntrypointGenerator,
}

/// Parent-supplied context for generating a [`JobBlueprint`].
///
/// The name and wiring come from the parent
/// [`workflow`](crate::generators::workflow) generator, which owns the graph
/// topology.
#[derive(Debug, Clone)]
pub struct JobContext {
    pub name: JobName,
    pub input_channel: ChannelName,
    pub output_channels: Vec<ChannelName>,
    /// The orchestrator mode the job's workflow runs under; entrypoints must
    /// match it.
    pub mode: OrchestratorMode,
}

impl Generate for JobGenerator {
    type Output = JobBlueprint;
    type Context = JobContext;

    fn generate(&self, rng: &mut ChaCha8Rng, context: JobContext) -> JobBlueprint {
        // Draw the entrypoint first so the rng cursor advances in a stable
        // order regardless of which fields we assign below.
        let entrypoint = self.entrypoint.generate(rng, context.mode);
        let content_hash =
            ContentHash::try_from(format!("hash-{}", context.name)).expect("valid content hash");

        JobBlueprint {
            name: context.name,
            content_hash,
            input_channel: context.input_channel,
            output_channels: context.output_channels,
            entrypoint,
        }
    }
}
