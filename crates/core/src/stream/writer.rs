use anyhow::Result;

use crate::context::RunContext;
use crate::models::{StreamItem, StreamRecord};
use crate::store::keyspace::{KeySpace, RunKeySpace};
use crate::store::{StorageProvider, Store, StoreWriteSet};

use super::sequencer::{SequenceReservation, StreamSequencer};

/// A writer for sequentially appending items to the stream.
/// NB: Only one writer should be used per stream - you may clone() as needed.
pub struct StreamWriter<S: StorageProvider> {
    store: Store<S>,
    keyspace: RunKeySpace,
    sequencer: StreamSequencer,
}

impl<S: StorageProvider> Clone for StreamWriter<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            keyspace: self.keyspace.clone(),
            sequencer: self.sequencer.clone(),
        }
    }
}

/// An uncommitted stream append. Dropping it rolls back the sequence reservation.
pub struct StreamAppendReservation<'a, S: StorageProvider> {
    store: Store<S>,
    sequence: Option<SequenceReservation<'a>>,
    additional_writes: StoreWriteSet,
    records: Vec<StreamRecord>,
}

impl<S: StorageProvider> StreamWriter<S> {
    pub async fn init(context: &RunContext<S>) -> Result<Self> {
        let keyspace = KeySpace::workflow(context.workflow_id.clone())
            .version(context.workflow_version_id.clone())
            .run(context.run_id.clone());

        let sequencer = StreamSequencer::load(context.clone()).await?;

        Ok(Self {
            store: context.store.clone(),
            keyspace,
            sequencer,
        })
    }

    /// Reserves sequence IDs and builds the writes for a batch of stream items.
    /// The returned reservation must be committed to persist the records.
    pub async fn append(
        &self,
        items: Vec<StreamItem>,
    ) -> Result<StreamAppendReservation<'_, S>, anyhow::Error> {
        if items.is_empty() {
            return Ok(StreamAppendReservation {
                store: self.store.clone(),
                sequence: None,
                additional_writes: StoreWriteSet::new(),
                records: Vec::new(),
            });
        }

        let mut sequence = self.sequencer.reserve(items.len()).await?;
        let records = items
            .into_iter()
            .zip(sequence.values.iter().copied())
            .map(|(item, id)| StreamRecord { id, item })
            .collect::<Vec<_>>();

        for record in &records {
            let key = self.keyspace.stream_item(&record.id);
            sequence.writes().add_json(&key, record)?;
        }

        Ok(StreamAppendReservation {
            store: self.store.clone(),
            sequence: Some(sequence),
            additional_writes: StoreWriteSet::new(),
            records,
        })
    }
}

impl<S: StorageProvider> StreamAppendReservation<'_, S> {
    pub fn records(&self) -> &[StreamRecord] {
        &self.records
    }

    pub(crate) fn extend(&mut self, write_set: StoreWriteSet) {
        self.additional_writes.extend(write_set);
    }

    pub async fn commit(self) -> Result<Vec<StreamRecord>, anyhow::Error> {
        let Self {
            store,
            sequence,
            additional_writes,
            records,
        } = self;

        if let Some(mut sequence) = sequence {
            sequence.writes().extend(additional_writes);
            sequence.commit(store).await?;
        } else {
            store.commit_write_set(additional_writes).await?;
        }

        Ok(records)
    }
}
