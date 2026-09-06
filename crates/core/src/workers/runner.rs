use anyhow::{Result, anyhow};
use command_group::{AsyncCommandGroup, AsyncGroupChild};
use std::io::ErrorKind;
use std::process::ExitStatus;
use std::time::{Duration, SystemTime};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::actor::ActorMessage;
use crate::ipc::{self, v0::RunCommandArgs};
use crate::models::{
    Entrypoint, Event, EventId, EventKind, JobFailedData, JobRunSource, JobStartedData,
    JobSucceededData, Source,
};
use crate::workers::WorkerContext;
use crate::{AppDeps, LogWriter};

const PROCESS_WAIT_POLL_INTERVAL: Duration = Duration::from_millis(25);

/// The local job runner runs the job on the same machine as the orchestrator service.
/// It kicks the job off and monitors stdout for events to send to the workflow run actor.
pub struct JobRunner<D: AppDeps> {
    context: WorkerContext<D>,
    source: JobRunSource,
    entrypoint: Entrypoint,
    args: RunCommandArgs, // TODO: Generalize - revist when adding support for other run commands?
}

impl<D: AppDeps> JobRunner<D> {
    pub fn new(
        context: WorkerContext<D>,
        source: JobRunSource,
        entrypoint: Entrypoint,
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
            Entrypoint::Python(cli) => cli.run_entrypoint(self.args.clone()),
        };

        // Both streams share one OS pipe, so order is kernel arrival order; child buffering and
        // concurrent, large, or partial writes can still interleave. Direct log handles would be
        // preferable, but stdout IPC requires interception.
        let (pipe_reader, pipe_writer) = os_pipe::pipe()?;
        let stderr_writer = pipe_writer.try_clone()?;
        command.stdout(pipe_writer);
        command.stderr(stderr_writer);

        if self.context.cancellation.is_cancelled() {
            return Ok(());
        }

        let mut child = command.group_spawn()?;
        drop(command);

        // todo: don't let cancellation pollute code to this level.
        let started = tokio::select! {
            biased;
            _ = self.context.cancellation.cancelled() => None,
            result = self.send_job_started_event() => Some(result),
        };
        match started {
            None => {
                Self::kill_process_group(&mut child).await?;
                return Ok(());
            }
            Some(Ok(())) => {}
            Some(Err(error)) => {
                return Err(Self::terminate_after_error(&mut child, error).await);
            }
        }

        let pipe_reader = Self::async_pipe_reader(pipe_reader);
        let output = tokio::select! {
            biased;
            _ = self.context.cancellation.cancelled() => None,
            result = self.process_output(pipe_reader) => Some(result),
        };
        match output {
            None => {
                Self::kill_process_group(&mut child).await?;
                return Ok(());
            }
            Some(Ok(())) => {}
            Some(Err(error)) => {
                return Err(Self::terminate_after_error(&mut child, error).await);
            }
        }

        let status = match self.wait_for_exit_or_cancel(&mut child).await {
            Ok(status) => status,
            Err(error) => {
                return Err(Self::terminate_after_error(&mut child, error).await);
            }
        };
        let Some(status) = status else {
            return Ok(());
        };

        // The process-group leader may exit while a descendant that closed its inherited output
        // handles keeps running. Always clear any residual group members before releasing the
        // worker task, including when cancellation races with leader exit.
        Self::kill_process_group(&mut child).await?;

        if self.context.cancellation.is_cancelled() {
            return Ok(());
        }

        if status.success() {
            self.send_job_succeeded_event().await?;
        } else {
            self.send_job_failed_event().await?;
        }

        Ok(())
    }

    async fn wait_for_exit_or_cancel(
        &self,
        child: &mut AsyncGroupChild,
    ) -> Result<Option<ExitStatus>> {
        let mut poll = tokio::time::interval(PROCESS_WAIT_POLL_INTERVAL);
        poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            // Poll only the group leader here. Polling AsyncGroupChild::try_wait would cache the
            // leader status in the wrapper, causing a later group kill to skip waiting for any
            // residual descendants.
            if let Some(status) = child.inner().try_wait()? {
                return Ok(Some(status));
            }

            tokio::select! {
                biased;
                _ = self.context.cancellation.cancelled() => {
                    Self::kill_process_group(child).await?;
                    return Ok(None);
                }
                _ = poll.tick() => {}
            }
        }
    }

    async fn terminate_after_error(
        child: &mut AsyncGroupChild,
        error: anyhow::Error,
    ) -> anyhow::Error {
        match Self::kill_process_group(child).await {
            Ok(()) => error,
            Err(cleanup_error) => error.context(format!(
                "additionally failed to terminate worker process group: {cleanup_error}"
            )),
        }
    }

    async fn kill_process_group(child: &mut AsyncGroupChild) -> Result<()> {
        match child.kill().await {
            Ok(()) => Ok(()),
            Err(error) if Self::process_group_is_gone(&error) => {
                // The process group won the race and exited before the kill request.
                child.try_wait()?;
                Ok(())
            }
            Err(error) => Err(error.into()),
        }
    }

    fn process_group_is_gone(error: &std::io::Error) -> bool {
        if matches!(error.kind(), ErrorKind::InvalidInput | ErrorKind::NotFound) {
            return true;
        }

        // POSIX killpg reports ESRCH when no process remains in the group. Rust currently maps
        // that value to Uncategorized rather than NotFound on Unix targets.
        #[cfg(unix)]
        if error.raw_os_error() == Some(3) {
            return true;
        }

        false
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

    async fn process_output(&self, pipe_reader: File) -> Result<()> {
        let mut reader = BufReader::new(pipe_reader);
        let mut line = Vec::new();

        while reader.read_until(b'\n', &mut line).await? != 0 {
            self.context
                .deps
                .log_writer()
                .write_all(&self.context.run_id, &self.source, &line)
                .await?;

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
            .send(ActorMessage {
                events: vec![event],
                reply_tx,
            })
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
