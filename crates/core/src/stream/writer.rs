use anyhow::Result;

use crate::AppDeps;
use crate::context::RunContext;
use crate::models::{StreamItem, StreamRecord};
use crate::store::{KeySpace, RunKeySpace, WriteSet};

use super::sequencer::StreamSequencer;

/// A writer for sequentially appending items to the stream.
/// NB: Only one writer should be used per stream - you may clone() as needed.
#[derive(Clone)]
pub struct StreamWriter {
    keyspace: RunKeySpace,
    sequencer: StreamSequencer,
}

impl StreamWriter {
    pub async fn init<D: AppDeps>(context: &RunContext<D>) -> Result<Self> {
        let keyspace = KeySpace::run(&context.run_id);

        let sequencer = StreamSequencer::load((*context).clone()).await?;

        Ok(Self {
            keyspace,
            sequencer,
        })
    }

    /// Reserves sequence IDs and builds the writes for a batch of stream items.
    /// Dropping the returned write set before committing rolls back the reservation.
    pub async fn append(&self, items: Vec<StreamItem>) -> Result<WriteSet> {
        if items.is_empty() {
            return Ok(WriteSet::new());
        }

        let reservation = self.sequencer.reserve(items.len()).await?;
        let write_set = reservation
            .sequence_numbers()
            .zip(items)
            .map(|(id, item)| {
                let record = StreamRecord { id, item };
                let key = self.keyspace.stream_item(&record.id);
                Ok((key, serde_json::to_value(record)?))
            })
            .collect::<Result<WriteSet>>()?;

        Ok(write_set.with_reservation(reservation))
    }
}
