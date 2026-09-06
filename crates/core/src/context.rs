use crate::{
    AppDeps, CancellationGroup, actor::ActorTx, models::WorkflowRunId, stream::StreamWriter,
    workers::WorkerPool,
};

pub struct ServiceContext<D: AppDeps> {
    pub deps: D,
    pub worker_pool: WorkerPool<D>,
}

impl<D: AppDeps> ServiceContext<D> {
    pub fn new(deps: D, worker_pool: WorkerPool<D>) -> Self {
        Self { deps, worker_pool }
    }
}

impl<D: AppDeps> Clone for ServiceContext<D> {
    fn clone(&self) -> Self {
        Self {
            deps: self.deps.clone(),
            worker_pool: self.worker_pool.clone(),
        }
    }
}

pub struct RunContext<D: AppDeps> {
    pub deps: D,
    pub worker_pool: WorkerPool<D>,
    pub run_id: WorkflowRunId,
    pub cancellation: CancellationGroup,
}

impl<D: AppDeps> RunContext<D> {
    pub fn new(
        context: &ServiceContext<D>,
        run_id: &WorkflowRunId,
        cancellation: CancellationGroup,
    ) -> Self {
        Self {
            deps: context.deps.clone(),
            worker_pool: context.worker_pool.clone(),
            run_id: run_id.clone(),
            cancellation,
        }
    }
}

impl<D: AppDeps> From<&RunContext<D>> for ServiceContext<D> {
    fn from(context: &RunContext<D>) -> Self {
        Self {
            deps: context.deps.clone(),
            worker_pool: context.worker_pool.clone(),
        }
    }
}

impl<D: AppDeps> Clone for RunContext<D> {
    fn clone(&self) -> Self {
        Self {
            deps: self.deps.clone(),
            worker_pool: self.worker_pool.clone(),
            run_id: self.run_id.clone(),
            cancellation: self.cancellation.clone(),
        }
    }
}

pub struct ActorContext<D: AppDeps> {
    pub deps: D,
    pub worker_pool: WorkerPool<D>,
    pub run_id: WorkflowRunId,
    pub actor_tx: ActorTx,
    pub stream_writer: StreamWriter,
    pub cancellation: CancellationGroup,
}

impl<D: AppDeps> ActorContext<D> {
    pub fn from(context: &RunContext<D>, actor_tx: ActorTx, stream_writer: StreamWriter) -> Self {
        Self {
            deps: context.deps.clone(),
            worker_pool: context.worker_pool.clone(),
            run_id: context.run_id.clone(),
            actor_tx,
            stream_writer,
            cancellation: context.cancellation.clone(),
        }
    }
}

impl<D: AppDeps> From<&ActorContext<D>> for RunContext<D> {
    fn from(context: &ActorContext<D>) -> Self {
        Self {
            deps: context.deps.clone(),
            worker_pool: context.worker_pool.clone(),
            run_id: context.run_id.clone(),
            cancellation: context.cancellation.clone(),
        }
    }
}

impl<D: AppDeps> Clone for ActorContext<D> {
    fn clone(&self) -> Self {
        Self {
            deps: self.deps.clone(),
            worker_pool: self.worker_pool.clone(),
            run_id: self.run_id.clone(),
            actor_tx: self.actor_tx.clone(),
            stream_writer: self.stream_writer.clone(),
            cancellation: self.cancellation.clone(),
        }
    }
}
