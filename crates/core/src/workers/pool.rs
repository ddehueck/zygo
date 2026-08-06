use std::sync::Arc;

use tokio::sync::{Semaphore, TryAcquireError};

use crate::{
    context::ActorContext,
    models::{JobArgs, JobEntrypoint, JobRunSource},
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
        args: JobArgs,
        entrypoint: JobEntrypoint,
    ) -> Result<()> {
        let permit = match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(TryAcquireError::NoPermits) => return Err(NoCapacity),
            Err(TryAcquireError::Closed) => return Err(Closed),
        };

        tokio::spawn(async move {
            if let Err(e) = Runner::new()
                .run_job(context, source, args, entrypoint)
                .await
            {
                // TODO: Better error handling here
                eprintln!("job failed: {e}");
            }
            drop(permit);
        });

        Ok(())
    }
}
