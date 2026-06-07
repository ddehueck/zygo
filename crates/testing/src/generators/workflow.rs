//! Generation of a workflow graph as a [`WorkflowBlueprint`]: the *intent* the
//! orchestrator's `RegisterWorkflow` request accepts, expressed entirely in
//! terms of names.
//!
//! This is the first real *composition* layer. It picks a topology, decides the
//! sizes within bounded limits (TigerStyle keeps the input space bounded), and
//! then delegates the per-job details to the
//! [`job`](crate::generators::job) generator.
//!
//! # No ids here
//!
//! A blueprint carries no ids. The orchestrator is the source of truth for ids:
//! it mints them at registration and returns them in the response. Generating
//! ids on the client and pretending they are real is the mistake this module is
//! deliberately structured to avoid — wiring is therefore expressed by *channel
//! name*, exactly as the request speaks.
//!
//! # Invariants honored by every generated blueprint
//!
//! - Every job has exactly one input channel (it is a single field).
//! - A job may have multiple output channels.
//! - Every channel is referenced by at least one job.
//! - The graph is acyclic.
//! - There is exactly one entry channel named [`SOURCE_CHANNEL_NAME`], so an
//!   external input always has somewhere to land.

use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use zygo_core::models::{ChannelName, ContentHash, JobName, OrchestratorMode, WorkflowName};
use zygo_core::orchestrator_proto::{ChannelSchema, JobSchema, RegisterWorkflowRequest};

use crate::generators::job::{JobBlueprint, JobContext, JobGenerator};
use crate::generators::Generate;

/// The logical name every generated workflow registers under.
pub const WORKFLOW_NAME: &str = "world";

/// The name of the entry channel that external inputs are inserted into.
pub const SOURCE_CHANNEL_NAME: &str = "source";

/// A whole workflow graph as the orchestrator's registration request describes
/// it: a name, a content hash, the set of channels (by name), and the jobs that
/// connect them.
#[derive(Debug, Clone)]
pub struct WorkflowBlueprint {
    pub name: WorkflowName,
    pub content_hash: ContentHash,
    pub channels: Vec<ChannelName>,
    pub jobs: Vec<JobBlueprint>,
}

/// Turn a generated blueprint into the gRPC request the orchestrator accepts.
///
/// This is the single, named home for "blueprint -> request"; the orchestrator
/// then mints ids and echoes them back in the response.
impl From<&WorkflowBlueprint> for RegisterWorkflowRequest {
    fn from(blueprint: &WorkflowBlueprint) -> Self {
        RegisterWorkflowRequest {
            name: blueprint.name.as_ref().to_string(),
            content_hash: blueprint.content_hash.as_ref().to_string(),
            channels: blueprint
                .channels
                .iter()
                .map(|name| ChannelSchema {
                    name: name.as_ref().to_string(),
                })
                .collect(),
            jobs: blueprint
                .jobs
                .iter()
                .map(|job| JobSchema {
                    name: job.name.as_ref().to_string(),
                    content_hash: job.content_hash.as_ref().to_string(),
                    input_channel_name: job.input_channel.as_ref().to_string(),
                    output_channel_names: job
                        .output_channels
                        .iter()
                        .map(|name| name.as_ref().to_string())
                        .collect(),
                    entrypoint: Some(job.entrypoint.clone().into()),
                })
                .collect(),
        }
    }
}

/// The shapes of workflow graph this generator can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topology {
    /// A straight chain: `source -> job -> channel -> job -> ... -> sink`.
    Linear,
    /// A splitter that fans one input out to several parallel workers that then
    /// converge on a single sink channel.
    FanOut,
}

/// Describes how a whole workflow graph is generated.
#[derive(Debug, Clone)]
pub struct WorkflowGenerator {
    /// How each job in the graph is generated.
    pub job: JobGenerator,
    /// The topologies a draw may select from.
    pub topologies: Vec<Topology>,
    /// Upper bound on the number of jobs in a linear chain.
    pub max_chain_length: usize,
    /// Upper bound on the fan-out width of a splitter.
    pub max_fan_out: usize,
}

