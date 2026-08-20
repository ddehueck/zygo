use zygo_core::{
    engine::RunCursor,
    models::{EventKind, StreamItem, WorkflowRunId},
    stream::{ReadResult, StreamReader},
};

use crate::db::{KvRepository, WorkflowRunRepository};

/// Processes workflow stream records and projects local read models.
/// While still exposing the underlying stream for local clients. e.g. ui updates.
pub struct LocalStreamProcessor {
    stream_reader: StreamReader<KvRepository>,
    workflow_run_repository: WorkflowRunRepository,
    workflow_run_id: WorkflowRunId,
}

impl LocalStreamProcessor {
    pub fn new(
        // TODO: clean up new args
        stream_reader: StreamReader<KvRepository>,
        workflow_run_repository: WorkflowRunRepository,
        workflow_run_id: WorkflowRunId,
    ) -> Self {
        Self {
            stream_reader,
            workflow_run_repository,
            workflow_run_id,
        }
    }

    pub async fn process_next(&self, cursor: RunCursor) -> anyhow::Result<ReadResult> {
        let result = self.stream_reader.next(cursor).await?;

        match result.record.as_ref().map(|record| &record.item) {
            Some(StreamItem::Event(event)) => match &event.kind {
                EventKind::TagInserted(data) => {
                    self.workflow_run_repository
                        .insert_tag(&self.workflow_run_id.to_string(), &data.name, &data.value)
                        .await?;
                }
                _ => {}
            },
            _ => {}
        }

        Ok(result)
    }
}
