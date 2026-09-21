use crate::context::{ActorContext, RunContext};
use crate::dependencies::EventStream;
use crate::engine::{EngineState, EngineStepResult};
use crate::{AppDeps, engine::Engine, models::Event};
use anyhow::Result;
use tokio::sync::Notify;
use tokio::sync::watch::{Receiver, Sender};

const ACTOR_MESSAGE_CHANNEL_CAPACITY: usize = 500;
const ACTOR_MESSAGE_BATCH_SIZE: usize = 100;

pub type ActorTx = tokio::sync::mpsc::Sender<ActorMessage>;
pub type ActorRx = tokio::sync::mpsc::Receiver<ActorMessage>;

pub type ActorStateRx = Receiver<EngineState>;
pub type ActorStateTx = Sender<EngineState>;

pub struct ActorMessage {
    pub events: Vec<Event>,
    pub reply_tx: tokio::sync::oneshot::Sender<Result<(), anyhow::Error>>,
}

pub struct Actor<D: AppDeps> {
    rx: ActorRx,
    event_notify: Notify,
    context: ActorContext<D>,
    state_tx: tokio::sync::watch::Sender<EngineState>,
}

pub struct ActorHandle {
    pub state_rx: ActorStateRx,
    handle: tokio::task::JoinHandle<()>,
}

impl ActorHandle {
    pub async fn spawn<D: AppDeps>(
        context: &RunContext<D>,
        initial_events: Vec<Event>,
    ) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel::<ActorMessage>(ACTOR_MESSAGE_CHANNEL_CAPACITY);
        let (state_tx, state_rx) =
            tokio::sync::watch::channel(EngineState::new(&context.run_id, &context.schema));

        // Bootstrap the durable stream before the engine can observe a terminal snapshot and exit.
        context.deps.stream().append(initial_events).await?;

        let actor = Actor {
            rx,
            event_notify: Notify::new(),
            context: ActorContext::from(context, tx.clone()),
            state_tx,
        };

        let handle = tokio::spawn(actor.run());

        Ok(Self { state_rx, handle })
    }
}

impl ActorHandle {
    pub async fn cancel(&self) {
        self.handle.abort();
    }
}

impl<D: AppDeps> Actor<D> {
    pub async fn run(self) {
        let Self {
            rx,
            event_notify,
            context,
            state_tx,
        } = self;

        tokio::select! {
            // Dropping the receiver future closes it when the engine reaches a terminal state.
            _ = Self::engine_loop(&context, &event_notify, &state_tx) => {}
            _ = Self::event_rx_loop(rx, &context, &event_notify) => {}
        }
    }

    async fn engine_loop(
        context: &ActorContext<D>,
        event_notify: &Notify,
        state_tx: &ActorStateTx,
    ) {
        let mut engine = match Engine::<D>::new(context.clone()).await {
            Ok(engine) => engine,
            Err(error) => {
                eprintln!(
                    "failed to initialize engine for run {}: {error}",
                    context.run_id
                );
                return;
            }
        };

        // Connect the state watcher to the engine
        engine.subscribe(state_tx).await;

        loop {
            match engine.step().await {
                Ok(EngineStepResult::Continue) => continue,
                Ok(EngineStepResult::Idle) => event_notify.notified().await,
                Ok(EngineStepResult::Terminal(_)) => break,
                Err(error) => {
                    eprintln!("failed to step engine for run {}: {error}", context.run_id);
                    break;
                }
            }
        }
    }

    async fn event_rx_loop(mut rx: ActorRx, context: &ActorContext<D>, event_notify: &Notify) {
        let mut messages = Vec::with_capacity(ACTOR_MESSAGE_BATCH_SIZE);

        while rx.recv_many(&mut messages, ACTOR_MESSAGE_BATCH_SIZE).await > 0 {
            let mut events = Vec::new();
            let mut reply_txs = Vec::with_capacity(messages.len());

            for message in messages.drain(..) {
                events.extend(message.events);
                reply_txs.push(message.reply_tx);
            }

            let append_result = context.deps.stream().append(events).await;
            if append_result.is_ok() {
                event_notify.notify_one();
            }

            let error = append_result.err().map(|error| format!("{error:#}"));
            let mut dropped_replies = 0;

            for reply_tx in reply_txs {
                let result = match &error {
                    Some(error) => Err(anyhow::anyhow!(error.clone())),
                    None => Ok(()),
                };

                if reply_tx.send(result).is_err() {
                    dropped_replies += 1;
                }
            }

            if dropped_replies > 0 {
                eprintln!(
                    "{dropped_replies} event senders dropped their reply channel for run {}",
                    context.run_id
                );
            }
        }
    }
}
