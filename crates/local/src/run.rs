use crate::RunContext;
use crate::engine::{Engine, EngineState, EngineStepResult};

use anyhow::Result;
use tokio::sync::watch;

pub type ActorStateRx = watch::Receiver<EngineState>;

pub struct RunHandle {
    pub state_rx: ActorStateRx,
    handle: tokio::task::JoinHandle<()>,
}

impl RunHandle {
    pub fn spawn(context: RunContext) -> Self {
        let initial_state = EngineState::new(&context.run_id, &context.schema);
        let (state_tx, state_rx) = watch::channel(initial_state);

        let run_id = context.run_id.clone();
        let runtime = context.runtime.clone();
        let handle = tokio::spawn(async move {
            if let Err(error) = run_engine(context, &state_tx).await {
                eprintln!("workflow actor {run_id} stopped: {error:#}");
                if let Err(cancel_error) = runtime.cancel().await {
                    eprintln!("workflow actor {run_id} cleanup failed: {cancel_error:#}");
                }
            }
        });

        Self { state_rx, handle }
    }

    pub async fn cancel(&self) {
        self.handle.abort();
    }
}

async fn run_engine(context: RunContext, state_tx: &watch::Sender<EngineState>) -> Result<()> {
    let runtime = context.runtime.clone();
    let mut engine = Engine::new(context);
    engine.subscribe(state_tx).await;

    loop {
        match engine.step().await? {
            EngineStepResult::Continue => {}
            EngineStepResult::Terminal => {
                runtime.cancel().await?;
                return Ok(());
            }
        }
    }
}
