use crate::AppDeps;
use crate::context::RunContext;
use crate::dependencies::EventStream;
use crate::engine::{Engine, EngineState, EngineStepResult};
use crate::models::Event;
use anyhow::Result;
use tokio::sync::watch;
use tokio::time::{Duration, sleep};

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub type ActorStateRx = watch::Receiver<EngineState>;

pub struct RunHandle {
    pub state_rx: ActorStateRx,
    handle: tokio::task::JoinHandle<()>,
}

impl RunHandle {
    pub async fn spawn<D: AppDeps>(
        context: &RunContext<D>,
        initial_events: Vec<Event>,
    ) -> Result<Self> {
        let initial_state = EngineState::new(&context.run_id, &context.schema);
        let (state_tx, state_rx) = watch::channel(initial_state);

        // Bootstrap the durable stream before the engine can observe its end.
        context.deps.stream().append(initial_events).await?;

        // Start the actor task.
        let context = context.clone();
        let handle = tokio::spawn(async move {
            if let Err(error) = run_engine(&context, &state_tx).await {
                eprintln!("workflow actor {} stopped: {error:#}", context.run_id);
            }
        });

        Ok(Self { state_rx, handle })
    }

    pub async fn cancel(&self) {
        self.handle.abort();
    }
}

async fn run_engine<D: AppDeps>(
    context: &RunContext<D>,
    state_tx: &watch::Sender<EngineState>,
) -> Result<()> {
    let mut engine = Engine::new(context.clone()).await?;
    engine.subscribe(state_tx).await;

    loop {
        match engine.step().await? {
            EngineStepResult::Continue => {}
            EngineStepResult::Idle => {
                sleep(EVENT_POLL_INTERVAL).await;
            }
            EngineStepResult::Terminal => {
                return Ok(());
            }
        }
    }
}
