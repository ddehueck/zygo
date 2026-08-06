use crate::context::{ActorContext, RunContext};
use crate::engine::{EngineSnapshot, StepResult};
use crate::models::StreamItem;
use crate::stream::StreamWriter;
use crate::{engine::Engine, models::Event, store::StorageProvider};
use anyhow::Result;
use tokio::sync::Notify;
use tokio::sync::watch::Receiver;

const ACTOR_MESSAGE_CHANNEL_CAPACITY: usize = 500;
const ACTOR_MESSAGE_BATCH_SIZE: usize = 100;

pub type ActorTx = tokio::sync::mpsc::Sender<ActorMessage>;
pub type ActorRx = tokio::sync::mpsc::Receiver<ActorMessage>;

pub struct ActorMessage {
    pub event: Event,
    pub reply_tx: tokio::sync::oneshot::Sender<Result<(), anyhow::Error>>,
}

pub struct Actor<S: StorageProvider> {
    rx: ActorRx,
    event_notify: Notify,
    context: ActorContext<S>,
    state_tx: tokio::sync::watch::Sender<EngineSnapshot>,
}

#[derive(Clone)]
pub struct ActorHandle {
    pub tx: ActorTx,
    pub state_rx: Receiver<EngineSnapshot>,
}

impl ActorHandle {
    pub async fn spawn<S: StorageProvider>(context: &RunContext<S>) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel::<ActorMessage>(ACTOR_MESSAGE_CHANNEL_CAPACITY);
        let (state_tx, state_rx) = tokio::sync::watch::channel(EngineSnapshot::default());
        let stream_writer = StreamWriter::init(context).await?;

        let actor = Actor {
            rx,
            event_notify: Notify::new(),
            context: ActorContext::from(context, tx.clone(), stream_writer),
            state_tx,
        };

        tokio::spawn(async move { actor.run().await });
        Ok(Self { tx, state_rx })
    }
}

impl<S: StorageProvider> Actor<S> {
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
            // TODO: Should we just pass the writer reference to the job runners?
            _ = Self::event_rx_loop(rx, &context, &event_notify) => {}
        }
    }

    async fn engine_loop(
        context: &ActorContext<S>,
        event_notify: &Notify,
        state_tx: &tokio::sync::watch::Sender<EngineSnapshot>,
    ) {
        let mut engine = match Engine::<S>::new(context.clone()).await {
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
                Ok(StepResult::Continue) => continue,
                Ok(StepResult::Terminal(_)) => break,
                Ok(StepResult::Idle) => event_notify.notified().await,
                Ok(StepResult::WorkerPoolCapacityRequired) => {
                    if let Err(error) = context.worker_pool.wait_for_capacity().await {
                        eprintln!(
                            "failed waiting for worker capacity for run {}: {error}",
                            context.run_id
                        );
                        break;
                    }
                }
                Err(error) => {
                    eprintln!("failed to step engine for run {}: {error}", context.run_id);
                    break;
                }
            }
        }
    }

    async fn event_rx_loop(mut rx: ActorRx, context: &ActorContext<S>, event_notify: &Notify) {
        let stream_writer = &context.stream_writer;

        let mut messages = Vec::with_capacity(ACTOR_MESSAGE_BATCH_SIZE);

        while rx.recv_many(&mut messages, ACTOR_MESSAGE_BATCH_SIZE).await > 0 {
            let (events, reply_txs): (Vec<_>, Vec<_>) = messages
                .drain(..)
                .map(|message| (StreamItem::Event(message.event), message.reply_tx))
                .unzip();

            let result: Result<()> = async {
                let write_set = stream_writer.append(events).await?;
                write_set.commit(&context.store).await?;
                Ok(())
            }
            .await;

            if result.is_ok() {
                event_notify.notify_one();
            }

            let error = result.err().map(|error| format!("{error:#}"));
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
