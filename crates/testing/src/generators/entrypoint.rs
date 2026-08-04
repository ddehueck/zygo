//! Generation of job entrypoints accepted directly by a `WorkflowSchema`.

use std::collections::HashMap;

use rand_chacha::ChaCha8Rng;
use zygo_core::models::{JobEntrypoint, LocalEntrypoint, OrchestratorMode, RemoteEntrypoint};

use crate::generators::{Generate, choose};

/// Describes the space of job entrypoints within an execution mode.
#[derive(Debug, Clone)]
pub struct EntrypointGenerator {
    pub working_dirs: Vec<String>,
    pub local_commands: Vec<String>,
    pub remote_urls: Vec<String>,
}

impl Default for EntrypointGenerator {
    fn default() -> Self {
        Self {
            working_dirs: vec![String::from("/tmp"), String::from("/var/tmp")],
            // LocalJobRunner emits lifecycle events itself. The generated process
            // only needs to exit successfully for the one-job service harness.
            local_commands: vec![String::from("true")],
            remote_urls: vec![
                String::from("https://jobs.dst.local/run"),
                String::from("https://jobs.dst.local/exec"),
            ],
        }
    }
}

impl Generate for EntrypointGenerator {
    type Output = JobEntrypoint;
    type Context = OrchestratorMode;

    fn generate(&self, rng: &mut ChaCha8Rng, mode: OrchestratorMode) -> JobEntrypoint {
        match mode {
            OrchestratorMode::Local => JobEntrypoint::Local(LocalEntrypoint {
                cwd: choose(rng, &self.working_dirs).clone(),
                exec: choose(rng, &self.local_commands).clone(),
            }),
            OrchestratorMode::Remote => JobEntrypoint::Remote(RemoteEntrypoint {
                url: choose(rng, &self.remote_urls).clone(),
                headers: HashMap::new(),
            }),
        }
    }
}
