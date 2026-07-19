use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use crate::context::RunContext;
use crate::models::SequenceId;
use crate::store::keyspace::{KeySpace, RunKeySpace};
use crate::store::{StorageProvider, Store, StoreWriteSet};

#[derive(Clone)]
pub struct StreamSequencer {
    keyspace: RunKeySpace,
    tail: Arc<Mutex<SequenceId>>,
}

impl StreamSequencer {
    pub async fn load<S: StorageProvider>(context: RunContext<S>) -> Result<Self> {
        let run_keyspace = KeySpace::workflow(context.workflow_id)
            .version(context.workflow_version_id)
            .run(context.run_id);

        let tail_key = run_keyspace.stream_append_cursor();
        let tail_bytes = context.store.get(&tail_key).await?;

        let tail = match tail_bytes {
            Some(v) => serde_json::from_slice::<SequenceId>(&v)?,
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

        let mut tail = self.tail.lock().await;

        let start = *tail;
        let start_copy = start.clone();

        let end_exclusive = start
            .get()
            .checked_add(n as u64)
            .map(SequenceId::new)
            .ok_or_else(|| anyhow::anyhow!("sequence overflow"))?;

        let batch = (start.get()..end_exclusive.get())
            .map(SequenceId::new)
            .collect();

        *tail = end_exclusive;

        let tail_key = self.keyspace.stream_append_cursor();
        let tail_bytes = serde_json::to_vec(&end_exclusive)?;

        let mut write_set = StoreWriteSet::new();
        write_set.push(&tail_key, tail_bytes);

        Ok(SequenceReservation {
            committed: false,
            guard: tail,
            rollback_value: start_copy,
            values: batch,
            writes: Some(write_set),
        })
    }
}

/// A reserved sequence batch that rolls back when dropped unless committed to the store.
pub struct SequenceReservation<'a> {
    committed: bool,
    guard: MutexGuard<'a, SequenceId>,
    rollback_value: SequenceId,
    pub values: Vec<SequenceId>,
    pub writes: Option<StoreWriteSet>,
}

impl<'a> SequenceReservation<'a> {
    pub async fn commit<S: StorageProvider>(mut self, store: Store<S>) -> Result<()> {
        let writes = self
            .writes
            .take()
            .expect("unexpected no write set at commit time");
        store.commit_write_set(writes).await?;
        self.committed = true;
        Ok(())
    }

    pub fn writes(&mut self) -> &mut StoreWriteSet {
        self.writes
            .as_mut()
            .expect("write set has already been consumed")
    }
}

impl<'a> Drop for SequenceReservation<'a> {
    fn drop(&mut self) {
        if !self.committed {
            *self.guard = self.rollback_value.clone();
        }
    }
}
