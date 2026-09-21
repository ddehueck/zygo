use anyhow::{Result, ensure};
use zygo_core::dependencies::EventStream;
use zygo_core::models::{Event, SequenceId, WorkflowRunId};

use crate::db::EventStreamRepository;

/// A persistent event stream bound to one workflow execution attempt.
#[derive(Clone)]
pub struct LocalEventStream {
    repository: EventStreamRepository,
    run_id: WorkflowRunId,
}

impl LocalEventStream {
    pub fn new(repository: EventStreamRepository, run_id: WorkflowRunId) -> Self {
        Self { repository, run_id }
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            !self.run_id.as_ref().trim().is_empty(),
            "workflow run ID cannot be empty"
        );
        Ok(())
    }

    fn validate_event(&self, event: &Event) -> Result<()> {
        self.validate()?;
        ensure!(
            event.run_id == self.run_id,
            "event run ID {:?} does not match stream run ID {:?}",
            event.run_id,
            self.run_id
        );
        Ok(())
    }
}

impl EventStream for LocalEventStream {
    async fn append(&self, events: Vec<Event>) -> Result<(), anyhow::Error> {
        self.validate()?;
        let mut payloads = Vec::with_capacity(events.len());
        for event in events {
            self.validate_event(&event)?;
            payloads.push(serde_json::to_string(&event)?);
        }
        self.repository
            .append(self.run_id.as_ref(), payloads)
            .await?;
        Ok(())
    }

    async fn get(&self, id: SequenceId) -> Result<Option<Event>, anyhow::Error> {
        self.validate()?;
        let Ok(sequence) = i64::try_from(id.get()) else {
            return Ok(None);
        };
        let Some(payload) = self.repository.get(self.run_id.as_ref(), sequence).await? else {
            return Ok(None);
        };
        let event: Event = serde_json::from_str(&payload)?;
        self.validate_event(&event)?;
        Ok(Some(event))
    }
}
