use std::sync::atomic::Ordering;

use crate::api::v0::{JobRunArgs, PythonCli};

use crate::models::{EventKind, JobStartedData, JobSucceededData};
use anyhow::{Result, bail};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::watch;

use super::{ManagedProcess, RunJobArgs, Worker, wait_for_cancellation};

pub enum WorkerOutcome {
    Succeeded,
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
        let command_args = JobRunArgs {
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

        let work = async {
            self.publish(
                &args,
                EventKind::JobStarted(JobStartedData {
                    job_run_id: args.job_run_id.clone(),
                }),
            )
            .await?;
            self.process_output(&args, pipe_file(reader)).await
        };
        let result = tokio::select! {
            biased;
            _ = wait_for_cancellation(cancellation) => {
                process.kill().await?;
                return Ok(WorkerOutcome::Cancelled);
            }
            result = work => result,
        };
        if let Err(error) = result {
            return Err(terminate_after_error(&mut process, error).await);
        }

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
            bail!("job process exited with {status}");
        }

        // Serialize terminal success against runtime cancellation so a cancelled
        // worker cannot treat a late exit as success after cancellation.
        let jobs = self.state.jobs.lock().await;
        if self.state.cancelled.load(Ordering::SeqCst) {
            return Ok(WorkerOutcome::Cancelled);
        }
        self.publish(
            &args,
            EventKind::JobSucceeded(JobSucceededData {
                job_run_id: args.job_run_id.clone(),
            }),
        )
        .await?;
        drop(jobs);

        Ok(WorkerOutcome::Succeeded)
    }

    /// Drains process output and publishes side effects before checking exit status.
    async fn process_output(&self, args: &RunJobArgs, pipe: File) -> Result<()> {
        let mut reader = BufReader::new(pipe);
        let mut line = Vec::new();
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
                    self.publish(args, kind).await?;
                }
            }
            line.clear();
        }
        Ok(())
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
