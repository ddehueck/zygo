//! Deterministic generation of resolved workflow schemas.

use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use zygo_core::models::{
    Channel, ChannelId, ContentHash, Edge, EdgeKind, JobId, OrchestratorMode, WorkflowSchema,
};

use crate::generators::Generate;
use crate::generators::job::{JobBlueprint, JobContext, JobGenerator};

pub const SOURCE_CHANNEL_ID: &str = "source";

#[derive(Debug, Clone)]
pub struct WorkflowBlueprint {
    pub content_hash: ContentHash,
    pub input_channel_id: ChannelId,
    pub channels: Vec<ChannelId>,
    pub jobs: Vec<JobBlueprint>,
}

impl From<&WorkflowBlueprint> for WorkflowSchema {
    fn from(blueprint: &WorkflowBlueprint) -> Self {
        let channels = blueprint
            .channels
            .iter()
            .cloned()
            .map(|id| Channel { id })
            .collect();
        let jobs = blueprint.jobs.iter().map(Into::into).collect();
        let edges = blueprint
            .jobs
            .iter()
            .flat_map(|job| {
                let input = Edge::new(job.id.clone(), job.input_channel.clone(), EdgeKind::Input);
                let outputs = job
                    .output_channels
                    .iter()
                    .cloned()
                    .map(|channel_id| Edge::new(job.id.clone(), channel_id, EdgeKind::Output));
                std::iter::once(input).chain(outputs)
            })
            .collect();

        Self {
            content_hash: blueprint.content_hash.clone(),
            input_channel_id: blueprint.input_channel_id.clone(),
            jobs,
            channels,
            edges,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topology {
    Linear,
    FanOut,
}

#[derive(Debug, Clone)]
pub struct WorkflowGenerator {
    pub job: JobGenerator,
    pub topologies: Vec<Topology>,
    pub max_chain_length: usize,
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
    type Output = WorkflowSchema;
    type Context = OrchestratorMode;

    fn generate(&self, rng: &mut ChaCha8Rng, mode: OrchestratorMode) -> WorkflowSchema {
        let topology = *crate::generators::choose(rng, &self.topologies);
        let plan = match topology {
            Topology::Linear => self.linear_plan(rng),
            Topology::FanOut => self.fan_out_plan(rng),
        };
        let blueprint = self.realize(rng, plan, mode);
        WorkflowSchema::from(&blueprint)
    }
}

impl WorkflowGenerator {
    fn realize(
        &self,
        rng: &mut ChaCha8Rng,
        plan: WorkflowPlan,
        mode: OrchestratorMode,
    ) -> WorkflowBlueprint {
        let channels = plan
            .channels
            .iter()
            .map(|name| ChannelId::try_from(name.clone()).expect("valid channel id"))
            .collect::<Vec<_>>();

        let jobs = plan
            .jobs
            .iter()
            .map(|job| {
                self.job.generate(
                    rng,
                    JobContext {
                        id: JobId::try_from(job.id.clone()).expect("valid job id"),
                        input_channel: ChannelId::try_from(job.input.clone())
                            .expect("valid input channel id"),
                        output_channels: job
                            .outputs
                            .iter()
                            .cloned()
                            .map(|output| {
                                ChannelId::try_from(output).expect("valid output channel id")
                            })
                            .collect(),
                        mode,
                    },
                )
            })
            .collect::<Vec<_>>();

        WorkflowBlueprint {
            content_hash: workflow_content_hash(&channels, &jobs),
            input_channel_id: ChannelId::try_from(SOURCE_CHANNEL_ID.to_owned())
                .expect("valid source channel id"),
            channels,
            jobs,
        }
    }

    fn linear_plan(&self, rng: &mut ChaCha8Rng) -> WorkflowPlan {
        let length = rng.random_range(1..=self.max_chain_length.max(1));
        let mut channels = vec![SOURCE_CHANNEL_ID.to_owned()];
        channels.extend((1..=length).map(|index| format!("ch-{index}")));

        let jobs = (0..length)
            .map(|index| JobPlan {
                id: format!("job-{index}"),
                input: if index == 0 {
                    SOURCE_CHANNEL_ID.to_owned()
                } else {
                    format!("ch-{index}")
                },
                outputs: vec![format!("ch-{}", index + 1)],
            })
            .collect();

        WorkflowPlan { channels, jobs }
    }

    fn fan_out_plan(&self, rng: &mut ChaCha8Rng) -> WorkflowPlan {
        let width = rng.random_range(2..=self.max_fan_out.max(2));
        let branches = (1..=width)
            .map(|index| format!("branch-{index}"))
            .collect::<Vec<_>>();

        let mut channels = vec![SOURCE_CHANNEL_ID.to_owned()];
        channels.extend(branches.iter().cloned());
        channels.push(String::from("sink"));

        let mut jobs = vec![JobPlan {
            id: String::from("splitter"),
            input: SOURCE_CHANNEL_ID.to_owned(),
            outputs: branches.clone(),
        }];
        jobs.extend(
            branches
                .into_iter()
                .enumerate()
                .map(|(index, branch)| JobPlan {
                    id: format!("worker-{}", index + 1),
                    input: branch,
                    outputs: vec![String::from("sink")],
                }),
        );

        WorkflowPlan { channels, jobs }
    }
}

fn workflow_content_hash(channels: &[ChannelId], jobs: &[JobBlueprint]) -> ContentHash {
    let mut parts = channels
        .iter()
        .map(|channel| format!("channel:{channel}"))
        .collect::<Vec<_>>();

    for job in jobs {
        parts.push(format!(
            "job:{}:{}:input:{}",
            job.id, job.content_hash, job.input_channel
        ));
        parts.extend(
            job.output_channels
                .iter()
                .map(|channel| format!("output:{}:{channel}", job.id)),
        );
    }

    ContentHash::try_from(parts.join("|")).expect("workflow content hash must be non-empty")
}

struct WorkflowPlan {
    channels: Vec<String>,
    jobs: Vec<JobPlan>,
}

struct JobPlan {
    id: String,
    input: String,
    outputs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::SeedableRng;

    use super::*;

    fn generate(generator: &WorkflowGenerator, seed: u64) -> WorkflowSchema {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        generator.generate(&mut rng, OrchestratorMode::Local)
    }

    fn assert_schema_invariants(schema: &WorkflowSchema) {
        assert!(!schema.jobs.is_empty());
        assert!(schema.channels.iter().any(|channel| {
            channel.id == schema.input_channel_id && channel.id.as_ref() == SOURCE_CHANNEL_ID
        }));

        let channel_ids = schema
            .channels
            .iter()
            .map(|channel| channel.id.clone())
            .collect::<HashSet<_>>();

        for edge in &schema.edges {
            assert!(channel_ids.contains(&edge.channel_id));
            assert!(schema.jobs.iter().any(|job| job.id == edge.job_id));
        }

        for job in &schema.jobs {
            assert!(
                schema
                    .edges
                    .iter()
                    .any(|edge| { edge.job_id == job.id && edge.kind == EdgeKind::Input })
            );
        }
    }

    #[test]
    fn generated_schemas_uphold_graph_invariants() {
        let generator = WorkflowGenerator::default();
        for seed in 0..256 {
            assert_schema_invariants(&generate(&generator, seed));
        }
    }

    #[test]
    fn generation_is_deterministic_for_a_seed() {
        let generator = WorkflowGenerator::default();
        assert_eq!(
            format!("{:?}", generate(&generator, 7)),
            format!("{:?}", generate(&generator, 7))
        );
    }

    #[test]
    fn one_job_linear_schema_has_two_channels_and_two_edges() {
        let generator = WorkflowGenerator {
            topologies: vec![Topology::Linear],
            max_chain_length: 1,
            ..WorkflowGenerator::default()
        };
        let schema = generate(&generator, 1);

        assert_eq!(schema.jobs.len(), 1);
        assert_eq!(schema.channels.len(), 2);
        assert_eq!(schema.edges.len(), 2);
    }
}
