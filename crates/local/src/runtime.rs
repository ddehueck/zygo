use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use anyhow::{Result, anyhow, bail};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, Semaphore, oneshot, watch};
use zygo_core::api::interface::{RunJobArgs, RunJobResult};
use zygo_core::api::v0::{PythonCli, RunCommandArgs};
use zygo_core::dependencies::{EventStream, JobRuntime};
use zygo_core::models::JobEnqueuedData;
use zygo_core::models::{
    Entrypoint, Event, EventId, EventKind, JobFailedData, JobRunId, JobRunSource, JobStartedData,
    JobSucceededData, Source, WorkflowRunId, WorkflowSchema,
};

use crate::LogsRepository;

/// Run-scoped local execution - todo: add caching at this layer.
///
/// Share the semaphore across runtime instances to enforce a service-wide limit.
/// Writes to event stream directly which is picked up by the polling loop of the core actor.
///
/// Queued cancellation emits `JobFailed`.
/// Active cancellation is unsupported for now. todo: clean up descendants.
#[derive(Clone)]
pub struct LocalRuntime<S> {
    schema: Arc<WorkflowSchema>,
    run_id: WorkflowRunId,
    stream: S,
    logs: LogsRepository,
    semaphore: Arc<Semaphore>,
    jobs: Arc<Mutex<HashMap<JobRunId, RunningJob>>>,
    cancelled: Arc<AtomicBool>,
}

struct RunningJob {
    source: JobRunSource,
    state: JobState,
    completion: watch::Receiver<Option<std::result::Result<(), String>>>,
}

enum JobState {
    Queued(oneshot::Sender<()>),
    Active,
    Cancelling,
}

