use crate::context::{ActorContext, RunContext};
use crate::models::{StreamItem, StreamRecord};
use crate::store::keyspace::{KeySpace, StoreKey};
use crate::store::{StorageProvider, StoreWriteSet};
use crate::stream::{ReadResult, StreamReader};

use super::arbiter::Arbiter;
use super::executor::Executor;
use super::state::{EngineSnapshot, ResultCache, RunCursor};
use super::step::{StepOutcome, StepResult};

pub struct Engine<S: StorageProvider> {
    context: ActorContext<S>,
    snapshot: EngineSnapshot,
    result_cache: ResultCache<S>,
    arbiter: Arbiter<S>,
    executor: Executor<S>,
    stream_reader: StreamReader<S>,
}

impl<S: StorageProvider> Engine<S> {
    pub async fn new(context: ActorContext<S>) -> Result<Self, anyhow::Error> {
        let workflow_version = context
            .store
            .versions()
            .get(&context.workflow_id, &context.workflow_version_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "workflow version {} for workflow {} does not exist",
                    context.workflow_version_id,
                    context.workflow_id
                )
            })?;

        // TODO: Explicit not found loading vs error handling.
        let engine_snapshot_key = Self::engine_snapshot_key_for_run(&context);
        let snapshot = context
            .store
            .get_json::<EngineSnapshot>(&engine_snapshot_key)
            .await?
            .unwrap_or_else(EngineSnapshot::new);

        let run_context = RunContext::from(&context);
        let result_cache = ResultCache::new(run_context.clone(), workflow_version.schema);
        let stream_reader = StreamReader::new(run_context);
        let executor = Executor::new(context.clone());

        Ok(Self {
            snapshot,
            result_cache,
            executor,
            arbiter: Arbiter::new(context.store.clone()),
            stream_reader,
            context,
        })
    }

    /// Execute a single step of the engine.
    /// - Read the next item from the stream.
    /// - If it's an event, arbitrate to produce commands, increment the sequence id. Flush to durable storage.
    /// - If it's a command, execute it, increment the sequence id. Flush to durable storage.
    pub async fn step(&mut self) -> Result<StepResult, anyhow::Error> {
        let stream_read = self
            .stream_reader
            .next(self.snapshot.cursor.clone())
            .await?;

        let Some(record) = stream_read.record else {
            return Ok(if self.snapshot.state.status.is_terminal() {
                StepResult::Terminal(self.snapshot.state.status.clone())
            } else {
                StepResult::Idle
            });
        };

        // TODO: Should be able to pull this from the stream record?
        let key = KeySpace::workflow(self.context.workflow_id.clone())
            .version(self.context.workflow_version_id.clone())
            .run(self.context.run_id.clone())
            .stream_item(&record.id);

        let outcome = self.evaluate(key, record).await?;
        let snapshot = self.commit(outcome, stream_read.next_cursor).await?;

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
                let commands = self
                    .arbiter
                    .arbitrate(&key, &event, &self.result_cache)
                    .await?;
                append.extend(commands.into_iter().map(StreamItem::Command));
            }
            StreamItem::Command(command) => {
                let result = self
                    .executor
                    .execute(command, &self.result_cache, &next_state)
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

    async fn commit(
        &self,
        outcome: StepOutcome,
        next_cursor: RunCursor,
    ) -> Result<EngineSnapshot, anyhow::Error> {
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
            cursor: next_cursor,
        };
        let mut snapshot_write_set = StoreWriteSet::new();
        snapshot_write_set.add_json(&self.engine_snapshot_key(), &snapshot)?;

        // 3) Commit newly produced stream items and the snapshot atomically.
        // TODO: This api feels funky.
        let mut reservation = self.context.stream_writer.append(append).await?;
        reservation.extend(snapshot_write_set);
        reservation.commit().await?;

        Ok(snapshot)
    }

    // TODO: This should live on the context object?
    fn engine_snapshot_key(&self) -> StoreKey {
        Self::engine_snapshot_key_for_run(&self.context)
    }

    fn engine_snapshot_key_for_run(context: &ActorContext<S>) -> StoreKey {
        KeySpace::workflow(context.workflow_id.clone())
            .version(context.workflow_version_id.clone())
            .run(context.run_id.clone())
            .engine_snapshot()
    }
}
