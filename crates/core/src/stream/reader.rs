use crate::engine::RunCursor;
use crate::models::{SequenceId, StreamRecord, WorkflowRunId};
use crate::store::{KeySpace, RunKeySpace, StorageProvider};

pub struct StreamReader<S: StorageProvider> {
    store: S,
    keyspace: RunKeySpace,
}

pub struct ReadResult {
    pub record: Option<StreamRecord>,
    pub next_cursor: RunCursor,
}

pub struct StreamIterator<S: StorageProvider> {
    reader: StreamReader<S>,
    cursor: RunCursor,
}

impl<S: StorageProvider> StreamReader<S> {
    pub fn new(store: S, run_id: &WorkflowRunId) -> Self {
        Self {
            store,
            keyspace: KeySpace::run(run_id),
        }
    }

    pub async fn next(&self, cursor: RunCursor) -> Result<ReadResult, anyhow::Error> {
        let next_id = cursor.next_id;
        let key = self.keyspace.stream_item(&next_id);
        let record = self
            .store
            .get(&key)
            .await?
            .map(serde_json::from_value)
            .transpose()?;

        let next_cursor = if record.is_some() {
            RunCursor::new(next_id.next())
        } else {
            cursor
        };

        Ok(ReadResult {
            record,
            next_cursor,
        })
    }

    pub fn into_stream(self) -> StreamIterator<S> {
        StreamIterator {
            reader: self,
            cursor: RunCursor::new(SequenceId::new(0)),
        }
    }

    /// Reads the entire stream and returns all items in sequence order.
    /// NB: Use with caution.
    pub async fn collect(self) -> Result<Vec<StreamRecord>, anyhow::Error> {
        let mut iter = self.into_stream();
        let mut records = Vec::new();
        while let Some(record) = iter.next().await? {
            records.push(record);
        }
        Ok(records)
    }
}

impl<S: StorageProvider> StreamIterator<S> {
    pub async fn next(&mut self) -> Result<Option<StreamRecord>, anyhow::Error> {
        let result = self.reader.next(self.cursor.clone()).await?;
        self.cursor = result.next_cursor;
        Ok(result.record)
    }
}
