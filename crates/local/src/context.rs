use tokio::sync::mpsc;

use crate::models::{Event, WorkflowRunId, WorkflowSchema};
use crate::{LocalRuntime, Repos};

pub struct RunContext {
    pub events: mpsc::UnboundedReceiver<Event>,
    pub runtime: LocalRuntime,
    pub repos: Repos,
    pub run_id: WorkflowRunId,
    pub schema: WorkflowSchema,
}
