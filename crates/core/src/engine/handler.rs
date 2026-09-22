use crate::AppDeps;
use crate::api::interface::{RunJobArgs, RunJobResult};
use crate::dependencies::JobRuntime;
use crate::engine::state::EngineState;
use crate::models::{
    ChannelItemInsertedData, Event, EventKind, JobRunId, JobRunStatus, WorkflowSchema, job_run_id,
};

pub struct EventHandlerResult {
    pub new_state: EngineState,
    pub new_events: Vec<Event>,
}

pub struct EventHandler<D: AppDeps> {
    deps: D,
    schema: WorkflowSchema,
}

impl<D: AppDeps> EventHandler<D> {
    pub fn new(deps: D, schema: WorkflowSchema) -> Self {
        Self { deps, schema }
    }

    pub async fn handle(
        &self,
        event: &Event,
        state: &EngineState,
    ) -> Result<EventHandlerResult, anyhow::Error> {
        match &event.kind {
            EventKind::DataReferenceInserted(_) => self.noop(state),
            EventKind::TagInserted(_) => self.noop(state),
            EventKind::ChannelItemInserted(data) => {
                self.handle_channel_item_inserted(state, data).await
            }
            EventKind::JobStarted(data) => {
                self.handle_job_status_update(state, data.job_run_id.clone(), JobRunStatus::Running)
            }
            EventKind::JobSucceeded(data) => self.handle_job_status_update(
                state,
                data.job_run_id.clone(),
                JobRunStatus::Succeeded,
            ),
            EventKind::JobFailed(data) => {
                self.handle_job_status_update(state, data.job_run_id.clone(), JobRunStatus::Failed)
            }
            EventKind::JobEnqueued(data) => {
                self.handle_job_status_update(state, data.job_run_id.clone(), JobRunStatus::Queued)
            }
        }
    }

    fn noop(&self, state: &EngineState) -> Result<EventHandlerResult, anyhow::Error> {
        Ok(EventHandlerResult {
            new_state: state.clone(),
            new_events: vec![],
        })
    }

    async fn handle_channel_item_inserted(
        &self,
        state: &EngineState,
        data: &ChannelItemInsertedData,
    ) -> Result<EventHandlerResult, anyhow::Error> {
        // Find all jobs that have the given channel as an input.
        // Request each job to be run.
        let jobs = self.schema.get_jobs_by_input_channel_id(&data.channel_id);
        let mut new_events = Vec::new();

        for job in jobs {
            let job_run_id = JobRunId::try_from(job_run_id(&job, data.item.as_ref()))?;

            let run_result = self
                .deps
                .runtime()
                .run_job(RunJobArgs {
                    input: data.item.clone(),
                    job_id: job.id.clone(),
                    workflow_run_id: state.id.clone(),
                    job_run_id: job_run_id.clone(),
                })
                .await?;

            match run_result {
                RunJobResult::Enqueued => {}
                RunJobResult::Cached { events } => {
                    new_events.extend(events);
                }
            }
        }

        Ok(EventHandlerResult {
            new_state: state.clone(),
            new_events,
        })
    }

    fn handle_job_status_update(
        &self,
        state: &EngineState,
        job_run_id: JobRunId,
        new_status: JobRunStatus,
    ) -> Result<EventHandlerResult, anyhow::Error> {
        let mut new_state = state.clone();
        new_state.set_job_status(job_run_id, new_status);

        Ok(EventHandlerResult {
            new_state,
            new_events: vec![],
        })
    }
}