impl Default for WorkflowGenerator {
    fn default() -> Self {
        Self {
            job: JobGenerator::default(),
            topologies: vec![Topology::Linear, Topology::FanOut],
            max_chain_length: 4,
            max_fan_out: 3,
        }
    }
}

impl Generate for WorkflowGenerator {
    type Output = WorkflowBlueprint;
    /// The orchestrator mode every job in the graph must conform to.
    type Context = OrchestratorMode;

    fn generate(&self, rng: &mut ChaCha8Rng, mode: OrchestratorMode) -> WorkflowBlueprint {
        let topology = *crate::generators::choose(rng, &self.topologies);
        let plan = match topology {
            Topology::Linear => self.linear_plan(rng),
            Topology::FanOut => self.fan_out_plan(rng),
        };
        self.realize(rng, plan, mode)
    }
}

impl WorkflowGenerator {
    /// Turn an abstract, name-based [`WorkflowPlan`] into a concrete blueprint by
    /// delegating each job to the [`JobGenerator`]. No ids are minted: the plan's
    /// names flow straight into the blueprint.
    fn realize(
        &self,
        rng: &mut ChaCha8Rng,
        plan: WorkflowPlan,
        mode: OrchestratorMode,
    ) -> WorkflowBlueprint {
        let channels = plan
            .channels
            .iter()
            .map(|channel| {
                ChannelName::try_from(channel.name.clone()).expect("valid channel name")
            })
            .collect();

        let jobs: Vec<JobBlueprint> = plan
            .jobs
            .iter()
            .map(|job_plan| {
                let name = JobName::try_from(job_plan.name.clone()).expect("valid job name");
                let input_channel =
                    ChannelName::try_from(job_plan.input.clone()).expect("valid channel name");
                let output_channels = job_plan
                    .outputs
                    .iter()
                    .map(|output| {
                        ChannelName::try_from(output.clone()).expect("valid channel name")
                    })
                    .collect();

                self.job.generate(
                    rng,
                    JobContext {
                        name,
                        input_channel,
                        output_channels,
                        mode,
                    },
                )
            })
            .collect();

        WorkflowBlueprint {
            name: WorkflowName::try_from(WORKFLOW_NAME.to_owned()).expect("valid workflow name"),
            content_hash: workflow_content_hash(&jobs),
            channels,
            jobs,
        }
    }

    /// `source -> job-0 -> ch-1 -> job-1 -> ... -> job-(n-1) -> ch-n`.
    fn linear_plan(&self, rng: &mut ChaCha8Rng) -> WorkflowPlan {
        let length = rng.random_range(1..=self.max_chain_length.max(1));

        let mut channels = vec![ChannelPlan::new(SOURCE_CHANNEL_NAME)];
        for index in 1..=length {
            channels.push(ChannelPlan::new(format!("ch-{index}")));
        }

        let mut jobs = Vec::with_capacity(length);
        for index in 0..length {
            let input = if index == 0 {
                SOURCE_CHANNEL_NAME.to_owned()
            } else {
                format!("ch-{index}")
            };
            jobs.push(JobPlan {
                name: format!("job-{index}"),
                input,
                outputs: vec![format!("ch-{}", index + 1)],
            });
        }

        WorkflowPlan { channels, jobs }
    }

