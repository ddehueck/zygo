use crate::engine::RunCursor;
use crate::models::{SequenceId, StreamRecord};
use crate::store::StorageProvider;
use crate::stream::Stream;

use super::types::StreamCursor;

pub struct StreamReader<S: StorageProvider> {
    stream: Stream<S>,
    cursor: StreamCursor,
}

pub struct StreamIterator<S: StorageProvider> {
    stream: Stream<S>,
    cursor: StreamCursor,
}

impl<S: StorageProvider> StreamReader<S> {
    pub fn new(stream: Stream<S>) -> Self {
        Self {
            stream,
            cursor: StreamCursor::new(SequenceId::new(0)),
        }
    }

    pub fn into_iter(self) -> StreamIterator<S> {
        StreamIterator {
            stream: self.stream,
            cursor: self.cursor,
        }
    }

    /// Reads the entire stream and returns all items in sequence order.
    pub async fn collect(self) -> Result<Vec<StreamRecord>, anyhow::Error> {
        let mut iter = self.into_iter();
        let mut records = Vec::new();
        while let Some(record) = iter.next().await? {
            records.push(record);
        }
        Ok(records)
    }
}

impl<S: StorageProvider> StreamIterator<S> {
    pub async fn next(&mut self) -> Result<Option<StreamRecord>, anyhow::Error> {
        let cursor = RunCursor::new(self.cursor.next_id);
        let Some((_key, record)) = self.stream.next(&cursor).await? else {
            return Ok(None);
        };
        self.cursor = StreamCursor::new(self.cursor.next_id.next());
        Ok(Some(record))
    }
}