impl RunningJob {
    fn cancel_queued(&mut self) -> Result<()> {
        if matches!(self.state, JobState::Active) {
            // TODO: Cancel active children and their process groups, drain/reap them,
            // and suppress further worker events before acknowledging cancellation.
            bail!("cancellation of an active local child is not implemented");
        }
        if let JobState::Queued(cancel) = std::mem::replace(&mut self.state, JobState::Cancelling) {
            let _ = cancel.send(());
        }
        Ok(())
    }

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

impl<S: EventStream> LocalRuntime<S> {
    pub fn new(
        schema: WorkflowSchema,
        run_id: WorkflowRunId,
        stream: S,
        logs: LogsRepository,
        semaphore: Arc<Semaphore>,
    ) -> Self {
        Self {
            schema: Arc::new(schema),
            run_id,
            stream,
            logs,
            semaphore,
            jobs: Arc::new(Mutex::new(HashMap::new())),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Stop admitting work and wait for queued jobs to publish their failures.
    /// Active jobs are left running and cause an explicit unsupported-cancellation error.
    pub async fn cancel(&self) -> Result<()> {
        let mut first_error = None;
        let completions = {
            let mut jobs = self.jobs.lock().await;
            self.cancelled.store(true, Ordering::SeqCst);
            let mut completions = Vec::new();
            // Mark every queued job before unlocking, so permit acquisition cannot
            // admit a waiting worker between cancellation requests.
            for job in jobs.values_mut() {
                match job.cancel_queued() {
                    Ok(()) => completions.push(job.completion.clone()),
                    Err(error) => {
                        first_error.get_or_insert(error);
                    }
                }
            }
            completions
        };
        for completion in completions {
            if let Err(error) = RunningJob::wait_for_completion(completion).await {
                first_error.get_or_insert(error);
            }
        }
        // TODO: Wait for active children to stop and be reaped before acknowledging
        // actor cancellation. This lifecycle belongs to the local worker pool.
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    async fn publish(&self, source: &JobRunSource, kind: EventKind) -> Result<()> {
        self.stream
            .append(vec![Event {
                id: EventId::new(),
                is_replay: false,
                timestamp: SystemTime::now(),
                kind,
                source: Source::JobRun(source.clone()),
                run_id: self.run_id.clone(),
            }])
            .await
    }

    async fn fail(&self, source: &JobRunSource, error: String) -> Result<()> {
        self.publish(
            source,
            EventKind::JobFailed(JobFailedData {
                job_id: source.job_id.clone(),
                job_run_id: source.job_run_id.clone(),
                error,
            }),
        )
        .await
    }

    async fn worker(
        &self,
        source: &JobRunSource,
        args: RunJobArgs,
        entrypoint: Entrypoint,
        mut cancel: oneshot::Receiver<()>,
    ) -> Result<()> {
        let permit = tokio::select! {
            biased;
            _ = &mut cancel => bail!("queued job cancelled"),
            permit = self.semaphore.clone().acquire_owned() => permit?,
        };
        {
            let mut jobs = self.jobs.lock().await;
            let Some(job) = jobs.get_mut(&source.job_run_id) else {
                return Ok(());
            };
            // This transition serializes admission against queued cancellation.
            if self.cancelled.load(Ordering::SeqCst) || matches!(job.state, JobState::Cancelling) {
                bail!("queued job cancelled");
            }
            job.state = JobState::Active;
        }
        let result = self.execute(source, args, entrypoint).await;
        drop(permit);
        result
    }

    async fn execute(
        &self,
        source: &JobRunSource,
        args: RunJobArgs,
        entrypoint: Entrypoint,
    ) -> Result<()> {
        let command_args = RunCommandArgs {
            job_id: args.job_id.to_string(),
            data_reference_uri: args.input.to_string(),
            workflow_run_id: self.run_id.to_string(),
            job_run_id: args.job_run_id.to_string(),
        };
        let mut command = match entrypoint {
            Entrypoint::Python(cli) => cli.build_run_job_command(command_args),
        };
        // One pipe preserves kernel arrival order across stdout and stderr.
        let (reader, writer) = std::io::pipe()?;
        command.stdout(writer.try_clone()?);
        command.stderr(writer);
        // TODO: Restore process-group cleanup, including descendants that outlive
        // the leader or keep its output pipe open, when active cancellation lands.
        command.kill_on_drop(true);
        let mut child = command.spawn()?;
        drop(command);

        let result = async {
            self.publish(
                source,
                EventKind::JobStarted(JobStartedData {
                    job_id: source.job_id.clone(),
                    job_run_id: source.job_run_id.clone(),
                    input: args.input,
                }),
            )
            .await?;
            self.process_output(source, pipe_file(reader)).await?;
            let status = child.wait().await?;
            if !status.success() {
                bail!("job process exited with {status}");
            }
            Ok::<_, anyhow::Error>(())
        }
        .await;
        if let Err(error) = result {
            if let Err(cleanup) = child.kill().await {
                return Err(error.context(format!("child cleanup failed: {cleanup}")));
            }
            return Err(error);
        }
        self.publish(
            source,
            EventKind::JobSucceeded(JobSucceededData {
                job_id: source.job_id.clone(),
                job_run_id: source.job_run_id.clone(),
            }),
        )
        .await
    }

    async fn process_output(&self, source: &JobRunSource, pipe: File) -> Result<()> {
        let mut reader = BufReader::new(pipe);
        let mut line = Vec::new();
        while reader.read_until(b'\n', &mut line).await? != 0 {
            self.logs.write_all(&self.run_id, source, &line).await?;
            let payload = line.strip_suffix(b"\n").unwrap_or(&line);
            let payload = payload.strip_suffix(b"\r").unwrap_or(payload);
            if let Ok(payload) = std::str::from_utf8(payload) {
                if let Some(kind) = PythonCli::parse_run_stdout(payload)? {
                    self.publish(source, kind).await?;
                }
            }
            line.clear();
        }
        Ok(())
    }
}

impl<S: EventStream> JobRuntime for LocalRuntime<S> {
    async fn run_job(&self, args: RunJobArgs) -> Result<RunJobResult> {
        if args.workflow_run_id != self.run_id {
            bail!("job belongs to a different workflow run");
        }
        let entrypoint = self
            .schema
            .get_job_entrypoint(&args.job_id)
            .ok_or_else(|| anyhow!("unknown job: {}", args.job_id))?;
        let mut jobs = self.jobs.lock().await;
        if self.cancelled.load(Ordering::SeqCst) {
            bail!("local workflow runtime has been cancelled");
        }
        if self.semaphore.is_closed() {
            bail!("local worker semaphore is closed");
        }
        if jobs.contains_key(&args.job_run_id) {
            bail!("job run is already queued or running: {}", args.job_run_id);
        }
        let source = JobRunSource {
            job_id: args.job_id.clone(),
            job_run_id: args.job_run_id.clone(),
        };
        self.publish(
            &source,
            EventKind::JobEnqueued(JobEnqueuedData {
                job_id: args.job_id.clone(),
                job_run_id: args.job_run_id.clone(),
                input: args.input.clone(),
            }),
        )
        .await?;
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let (completion_tx, completion_rx) = watch::channel(None);
        jobs.insert(
            args.job_run_id.clone(),
            RunningJob {
                source: source.clone(),
                state: JobState::Queued(cancel_tx),
                completion: completion_rx,
            },
        );
        let runtime = self.clone();
        tokio::spawn(async move {
            // The worker owns terminal publication, including queued cancellation
            // and semaphore closure. Cancellation callers never publish a second failure.
            let result = match runtime.worker(&source, args, entrypoint, cancel_rx).await {
                Ok(()) => Ok(()),
                Err(error) => {
                    let error_message = format!("{error:#}");
                    match runtime.fail(&source, error_message.clone()).await {
                        Ok(()) => Ok(()),
                        Err(publish_error) => {
                            Err(publish_error.context(format!("job failed: {error_message}")))
                        }
                    }
                }
            };
            if let Err(error) = &result {
                eprintln!(
                    "failed to publish JobFailed for {}: {error:#}",
                    source.job_run_id
                );
            }
            runtime.jobs.lock().await.remove(&source.job_run_id);
            completion_tx.send_replace(Some(result.map_err(|error| format!("{error:#}"))));
        });
        Ok(RunJobResult::Enqueued)
    }

    async fn stop_job(&self, source: JobRunSource) -> Result<()> {
        let mut jobs = self.jobs.lock().await;
        let Some(job) = jobs.get_mut(&source.job_run_id) else {
            return Ok(());
        };
        if job.source.job_id != source.job_id {
            bail!("job source does not match the queued or running job");
        }
        job.cancel_queued()?;
        let completion = job.completion.clone();
        drop(jobs);
        RunningJob::wait_for_completion(completion).await
    }
}

fn pipe_file(reader: std::io::PipeReader) -> File {
    #[cfg(unix)]
    let file = {
        use std::os::fd::OwnedFd;
        std::fs::File::from(OwnedFd::from(reader))
    };
    #[cfg(windows)]
    let file = {
        use std::os::windows::io::OwnedHandle;
        std::fs::File::from(OwnedHandle::from(reader))
    };
    File::from_std(file)
}
