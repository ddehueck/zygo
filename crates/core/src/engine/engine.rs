use std::sync::Arc;

use crate::models::{Event, StreamItem, StreamRecord};
use crate::store::keyspace::{KeySpace, StoreKey};
use crate::store::{StorageProvider, Store, StoreWriteSet};

use super::arbiter::Arbiter;
use super::executor::Executor;
use super::state::{EngineSnapshot, RunContext, RunCursor, RunScope};
use super::step::{StepOutcome, StepResult};
use crate::stream::Stream;

pub struct Engine<S: StorageProvider> {
    store: Arc<Store<S>>,
    context: RunContext,
    snapshot: EngineSnapshot,
    arbiter: Arbiter<S>,
    executor: Executor<S>,
    stream: Stream<S>,
}

impl<S: StorageProvider> Engine<S> {
    pub async fn new(scope: RunScope, store: Store<S>) -> Result<Self, anyhow::Error> {
        let workflow_version = store
            .versions()
            .get(&scope.workflow_id, &scope.workflow_version_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "workflow version {} for workflow {} does not exist",
                    scope.workflow_version_id,
                    scope.workflow_id
                )
            })?;

        // TODO: Explicit not found loading vs error handling.
        let engine_snapshot_key = Self::engine_snapshot_key_for_scope(&scope);
        let snapshot = store
            .get_json::<EngineSnapshot>(&engine_snapshot_key)
            .await?
            .unwrap_or_else(EngineSnapshot::new);

        let context = RunContext::new(scope, workflow_version.schema);
        let stream = Stream::new(store.clone(), context.scope.clone());
        let store = Arc::new(store);

        Ok(Self {
            context,
            snapshot,
            store: Arc::clone(&store),
            executor: Executor::new(Arc::clone(&store)),
            arbiter: Arbiter::new(Arc::clone(&store)),
            stream,
        })
    }

    /// Execute a single step of the engine.
    /// - Read the next item from the stream.
    /// - If it's an event, arbitrate to produce commands, increment the sequence id. Flush to durable storage.
    /// - If it's a command, execute it, increment the sequence id. Flush to durable storage.
    pub async fn step(&mut self) -> Result<StepResult, anyhow::Error> {
        let Some((key, record)) = self.stream.next(&self.snapshot.cursor).await? else {
            return Ok(if self.snapshot.state.status.is_terminal() {
                StepResult::Terminal(self.snapshot.state.status.clone())
            } else {
                StepResult::Idle
            });
        };

        let outcome = self.evaluate(key, record).await?;
        let snapshot = self.commit(outcome).await?;

        self.snapshot = snapshot;

        Ok(StepResult::Continue)
    }

    async fn evaluate(
        &self,
        key: StoreKey,
        record: StreamRecord,
    ) -> Result<StepOutcome, anyhow::Error> {
        let mut next_state = self.snapshot.state.clone();
        let mut append = Vec::new();

        // An event records what has happened
        // A command records what should happen next.
        match record.item {
            StreamItem::Event(event) => {
                let commands = self.arbiter.arbitrate(&key, &event, &self.context).await?;
                append.extend(commands.into_iter().map(StreamItem::Command));
            }
            StreamItem::Command(command) => {
                let result = self
                    .executor
                    .execute(command, &self.context, &next_state)
                    .await?;

                append.extend(result.next_events.into_iter().map(StreamItem::Event));
                next_state = result.next_state;
            }
        }

        Ok(StepOutcome {
            processed_id: record.id,
            next_state,
            append,
        })
    }

    async fn commit(&self, outcome: StepOutcome) -> Result<EngineSnapshot, anyhow::Error> {
        let StepOutcome {
            processed_id,
            next_state,
            append,
        } = outcome;

        // 1) Guard: commit must follow the currently readable sequence item.
        assert!(
            self.snapshot.cursor.next_id <= processed_id,
            "cannot commit run step for {}; next readable item is {}",
            processed_id,
            self.snapshot.cursor.next_id
        );

        // 2) Build the next snapshot and its write set.
        let snapshot = EngineSnapshot {
            state: next_state,
            cursor: RunCursor {
                next_id: processed_id.next(),
            },
        };
        let mut snapshot_write_set = StoreWriteSet::with_capacity(1);
        snapshot_write_set.add_json(&self.engine_snapshot_key(), &snapshot)?;

        // 3) Build stream append write set from newly produced items.
        let stream_write_set = self.stream.append(append).await?;

        // 4) Merge write sets and commit atomically.
        snapshot_write_set.extend(stream_write_set);
        self.store.commit_write_set(snapshot_write_set).await?;

        Ok(snapshot)
    }

    // TODO: This should live on the context object?
    fn engine_snapshot_key(&self) -> StoreKey {
        Self::engine_snapshot_key_for_scope(&self.context.scope)
    }

    fn engine_snapshot_key_for_scope(scope: &RunScope) -> StoreKey {
        KeySpace::workflow(scope.workflow_id.clone())
            .version(scope.workflow_version_id.clone())
            .run(scope.run_id.clone())
            .engine_snapshot()
    }
}
