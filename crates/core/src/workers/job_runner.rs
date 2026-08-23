use crate::{
    context::ActorContext,
    ipc::v0::RunCommandArgs,
    models::{Entrypoint, JobRunSource},
    store::StorageProvider,
    workers::LocalJobRunner,
};

pub struct Runner;

impl Runner {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_job<S: StorageProvider>(
        &self,
        context: ActorContext<S>,
        source: JobRunSource,
        entrypoint: Entrypoint,
        args: RunCommandArgs,
    ) -> Result<(), anyhow::Error> {
        LocalJobRunner::new(context, source, entrypoint, args)
            .run()
            .await
    }
}
