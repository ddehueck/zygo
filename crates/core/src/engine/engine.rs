use super::state::EngineState;
use crate::context::ActorContext;
use crate::dependencies::{AppDeps, EventStream};
use crate::engine::handler::EventHandler;
use crate::models::WorkflowRunStatus;
use tokio::sync::watch;

pub struct Engine<D: AppDeps> {
    context: ActorContext<D>,
    handler: EventHandler<D>,
    state: EngineState,
    state_tx: Option<watch::Sender<EngineState>>,
}

impl<D: AppDeps> Engine<D> {
    pub async fn new(context: ActorContext<D>) -> Result<Self, anyhow::Error> {
        let state = EngineState::new(&context.run_id, &context.schema);
        let handler = EventHandler::new(context.deps.clone(), context.schema.clone());

        Ok(Self {
            context,
            handler,
            state,
            state_tx: None,
        })
    }

    /// Execute a single step of the engine.
    /// - Reads the next item from the event stream.
    /// - Handles the event.
    /// - Updates engine state and publishes new events.
    pub async fn step(&mut self) -> Result<EngineStepResult, anyhow::Error> {
        let stream = self.context.deps.stream();
        let next_id = self.state.next_id();

        let Some(event) = stream.get(next_id).await? else {
            return Ok(if self.state.status.is_terminal() {
                EngineStepResult::Terminal(self.state.status.clone())
            } else {
                EngineStepResult::Idle
            });
        };

        // Handle the event
        let result = self.handler.handle(&event, &self.state).await?;

        // Commit results and increment stream cursor
        self.state = result.new_state.clone();
        self.state.increment_cursor();
        if !result.new_events.is_empty() {
            stream.append(result.new_events).await?;
        }

        // Publish state update
        if let Some(tx) = &self.state_tx {
            tx.send(self.state.clone()).ok();
        }

        Ok(EngineStepResult::Continue)
    }

    pub async fn subscribe(&mut self, state_tx: &tokio::sync::watch::Sender<EngineState>) {
        self.state_tx = Some(state_tx.clone());
    }
}

pub enum EngineStepResult {
    Continue,
    Idle,
    Terminal(WorkflowRunStatus),
}
