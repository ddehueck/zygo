mod error;
mod job_runner;
mod local_runner;
mod log;
mod pool;

use crate::{CancellationGroup, actors::ActorTx, models::WorkflowRunId};

pub use error::{Error, Result};
use local_runner::LocalJobRunner;
pub use log::{WorkerLog, WorkerLogReader};
pub use pool::WorkerPool;

struct WorkerContext {
    run_id: WorkflowRunId,
    actor_tx: ActorTx,
    cancellation: CancellationGroup,
}
