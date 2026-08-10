use anyhow::{Result, anyhow};
use std::time::SystemTime;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

use crate::ipc;
use crate::ipc::v0::RunCommandArgs;
use crate::models::{JobEntrypoint, JobRunId};

use crate::workers::WorkerLog;
use crate::{
    actors::ActorMessage,
    context::ActorContext,
    models::{
        Event, EventId, EventKind, JobFailedData, JobRunSource, JobStartedData, JobSucceededData,
        Source,
    },
    store::StorageProvider,
};

/// The local job runner runs the job on the same machine as the orchestrator service.
/// It kicks the job off and monitors stdout for events to send to the workflow run actor.
pub struct LocalJobRunner<S: StorageProvider> {
    context: ActorContext<S>,
    source: JobRunSource,
    entrypoint: JobEntrypoint,
    args: RunCommandArgs, // TODO: Generalize - revist when adding support for other run commands?
}

impl<S: StorageProvider> LocalJobRunner<S> {
    pub fn new(
        context: ActorContext<S>,
        source: JobRunSource,
        entrypoint: JobEntrypoint,
        args: RunCommandArgs,
    ) -> Self {
        Self {
            context,
            source,
            entrypoint,
            args,
        }
    }

    pub async fn run(&self) -> Result<()> {
        // TODO: Bubble up errors in events.
        let mut command = match &self.entrypoint {
            JobEntrypoint::Python(cli) => cli.run_entrypoint(self.args.clone()),
        };

        let worker_log = WorkerLog::new(JobRunId::try_from(self.args.job_run_id.clone())?);
        let mut log_file = worker_log.get_write_file().await?;

        // Both streams share one OS pipe, so order is kernel arrival order; child buffering and
        // concurrent, large, or partial writes can still interleave. Direct log handles would be
        // preferable, but stdout IPC requires interception.
        let (pipe_reader, pipe_writer) = os_pipe::pipe()?;
        let stderr_writer = pipe_writer.try_clone()?;
        command.stdout(pipe_writer);
        command.stderr(stderr_writer);

        let mut child = command.spawn()?;
        drop(command);

        self.send_job_started_event().await?;

        let pipe_reader = Self::async_pipe_reader(pipe_reader);
        self.process_output(pipe_reader, &mut log_file).await?;
        let status = child.wait().await?;

        if status.success() {
            self.send_job_succeeded_event().await?;
        } else {
            self.send_job_failed_event().await?;
        }

        Ok(())
    }

    fn async_pipe_reader(pipe_reader: os_pipe::PipeReader) -> File {
        #[cfg(not(windows))]
        let pipe_file = {
            use std::os::fd::OwnedFd;
            std::fs::File::from(OwnedFd::from(pipe_reader))
        };
        #[cfg(windows)]
        let pipe_file = {
            use std::os::windows::io::OwnedHandle;
            std::fs::File::from(OwnedHandle::from(pipe_reader))
        };

        File::from_std(pipe_file)
    }

    async fn process_output(&self, pipe_reader: File, log_file: &mut File) -> Result<()> {
        let mut reader = BufReader::new(pipe_reader);
        let mut line = Vec::new();

        while reader.read_until(b'\n', &mut line).await? != 0 {
            log_file.write_all(&line).await?;

            let line_without_newline = line.strip_suffix(b"\n").unwrap_or(&line);
            let line_without_newline = line_without_newline
                .strip_suffix(b"\r")
                .unwrap_or(line_without_newline);
            if let Ok(line) = std::str::from_utf8(line_without_newline) {
                match ipc::v0::PythonCli::parse_run_stdout(line) {
                    Ok(Some(kind)) => self.send_event(self.build_event(kind)).await?,
                    Ok(None) => {}
                    Err(error) => return Err(error.into()),
                }
            }

            line.clear();
        }

        log_file.flush().await?;
        Ok(())
    }

    fn build_event(&self, kind: EventKind) -> Event {
        Event {
            id: EventId::new(),
            is_replay: false,
            timestamp: SystemTime::now(),
            kind,
            source: Source::JobRun(self.source.clone()),
            run_id: self.context.run_id.clone(),
        }
    }

    async fn send_event(&self, event: Event) -> Result<()> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

        self.context
            .actor_tx
            .send(ActorMessage { event, reply_tx })
            .await
            .map_err(|_| anyhow!("workflow actor stopped before accepting job event"))?;

        reply_rx
            .await
            .map_err(|_| anyhow!("workflow actor dropped the job event response"))??;

        Ok(())
    }

    async fn send_job_started_event(&self) -> Result<()> {
        let kind = EventKind::JobStarted(JobStartedData {
            job_id: self.source.job_id.clone(),
            job_run_id: self.source.job_run_id.clone(),
        });
        self.send_event(self.build_event(kind)).await
    }

    async fn send_job_succeeded_event(&self) -> Result<()> {
        let kind = EventKind::JobSucceeded(JobSucceededData {
            job_id: self.source.job_id.clone(),
            job_run_id: self.source.job_run_id.clone(),
        });
        self.send_event(self.build_event(kind)).await
    }

    async fn send_job_failed_event(&self) -> Result<()> {
        let kind = EventKind::JobFailed(JobFailedData {
            job_id: self.source.job_id.clone(),
            job_run_id: self.source.job_run_id.clone(),
        });
        self.send_event(self.build_event(kind)).await
    }
}
