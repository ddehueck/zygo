use crate::{
    ipc::v0::RunCommandArgs,
    models::{Entrypoint, JobRunSource},
    workers::{LocalJobRunner, WorkerContext},
};

pub struct Runner;

impl Runner {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_job(
        &self,
        context: WorkerContext,
        source: JobRunSource,
        entrypoint: Entrypoint,
        args: RunCommandArgs,
    ) -> Result<(), anyhow::Error> {
        LocalJobRunner::new(context, source, entrypoint, args)
            .run()
            .await
    }
}
