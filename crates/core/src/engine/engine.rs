use super::arbiter::Arbiter;
use super::executor::{ExecuteOutcome, Executor};
use super::state::{EngineSnapshot, ResultCache, RunCursor};
use super::step::{StepOutcome, StepResult};
use crate::context::{ActorContext, RunContext};
use crate::models::{StreamItem, StreamRecord, WorkflowSchema};
use crate::store::keyspace::KeySpace;
use crate::store::{StorageProvider, StoreKey};
use crate::stream::StreamReader;

enum EvaluateOutcome {
    Completed(StepOutcome),
    WorkerPoolCapacityRequired,
}

pub struct Engine<S: StorageProvider> {
    context: ActorContext<S>,
    snapshot: EngineSnapshot,
    result_cache: ResultCache<S>,
    arbiter: Arbiter,
    executor: Executor<S>,
    stream_reader: StreamReader<S>,
}

impl<S: StorageProvider> Engine<S> {
    pub async fn new(context: ActorContext<S>) -> Result<Self, anyhow::Error> {
        let run_keyspace = KeySpace::run(&context.run_id);
        let schema_key = run_keyspace.schema();
        let schema_value = context.store.get(&schema_key).await?.ok_or_else(|| {
            anyhow::anyhow!(
                "workflow schema is missing for run {}; store the run schema before starting the engine",
                context.run_id
            )
        })?;
        let schema = serde_json::from_value::<WorkflowSchema>(schema_value).map_err(|error| {
            anyhow::anyhow!(
                "failed to deserialize workflow schema for run {}: {error}",
                context.run_id
            )
        })?;

        let snapshot_key = run_keyspace.snapshot();
        let snapshot = context
            .store
            .get(&snapshot_key)
            .await?
            .map(serde_json::from_value::<EngineSnapshot>)
            .transpose()
            .map_err(|error| {
                anyhow::anyhow!(
                    "failed to deserialize engine snapshot for run {}: {error}",
                    context.run_id
                )
            })?
            .unwrap_or_else(EngineSnapshot::new);

        let run_context = RunContext::from(&context);
        let result_cache = ResultCache::new(run_context, schema);
        let stream_reader = StreamReader::new(context.store.clone(), &context.run_id);
        let executor = Executor::new(context.clone());

        Ok(Self {
            snapshot,
            result_cache,
            executor,
            arbiter: Arbiter,
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

        let key = KeySpace::run(&self.context.run_id).stream_item(&record.id);

        let outcome = match self.evaluate(key, record).await? {
            EvaluateOutcome::Completed(outcome) => outcome,
            EvaluateOutcome::WorkerPoolCapacityRequired => {
                return Ok(StepResult::WorkerPoolCapacityRequired);
            }
        };
        let snapshot = self.commit(outcome, stream_read.next_cursor).await?;

        self.snapshot = snapshot;

        Ok(StepResult::Continue)
    }

    async fn evaluate(
        &self,
        key: StoreKey,
        record: StreamRecord,
    ) -> Result<EvaluateOutcome, anyhow::Error> {
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
                match self
                    .executor
                    .execute(command, &self.result_cache, &next_state)
                    .await?
                {
                    ExecuteOutcome::Completed(result) => {
                        append.extend(result.next_events.into_iter().map(StreamItem::Event));
                        next_state = result.next_state;
                    }
                    ExecuteOutcome::WorkerPoolCapacityRequired => {
                        return Ok(EvaluateOutcome::WorkerPoolCapacityRequired);
                    }
                }
            }
        }

        Ok(EvaluateOutcome::Completed(StepOutcome {
            processed_id: record.id,
            next_state,
            append,
        }))
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

        // 2) Build the next snapshot.
        let snapshot = EngineSnapshot {
            state: next_state,
            cursor: next_cursor,
        };

        // 3) Commit newly produced stream items and the snapshot atomically.
        let mut write_set = self.context.stream_writer.append(append).await?;
        write_set.push(self.engine_snapshot_key(), serde_json::to_value(&snapshot)?);
        write_set.commit(&self.context.store).await?;

        Ok(snapshot)
    }

    fn engine_snapshot_key(&self) -> StoreKey {
        KeySpace::run(&self.context.run_id).snapshot()
    }
}
