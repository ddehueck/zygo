use crate::{
    context::ActorContext,
    models::{JobArgs, JobEntrypoint, JobRunSource},
    store::StorageProvider,
    workers::local_runner::LocalJobRunner,
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
        args: JobArgs,
        entrypoint: JobEntrypoint,
    ) -> Result<(), anyhow::Error> {
        match entrypoint {
            JobEntrypoint::Local(entrypoint) => {
                LocalJobRunner::new(context, source, args, entrypoint)
                    .run()
                    .await
            }
            JobEntrypoint::Remote(_) => panic!("remote job entrypoints are not implemented"),
        }
    }
}
