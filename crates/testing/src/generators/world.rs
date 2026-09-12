//! Top-level generation of a workflow schema and its candidate inputs.

use rand_chacha::ChaCha8Rng;
use zygo_core::models::{DataReference, OrchestratorMode, WorkflowSchema};

use crate::generators::event::EventGenerator;
use crate::generators::workflow::WorkflowGenerator;
use crate::generators::{Generate, choose};

#[derive(Debug, Clone)]
pub struct World {
    pub schema: WorkflowSchema,
    pub inputs: Vec<DataReference>,
}

#[derive(Debug, Clone)]
pub struct WorldGenerator {
    pub modes: Vec<OrchestratorMode>,
    pub workflow: WorkflowGenerator,
    pub event: EventGenerator,
}

impl Default for WorldGenerator {
    fn default() -> Self {
        Self {
            modes: vec![OrchestratorMode::Local],
            workflow: WorkflowGenerator::default(),
            event: EventGenerator::default(),
        }
    }
}

impl Generate for WorldGenerator {
    type Output = World;
    type Context = ();

    fn generate(&self, rng: &mut ChaCha8Rng, _context: ()) -> World {
        let mode = *choose(rng, &self.modes);
        World {
            schema: self.workflow.generate(rng, mode),
            inputs: self.event.generate(rng, ()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generators::GenerateExt;

    use super::*;

    #[test]
    fn generation_is_deterministic_for_a_seed() {
        let generator = WorldGenerator::default();
        assert_eq!(
            format!("{:?}", generator.generate_seeded(123)),
            format!("{:?}", generator.generate_seeded(123))
        );
    }

    #[test]
    fn generated_world_contains_a_schema_and_inputs() {
        let world = WorldGenerator::default().generate_seeded(99);

        assert!(!world.schema.jobs.is_empty());
        assert!(!world.inputs.is_empty());
        assert!(
            world
                .inputs
                .iter()
                .all(|input| !input.uri.trim().is_empty())
        );
    }
}
