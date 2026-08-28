use std::sync::Arc;

use tokio::sync::{Semaphore, TryAcquireError};

use crate::{
    context::ActorContext,
    ipc::v0::RunCommandArgs,
    models::{Entrypoint, JobRunSource},
    store::StorageProvider,
    workers::{
        Error::{Closed, NoCapacity},
        Result,
        job_runner::Runner,
    },
};

#[derive(Clone)]
pub struct WorkerPool {
    semaphore: Arc<Semaphore>,
}

impl WorkerPool {
    pub fn new(config_max_workers: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config_max_workers)),
        }
    }

    pub async fn wait_for_capacity(&self) -> Result<()> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Closed)?;
        drop(permit);
        Ok(())
    }

    pub fn spawn_job<S: StorageProvider>(
        &self,
        context: ActorContext<S>,
        source: JobRunSource,
        args: RunCommandArgs,
        entrypoint: Entrypoint,
    ) -> Result<()> {
        if context.cancellation.is_cancelled() {
            return Err(Closed);
        }

        let permit = match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(TryAcquireError::NoPermits) => return Err(NoCapacity),
            Err(TryAcquireError::Closed) => return Err(Closed),
        };

        let task_group = context.cancellation.clone();
        let cancellation = task_group.clone();
        drop(task_group.spawn(async move {
            if cancellation.is_cancelled() {
                drop(permit);
                return;
            }

            if let Err(error) = Runner::new()
                .run_job(context, source, entrypoint, args)
                .await
            {
                // TODO: Better error handling here
                eprintln!("job failed: {error}");
            }
            drop(permit);
        }));

        Ok(())
    }
}
