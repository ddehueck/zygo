use crate::context::{ActorContext, RunContext};
use crate::engine::StepResult;
use crate::models::StreamItem;
use crate::stream::StreamWriter;
use crate::{engine::Engine, models::Event, store::StorageProvider};
use anyhow::Result;
use tokio::sync::Notify;

const ACTOR_MESSAGE_CHANNEL_CAPACITY: usize = 500;

pub type ActorTx = tokio::sync::mpsc::Sender<ActorMessage>;
pub type ActorRx = tokio::sync::mpsc::Receiver<ActorMessage>;

pub struct ActorMessage {
    pub event: Event,
    pub reply_tx: tokio::sync::oneshot::Sender<Result<(), anyhow::Error>>,
}

pub struct Actor<S: StorageProvider> {
    rx: ActorRx,
    event_notify: Notify,
    resource_notify: Notify,
    context: ActorContext<S>,
}

#[derive(Clone)]
pub struct ActorHandle {
    pub tx: ActorTx,
}

impl ActorHandle {
    pub async fn spawn<S: StorageProvider>(run_context: RunContext<S>) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel::<ActorMessage>(ACTOR_MESSAGE_CHANNEL_CAPACITY);
        let stream_writer = StreamWriter::init(&run_context).await?;

        let actor = Actor {
            rx,
            event_notify: Notify::new(),
            resource_notify: Notify::new(), // TODO: Should this come from the worker pool?
            context: ActorContext::from(&run_context, tx.clone(), stream_writer),
        };

        tokio::spawn(async move { actor.run().await });
        Ok(Self { tx })
    }
}

impl<S: StorageProvider> Actor<S> {
    pub async fn run(self) {
        let Self {
            rx,
            event_notify,
            context,
            ..
        } = self;

        tokio::select! {
            // Dropping the receiver future closes it when the engine reaches a terminal state.
            _ = Self::engine_loop(&context, &event_notify) => {}
            _ = Self::event_rx_loop(rx, context.clone(), &event_notify) => {}
        }
    }

    async fn engine_loop(context: &ActorContext<S>, event_notify: &Notify) {
        // TODO: Better error handling?
        let mut engine = Engine::<S>::new(context.clone()).await.unwrap();

        loop {
            match engine.step().await.expect("failed to step engine") {
                StepResult::Idle => event_notify.notified().await,
                StepResult::Continue => continue,
                StepResult::Terminal(_) => break,
                // StepResult::BlockedByWorkerPool => spawn::fn(on_notify => StepEngine)
            }
        }
    }

    async fn event_rx_loop(mut rx: ActorRx, context: ActorContext<S>, event_notify: &Notify) {
        let stream_writer = context.stream_writer;

        // TODO: Batch.
        while let Some(message) = rx.recv().await {
            let ActorMessage { event, reply_tx } = message;

            // TODO: Send an error to the reply if needed but continue receiving messages.
            if let Ok(reservation) = stream_writer.append(vec![StreamItem::Event(event)]).await {
                reservation.commit().await.ok();
            }

            event_notify.notify_one();
            // TODO: Error handling here? Nothing we quite can do if the the client rx is dropped.
            // We should debug log and continue. Client may send a retry but that should be idempotent.
            reply_tx.send(Ok(())).ok();
        }
    }
}
