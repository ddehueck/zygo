use crate::models::{SequenceId, StreamItem, WorkflowRunStatus};

use super::state::RunState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Idle,
    Continue,
    Terminal(WorkflowRunStatus),
}

pub struct StepOutcome {
    pub processed_id: SequenceId,
    pub next_state: RunState,
    pub append: Vec<StreamItem>,
}
