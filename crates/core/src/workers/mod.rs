mod error;
mod log;
mod pool;
mod runner;

use crate::{AppDeps, actor::ActorTx, cancellation::CancellationGroup, models::WorkflowRunId};

pub struct WorkerContext<D: AppDeps> {
    deps: D,
    run_id: WorkflowRunId,
    actor_tx: ActorTx,
    cancellation: CancellationGroup,
}

pub use error::{Error, Result};
pub use log::{WorkerLog, WorkerLogReader};
pub use pool::WorkerPool;
