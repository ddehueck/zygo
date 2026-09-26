mod managed_process;
mod worker;
mod worker_pool;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use crate::api::v0::PythonCli;
use crate::models::{
    DataReferenceUri, Event, EventId, EventKind, JobEnqueuedData, JobFailedData, JobId, JobRunId,
    JobRunSource, Source, WorkflowRunId,
};
use anyhow::{Result, anyhow, bail};
use tokio::sync::{Mutex, mpsc, watch};

use crate::LogsRepository;
use managed_process::ManagedProcess;
use worker::WorkerOutcome;
use worker_pool::WorkerPool;

#[derive(Clone)]
pub struct RunJobArgs {
    pub input: DataReferenceUri,
    pub job_id: JobId,
    pub workflow_run_id: WorkflowRunId,
    pub job_run_id: JobRunId,
}

/// Run-scoped local execution - todo: add caching at this layer.
///
/// Each runtime owns a FIFO worker pool and publishes events to the run actor.
/// The actor projects and handles each event in arrival order.
///
/// Runtime cancellation stops queued and active jobs without publishing a
/// terminal job event, then waits for active process trees to terminate.
pub struct LocalRuntime {
    worker: Worker,
    worker_pool: WorkerPool,
}

struct Worker {
    ctx: Arc<RuntimeContext>,
    state: Arc<RuntimeState>,
}

struct RuntimeContext {
    python_cli: PythonCli,
    run_id: WorkflowRunId,
    events: mpsc::UnboundedSender<Event>,
    logs: LogsRepository,
}

struct RuntimeState {
    jobs: Mutex<HashMap<JobRunId, TrackedJob>>,
    cancelled: AtomicBool,
    cancellation: watch::Sender<bool>,
}

struct TrackedJob {
    completion: watch::Receiver<Option<std::result::Result<(), String>>>,
}

struct JobCompletion {
    sender: watch::Sender<Option<std::result::Result<(), String>>>,
    completed: AtomicBool,
}

impl Clone for LocalRuntime {
    fn clone(&self) -> Self {
        Self {
            worker: self.worker.clone(),
            worker_pool: self.worker_pool.clone(),
        }
    }
}

impl Clone for Worker {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx.clone(),
            state: self.state.clone(),
        }
    }
}

impl TrackedJob {
    async fn wait_for_completion(
        mut completion: watch::Receiver<Option<std::result::Result<(), String>>>,
    ) -> Result<()> {
        loop {
            if let Some(result) = completion.borrow_and_update().clone() {
                return result.map_err(|error| anyhow!(error));
            }
            completion
                .changed()
                .await
                .map_err(|_| anyhow!("local worker stopped without reporting completion"))?;
        }
    }
}

impl JobCompletion {
    fn complete(&self, result: Result<()>) {
        if !self.completed.swap(true, Ordering::SeqCst) {
            self.sender
                .send_replace(Some(result.map_err(|error| format!("{error:#}"))));
        }
    }
}

impl Drop for JobCompletion {
    fn drop(&mut self) {
        if !self.completed.swap(true, Ordering::SeqCst) {
            self.sender.send_replace(Some(Err(
                "local worker stopped without reporting completion".to_owned(),
            )));
        }
    }
}

impl LocalRuntime {
    pub fn new(
        python_cli: PythonCli,
        run_id: WorkflowRunId,
        events: mpsc::UnboundedSender<Event>,
        logs: LogsRepository,
        num_workers: usize,
    ) -> Self {
        let (cancellation, _) = watch::channel(false);
        Self {
            worker: Worker {
                ctx: Arc::new(RuntimeContext {
                    python_cli,
                    run_id,
                    events,
                    logs,
                }),
                state: Arc::new(RuntimeState {
                    jobs: Mutex::new(HashMap::new()),
                    cancelled: AtomicBool::new(false),
                    cancellation,
                }),
            },
            worker_pool: WorkerPool::new(num_workers),
        }
    }

