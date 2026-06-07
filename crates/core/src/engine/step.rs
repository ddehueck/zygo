use crate::models::{RunStatus, SequenceId, StreamItem};

use super::state::RunState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Idle,
    Continue,
    Terminal(RunStatus),
}

pub struct StepOutcome {
    pub processed_id: SequenceId,
    pub next_state: RunState,
    pub append: Vec<StreamItem>,
}
