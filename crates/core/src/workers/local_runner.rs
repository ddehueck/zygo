use anyhow::{Result, anyhow};
use std::{process::Stdio, time::SystemTime};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::ipc;
use crate::ipc::v0::RunCommandArgs;
use crate::models::JobEntrypoint;
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
    args: RunCommandArgs, // TODO: Generalize?
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

        command.stdout(Stdio::piped());
        command.stderr(Stdio::inherit());

        let mut child = command.spawn()?;

        self.send_job_started_event().await?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("job process stdout was not captured"))?;
        let mut lines = BufReader::new(stdout).lines();

        // TODO: backpressure/error cases when reading a ton of stdout?
        while let Some(line) = lines.next_line().await? {
            // TODO: This needs to either be independent of the Python CLI or
            // in a shared interface.
            match ipc::v0::PythonCli::parse_run_stdout(&line) {
                Ok(Some(kind)) => self.send_event(self.build_event(kind)).await?,
                Ok(None) => {}
                Err(e) => return Err(e.into()),
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