    /// Stop admitting work, terminate all active process trees, drain queued
    /// jobs, and wait for every active worker to finish.
    pub async fn cancel(&self) -> Result<()> {
        let completions = {
            let jobs = self.worker.state.jobs.lock().await;
            self.worker.state.cancelled.store(true, Ordering::SeqCst);
            self.worker_pool.close();
            self.worker.state.cancellation.send_replace(true);
            jobs.values()
                .map(|job| job.completion.clone())
                .collect::<Vec<_>>()
        };

        let pool_error = self.worker_pool.join().await.err();
        let mut first_error = None;
        for completion in completions {
            if let Err(error) = TrackedJob::wait_for_completion(completion).await {
                first_error.get_or_insert(error);
            }
        }
        self.worker.state.jobs.lock().await.clear();

        match first_error.or(pool_error) {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl Worker {
    async fn publish(&self, args: &RunJobArgs, kind: EventKind) -> Result<()> {
        self.ctx
            .events
            .send(Event {
                id: EventId::new(),
                is_replay: false,
                timestamp: SystemTime::now(),
                kind,
                source: Source::JobRun(JobRunSource {
                    job_id: args.job_id.clone(),
                    job_run_id: args.job_run_id.clone(),
                }),
                run_id: self.ctx.run_id.clone(),
            })
            .map_err(|_| anyhow!("workflow actor stopped before accepting job event"))
    }

    async fn fail(&self, args: &RunJobArgs, error: String) -> Result<()> {
        self.publish(
            args,
            EventKind::JobFailed(JobFailedData {
                job_run_id: args.job_run_id.clone(),
                error,
            }),
        )
        .await
    }
}

impl LocalRuntime {
    pub async fn run_job(&self, args: RunJobArgs) -> Result<()> {
        if args.workflow_run_id != self.worker.ctx.run_id {
            bail!("job belongs to a different workflow run");
        }

        let mut jobs = self.worker.state.jobs.lock().await;
        if self.worker.state.cancelled.load(Ordering::SeqCst) {
            bail!("local workflow runtime has been cancelled");
        }
        if self.worker_pool.is_closed() {
            bail!("local worker pool is closed");
        }
        if jobs.contains_key(&args.job_run_id) {
            bail!("job run is already queued or running: {}", args.job_run_id);
        }

        self.worker
            .publish(
                &args,
                EventKind::JobEnqueued(JobEnqueuedData {
                    job_id: args.job_id.clone(),
                    job_run_id: args.job_run_id.clone(),
                    input: args.input.clone(),
                }),
            )
            .await?;

        let cancellation = self.worker.state.cancellation.subscribe();
        let (completion_tx, completion_rx) = watch::channel(None);
        let completion = Arc::new(JobCompletion {
            sender: completion_tx,
            completed: AtomicBool::new(false),
        });
        jobs.insert(
            args.job_run_id.clone(),
            TrackedJob {
                completion: completion_rx,
            },
        );

        let worker = self.worker.clone();
        let cancelled_completion = completion.clone();
        let job_run_id = args.job_run_id.clone();
        let enqueue_result = self.worker_pool.enqueue(
            async move {
                let result = match worker.worker(args.clone(), cancellation).await {
                    Ok(
                        WorkerOutcome::Succeeded
                        | WorkerOutcome::FailedByClient
                        | WorkerOutcome::Cancelled,
                    ) => Ok(()),
                    Err(error) => {
                        let jobs = worker.state.jobs.lock().await;
                        let result =
                            if worker.state.cancelled.load(Ordering::SeqCst) {
                                Err(error)
                            } else {
                                let error_message = format!("{error:#}");
                                match worker.fail(&args, error_message.clone()).await {
                                    Ok(()) => Err(anyhow!(error_message)),
                                    Err(publish_error) => Err(publish_error
                                        .context(format!("job failed: {error_message}"))),
                                }
                            };
                        drop(jobs);
                        result
                    }
                };
                worker.state.jobs.lock().await.remove(&args.job_run_id);
                completion.complete(result);
            },
            move || cancelled_completion.complete(Ok(())),
        );

        if let Err(error) = enqueue_result {
            jobs.remove(&job_run_id);
            return Err(error);
        }

        Ok(())
    }
}

async fn wait_for_cancellation(cancellation: &mut watch::Receiver<bool>) {
    loop {
        if *cancellation.borrow_and_update() {
            return;
        }
        if cancellation.changed().await.is_err() {
            return;
        }
    }
}