    /// `source -> splitter -> [branch-1..branch-w] -> worker-k -> sink`.
    fn fan_out_plan(&self, rng: &mut ChaCha8Rng) -> WorkflowPlan {
        let width = rng.random_range(2..=self.max_fan_out.max(2));

        let branch_names: Vec<String> = (1..=width).map(|k| format!("branch-{k}")).collect();

        let mut channels = vec![ChannelPlan::new(SOURCE_CHANNEL_NAME)];
        channels.extend(branch_names.iter().map(ChannelPlan::new));
        channels.push(ChannelPlan::new("sink"));

        let mut jobs = vec![JobPlan {
            name: "splitter".to_owned(),
            input: SOURCE_CHANNEL_NAME.to_owned(),
            outputs: branch_names.clone(),
        }];
        for (index, branch) in branch_names.into_iter().enumerate() {
            jobs.push(JobPlan {
                name: format!("worker-{}", index + 1),
                input: branch,
                outputs: vec!["sink".to_owned()],
            });
        }

        WorkflowPlan { channels, jobs }
    }
}

/// A deterministic fingerprint of the workflow definition, derived from the
/// (sorted) content hashes of its jobs.
fn workflow_content_hash(jobs: &[JobBlueprint]) -> ContentHash {
    let mut parts: Vec<String> = jobs.iter().map(|job| job.content_hash.to_string()).collect();
    parts.sort();
    ContentHash::try_from(parts.join(":")).expect("valid content hash")
}

/// An abstract, name-based description of a workflow graph. Keeping naming and
/// wiring in one plain structure is what lets each topology stay small and
/// obviously correct; [`WorkflowGenerator::realize`] then enriches each job with
/// its generated content hash and entrypoint.
struct WorkflowPlan {
    channels: Vec<ChannelPlan>,
    jobs: Vec<JobPlan>,
}

struct ChannelPlan {
    name: String,
}

impl ChannelPlan {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

struct JobPlan {
    name: String,
    input: String,
    outputs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use std::collections::HashSet;

    fn generate(generator: &WorkflowGenerator, seed: u64) -> WorkflowBlueprint {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        generator.generate(&mut rng, OrchestratorMode::Local)
    }

    fn assert_blueprint_invariants(blueprint: &WorkflowBlueprint) {
        assert!(!blueprint.jobs.is_empty(), "blueprint must contain jobs");

        let declared: HashSet<&str> = blueprint
            .channels
            .iter()
            .map(|channel| channel.as_ref())
            .collect();

        // Every job's single input channel is a declared channel.
        for job in &blueprint.jobs {
            assert!(
                declared.contains(job.input_channel.as_ref()),
                "job '{}' input channel must be declared",
                job.name
            );
        }

        // Every declared channel is referenced by at least one job, as an input
        // or an output.
        let wired: HashSet<&str> = blueprint
            .jobs
            .iter()
            .flat_map(|job| {
                std::iter::once(job.input_channel.as_ref())
                    .chain(job.output_channels.iter().map(|name| name.as_ref()))
            })
            .collect();
        for channel in &blueprint.channels {
            assert!(
                wired.contains(channel.as_ref()),
                "channel '{channel}' must be connected to at least one job",
            );
        }

        // Exactly one entry channel named "source".
        let source_channels = blueprint
            .channels
            .iter()
            .filter(|channel| channel.as_ref() == SOURCE_CHANNEL_NAME)
            .count();
        assert_eq!(source_channels, 1, "blueprint has a single source channel");
    }

    #[test]
    fn generated_blueprints_uphold_invariants_across_seeds() {
        let generator = WorkflowGenerator::default();
        for seed in 0..256 {
            assert_blueprint_invariants(&generate(&generator, seed));
        }
    }

    #[test]
    fn generation_is_deterministic_for_a_seed() {
        let generator = WorkflowGenerator::default();
        let first = generate(&generator, 7);
        let second = generate(&generator, 7);
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
    }

    #[test]
    fn linear_topology_produces_a_single_chain() {
        let generator = WorkflowGenerator {
            topologies: vec![Topology::Linear],
            ..WorkflowGenerator::default()
        };
        let blueprint = generate(&generator, 1);

        // A linear chain of n jobs has n + 1 channels.
        assert_eq!(blueprint.channels.len(), blueprint.jobs.len() + 1);
    }
}
