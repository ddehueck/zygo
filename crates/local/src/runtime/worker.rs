use std::sync::atomic::Ordering;

use crate::api::v0::{PythonCli, RunCommandArgs};

use crate::models::EventKind;
use anyhow::{Result, bail};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::watch;

use super::{ManagedProcess, RunJobArgs, Worker, wait_for_cancellation};

pub enum WorkerOutcome {
    Succeeded,
    /// The Python client already published `job_failed` before exiting nonzero.
    FailedByClient,
    Cancelled,
}

impl Worker {
    pub async fn worker(
        &self,
        args: RunJobArgs,
        mut cancellation: watch::Receiver<bool>,
    ) -> Result<WorkerOutcome> {
        if self.state.cancelled.load(Ordering::SeqCst) {
            return Ok(WorkerOutcome::Cancelled);
        }
        self.execute(args, &mut cancellation).await
    }

    async fn execute(
        &self,
        args: RunJobArgs,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<WorkerOutcome> {
        let command_args = RunCommandArgs {
            job_id: args.job_id.to_string(),
            data_reference_uri: args.input.to_string(),
            workflow_run_id: self.ctx.run_id.to_string(),
            job_run_id: args.job_run_id.to_string(),
        };
        let mut command = self
            .ctx
            .python_cli
            .build_run_job_command(command_args, None);

        // One pipe preserves kernel arrival order across stdout and stderr.
        let (reader, writer) = std::io::pipe()?;
        command.stdout(writer.try_clone()?);
        command.stderr(writer);
        let mut process = ManagedProcess::spawn(command)?;

        let work = async { self.process_output(&args, pipe_file(reader)).await };
        let result = tokio::select! {
            biased;
            _ = wait_for_cancellation(cancellation) => {
                process.kill().await?;
                return Ok(WorkerOutcome::Cancelled);
            }
            result = work => result,
        };
        let client_failed = match result {
            Ok(client_failed) => client_failed,
            Err(error) => {
                return Err(terminate_after_error(&mut process, error).await);
            }
        };

        let result = tokio::select! {
            biased;
            _ = wait_for_cancellation(cancellation) => {
                process.kill().await?;
                return Ok(WorkerOutcome::Cancelled);
            }
            result = process.wait() => result,
        };
        let status = match result {
            Ok(status) => status,
            Err(error) => {
                return Err(terminate_after_error(&mut process, error.into()).await);
            }
        };
        if !status.success() {
            if client_failed {
                return Ok(WorkerOutcome::FailedByClient);
            }
            bail!("job process exited with {status}");
        }

        // Serialize terminal success against runtime cancellation so a cancelled
        // worker cannot treat a late exit as success after cancellation.
        let jobs = self.state.jobs.lock().await;
        if self.state.cancelled.load(Ordering::SeqCst) {
            return Ok(WorkerOutcome::Cancelled);
        }
        drop(jobs);

        Ok(WorkerOutcome::Succeeded)
    }

    /// Reads process output and publishes IPC events. Returns whether the client
    /// already emitted `job_failed`.
    async fn process_output(&self, args: &RunJobArgs, pipe: File) -> Result<bool> {
        let mut reader = BufReader::new(pipe);
        let mut line = Vec::new();
        let mut client_failed = false;
        while reader.read_until(b'\n', &mut line).await? != 0 {
            self.ctx
                .logs
                .write_all(&self.ctx.run_id, &args.job_run_id, &line)
                .await?;

            let payload = line.strip_suffix(b"\n").unwrap_or(&line);
            let payload = payload.strip_suffix(b"\r").unwrap_or(payload);
            if let Ok(payload) = std::str::from_utf8(payload) {
                if let Some(message) = PythonCli::parse_run_stdout(payload)? {
                    let kind = message.into_event_kind()?;
                    if matches!(kind, EventKind::JobFailed(_)) {
                        client_failed = true;
                    }
                    self.publish(args, kind).await?;
                }
            }
            line.clear();
        }
        Ok(client_failed)
    }
}

async fn terminate_after_error(
    process: &mut ManagedProcess,
    error: anyhow::Error,
) -> anyhow::Error {
    match process.terminate().await {
        Ok(()) => error,
        Err(cleanup) => error.context(format!("process tree cleanup failed: {cleanup}")),
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
