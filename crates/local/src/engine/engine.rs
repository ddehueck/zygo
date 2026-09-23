use super::handler::EventHandler;
use super::state::EngineState;
use crate::RunContext;

use crate::projector::LocalProjector;
use tokio::sync::{mpsc, watch};

pub struct Engine {
    context: RunContext,
    handler: EventHandler,
    projector: LocalProjector,

    state: EngineState,
    state_tx: Option<watch::Sender<EngineState>>,
}

impl Engine {
    pub fn new(context: RunContext) -> Self {
        let state = EngineState::new(&context.run_id, &context.schema);
        let handler = EventHandler::new(context.runtime.clone(), context.schema.clone());
        let projector = LocalProjector::new(context.repos.clone(), context.run_id.clone());

        Self {
            context,
            handler,
            projector,

            state,
            state_tx: None,
        }
    }

    pub async fn step(&mut self) -> Result<EngineStepResult, anyhow::Error> {
        let event = match self.context.events.try_recv() {
            Ok(event) => event,
            Err(mpsc::error::TryRecvError::Empty) if self.state.status.is_terminal() => {
                return Ok(EngineStepResult::Terminal);
            }
            Err(mpsc::error::TryRecvError::Empty) => self
                .context
                .events
                .recv()
                .await
                .ok_or_else(|| anyhow::anyhow!("workflow event producer stopped"))?,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                return Err(anyhow::anyhow!("workflow event producer stopped"));
            }
        };

        anyhow::ensure!(
            event.run_id == self.state.id,
            "event belongs to a different workflow run"
        );

        // Do stuff with the event
        self.projector.project(&event).await?;
        let new_state = self.handler.handle(&event, &self.state).await?;

        // Update state and notify subscribers
        self.state = new_state;

        if let Some(tx) = &self.state_tx {
            tx.send_replace(self.state.clone());
        }

        Ok(EngineStepResult::Continue)
    }

    pub async fn subscribe(&mut self, state_tx: &tokio::sync::watch::Sender<EngineState>) {
        self.state_tx = Some(state_tx.clone());
    }
}

pub enum EngineStepResult {
    Continue,
    Terminal,
}
