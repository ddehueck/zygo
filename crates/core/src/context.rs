use crate::{
    AppDeps,
    actor::ActorTx,
    models::{WorkflowRunId, WorkflowSchema},
};

pub struct RunContext<D: AppDeps> {
    pub deps: D,
    pub run_id: WorkflowRunId,
    pub schema: WorkflowSchema,
}

impl<D: AppDeps> Clone for RunContext<D> {
    fn clone(&self) -> Self {
        Self {
            deps: self.deps.clone(),
            run_id: self.run_id.clone(),
            schema: self.schema.clone(),
        }
    }
}

pub struct ActorContext<D: AppDeps> {
    pub deps: D,
    pub run_id: WorkflowRunId,
    pub schema: WorkflowSchema,
    pub actor_tx: ActorTx,
}

impl<D: AppDeps> ActorContext<D> {
    pub fn from(context: &RunContext<D>, actor_tx: ActorTx) -> Self {
        Self {
            deps: context.deps.clone(),
            run_id: context.run_id.clone(),
            schema: context.schema.clone(),
            actor_tx,
        }
    }
}

impl<D: AppDeps> From<&ActorContext<D>> for RunContext<D> {
    fn from(context: &ActorContext<D>) -> Self {
        Self {
            deps: context.deps.clone(),
            run_id: context.run_id.clone(),
            schema: context.schema.clone(),
        }
    }
}

impl<D: AppDeps> Clone for ActorContext<D> {
    fn clone(&self) -> Self {
        Self {
            deps: self.deps.clone(),
            run_id: self.run_id.clone(),
            actor_tx: self.actor_tx.clone(),
            schema: self.schema.clone(),
        }
    }
}
