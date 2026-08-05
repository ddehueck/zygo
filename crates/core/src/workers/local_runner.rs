use anyhow::{Result, anyhow};
use std::{path::PathBuf, process::Stdio, time::SystemTime};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::workers::parse_line;
use crate::{
    actors::ActorMessage,
    context::ActorContext,
    models::{
        Event, EventId, EventKind, JobArgs, JobFailedData, JobRunSource, JobStartedData,
        JobSucceededData, LocalEntrypoint, Source,
    },
    store::StorageProvider,
};

/// The local job runner runs the job on the same machine as the orchestrator service.
/// It kicks the job off and monitors stdout for events to send to the workflow run actor.
pub struct LocalJobRunner<S: StorageProvider> {
    context: ActorContext<S>,
    source: JobRunSource,
    args: JobArgs,
    entrypoint: LocalEntrypoint,
}

impl<S: StorageProvider> LocalJobRunner<S> {
    pub fn new(
        context: ActorContext<S>,
        source: JobRunSource,
        args: JobArgs,
        entrypoint: LocalEntrypoint,
    ) -> Self {
        Self {
            context,
            source,
            args,
            entrypoint,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let cwd = PathBuf::from(&self.entrypoint.cwd);
        let exec_cmd = self.entrypoint.exec.clone();
        let job_args_json = serde_json::to_string(&self.args)?;

        // Keep ratatui as the sole terminal writer while the job runs.
        // TODO: Bubble up errors in events.
        let mut child = Command::new(exec_cmd)
            .args(&self.entrypoint.args)
            .arg("--job-args")
            .arg(&job_args_json)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        self.send_job_started_event().await?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("job process stdout was not captured"))?;
        let mut lines = BufReader::new(stdout).lines();

        // TODO: backpressure/error cases when reading a ton of stdout?
        while let Some(line) = lines.next_line().await? {
            if let Some(message) = parse_line(&line)? {
                self.send_event(self.build_event(message.into())).await?;
            }
        }

        let status = child.wait().await?;

        if status.success() {
            self.send_job_succeeded_event().await?;
        } else {
            self.send_job_failed_event().await?;
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
