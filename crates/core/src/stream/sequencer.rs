use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::AppDeps;
use crate::context::RunContext;
use crate::dependencies::StorageProvider;
use crate::models::SequenceId;
use crate::store::{KeySpace, RunKeySpace, StoreKey, WriteSetReservation};

#[derive(Clone)]
pub struct StreamSequencer {
    keyspace: RunKeySpace,
    tail: Arc<Mutex<SequenceId>>,
}

impl StreamSequencer {
    pub async fn load<D: AppDeps>(context: RunContext<D>) -> Result<Self> {
        let run_keyspace = KeySpace::run(&context.run_id);

        let tail_key = run_keyspace.tail();
        let tail_value = context.deps.store().get(&tail_key).await?;

        let tail = match tail_value {
            Some(value) => serde_json::from_value::<SequenceId>(value)?,
            None => SequenceId::new(0),
        };

        Ok(Self {
            keyspace: run_keyspace,
            tail: Arc::new(Mutex::new(tail)),
        })
    }

    pub async fn reserve(&self, n: usize) -> Result<SequenceReservation> {
        if n == 0 {
            return Err(anyhow::anyhow!("n must be greater than 0"));
        }

        let mut tail = Arc::clone(&self.tail).lock_owned().await;

        let start = *tail;

        let end_exclusive = start
            .get()
            .checked_add(n as u64)
            .map(SequenceId::new)
            .ok_or_else(|| anyhow::anyhow!("sequence overflow"))?;

        let sequence_range = start.get()..end_exclusive.get();

        *tail = end_exclusive;

        let tail_key = self.keyspace.tail();
        let tail_value = serde_json::to_value(end_exclusive)?;

        Ok(SequenceReservation {
            committed: false,
            guard: tail,
            rollback_value: start,
            sequence_range,
            write: (tail_key, tail_value),
        })
    }
}

/// A reserved sequence batch that rolls back when dropped unless committed to the store.
pub struct SequenceReservation {
    committed: bool,
    guard: OwnedMutexGuard<SequenceId>,
    rollback_value: SequenceId,
    sequence_range: std::ops::Range<u64>,
    write: (StoreKey, serde_json::Value),
}

impl SequenceReservation {
    pub fn sequence_numbers(&self) -> impl Iterator<Item = SequenceId> + '_ {
        self.sequence_range.clone().map(SequenceId::new)
    }
}

impl WriteSetReservation for SequenceReservation {
    fn add_writes(&self, entries: &mut Vec<(StoreKey, serde_json::Value)>) {
        entries.push(self.write.clone());
    }

    fn commit(&mut self) {
        self.committed = true;
    }
}

impl Drop for SequenceReservation {
    fn drop(&mut self) {
        if !self.committed {
            *self.guard = self.rollback_value;
        }
    }
}
