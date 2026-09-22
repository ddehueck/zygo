use crate::AppDeps;
use crate::models::{WorkflowRunId, WorkflowSchema};

#[derive(Clone)]
pub struct RunContext<D: AppDeps> {
    pub deps: D,
    pub run_id: WorkflowRunId,
    pub schema: WorkflowSchema,
}

pub type ActorContext<D> = RunContext<D>;
