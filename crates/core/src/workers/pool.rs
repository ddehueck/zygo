use std::sync::Arc;

use tokio::sync::{Notify, Semaphore};

use crate::{
    context::ActorContext,
    models::{JobArgs, JobEntrypoint, JobRunSource},
    store::StorageProvider,
    workers::job_runner::Runner,
};

#[derive(Clone)]
pub struct WorkerPool {
    notifier: Arc<Notify>,
    semaphore: Arc<Semaphore>,
}

impl WorkerPool {
    pub fn new(config_max_workers: usize) -> Self {
        Self {
            notifier: Arc::new(Notify::new()),
            semaphore: Arc::new(Semaphore::new(config_max_workers)),
        }
    }

    pub fn spawn_job<S: StorageProvider>(
        &self,
        context: ActorContext<S>,
        source: JobRunSource,
        args: JobArgs,
        entrypoint: JobEntrypoint,
    ) -> Result<(), tokio::sync::TryAcquireError> {
        let permit = self.semaphore.clone().try_acquire_owned()?;
        let notifier = self.notifier.clone();

        tokio::spawn(async move {
            if let Err(e) = Runner::new()
                .run_job(context, source, args, entrypoint)
                .await
            {
                // TODO: Better error handling here
                eprintln!("job failed: {e}");
            }
            drop(permit);
            notifier.notify_one();
        });

        Ok(())
    }
}
