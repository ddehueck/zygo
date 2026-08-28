use std::{collections::VecDeque, sync::Arc};

use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};

use crate::{
    context::ActorContext,
    ipc::v0::RunCommandArgs,
    models::{Entrypoint, JobRunSource, WorkflowRunId},
    store::StorageProvider,
    workers::{
        Error::{Closed, Unknown},
        Result, WorkerContext,
        job_runner::Runner,
    },
};

#[derive(Clone)]
pub struct WorkerPool {
    semaphore: Arc<Semaphore>,
    queue: Arc<std::sync::Mutex<VecDeque<WorkerPoolMessage>>>,
}

enum WorkerPoolMessage {
    RunJob(QueuedJob),
}

struct QueuedJob {
    context: WorkerContext,
    source: JobRunSource,
    args: RunCommandArgs,
    entrypoint: Entrypoint,
}

impl WorkerPool {
    pub fn new(config_max_workers: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config_max_workers)),
            queue: Arc::new(std::sync::Mutex::new(VecDeque::new())),
        }
    }

    pub fn enqueue_job<S: StorageProvider>(
        &self,
        context: ActorContext<S>,
        source: JobRunSource,
        args: RunCommandArgs,
        entrypoint: Entrypoint,
    ) -> Result<()> {
        if context.cancellation.is_cancelled() {
            return Err(Closed);
        }

        let context = WorkerContext {
            run_id: context.run_id,
            actor_tx: context.actor_tx,
            cancellation: context.cancellation,
        };

        let message = WorkerPoolMessage::RunJob(QueuedJob {
            context,
            source,
            args,
            entrypoint,
        });

        self.queue
            .lock()
            .map_err(|_| Unknown(anyhow::anyhow!("worker queue lock poisoned")))?
            .push_back(message);

        self.dispatch_available()
    }

    pub fn cancel_run(&self, run_id: &WorkflowRunId) -> Result<()> {
        self.queue
            .lock()
            .map_err(|_| Unknown(anyhow::anyhow!("worker queue lock poisoned")))?
            .retain(|message| match message {
                WorkerPoolMessage::RunJob(job) => job.context.run_id != *run_id,
            });

        self.dispatch_available()
    }

    fn dispatch_available(&self) -> Result<()> {
        loop {
            let next = {
                let mut queue = self
                    .queue
                    .lock()
                    .map_err(|_| Unknown(anyhow::anyhow!("worker queue lock poisoned")))?;

                let Some(_) = queue.front() else {
                    return Ok(());
                };

                let permit = match self.semaphore.clone().try_acquire_owned() {
                    Ok(permit) => permit,
                    Err(TryAcquireError::NoPermits) => return Ok(()),
                    Err(TryAcquireError::Closed) => return Err(Closed),
                };
                let message = queue.pop_front().expect("worker queue is not empty");
                (message, permit)
            };

            self.spawn_message(next.0, next.1);
        }
    }

    fn spawn_message(&self, message: WorkerPoolMessage, permit: OwnedSemaphorePermit) {
        let WorkerPoolMessage::RunJob(job) = message;
        let task_group = job.context.cancellation.clone();
        let cancellation = task_group.clone();
        let pool = self.clone();

        drop(task_group.spawn(async move {
            let result = async {
                if cancellation.is_cancelled() {
                    return Ok(());
                }

                Runner::new()
                    .run_job(job.context, job.source, job.entrypoint, job.args)
                    .await
            }
            .await;

            if let Err(error) = result {
                // TODO: Better error handling here
                eprintln!("job failed: {error}");
            }

            drop(permit);

            // Dispatch any remaining queued jobs
            if let Err(error) = pool.dispatch_available() {
                eprintln!("failed to dispatch queued job: {error}");
            }
        }));
    }
}
