use crate::context::RunContext;
use crate::engine::RunCursor;
use crate::models::{SequenceId, StreamRecord};
use crate::store::keyspace::RunKeySpace;
use crate::store::{StorageProvider, Store};

pub struct StreamReader<S: StorageProvider> {
    store: Store<S>,
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
    pub fn new(context: RunContext<S>) -> Self {
        Self {
            store: context.store.clone(),
            keyspace: context.into(),
        }
    }

    pub async fn next(&self, cursor: RunCursor) -> Result<ReadResult, anyhow::Error> {
        let next_id = cursor.next_id;
        let key = self.keyspace.stream_item(&next_id);
        let record = self.store.get_json(&key).await?;
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

    pub fn into_iter(self) -> StreamIterator<S> {
        StreamIterator {
            reader: self,
            cursor: RunCursor::new(SequenceId::new(0)),
        }
    }

    /// Reads the entire stream and returns all items in sequence order.
    /// NB: Use with caution.
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
        let result = self.reader.next(self.cursor.clone()).await?;
        self.cursor = result.next_cursor;
        Ok(result.record)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::context::{RunContext, ServiceContext};
    use crate::models::{
        DataReference, DataReferenceInsertedData, Event, EventId, EventKind, RunId, Source,
        StreamItem, WorkflowId, WorkflowVersionId,
    };
    use crate::store::{MemoryStore, Store};
    use crate::workers::WorkerPool;

    use super::*;

    #[tokio::test]
    async fn advances_cursor_only_when_a_record_is_read() {
        let service_context =
            ServiceContext::new(Store::new(MemoryStore::new()), WorkerPool::new(1));
        let context = RunContext::new(
            &service_context,
            WorkflowId::try_from("workflow".to_owned()).unwrap(),
            WorkflowVersionId::try_from("version".to_owned()).unwrap(),
            RunId::try_from("run".to_owned()).unwrap(),
        );
        let sequence_id = SequenceId::new(4);
        let record = StreamRecord {
            id: sequence_id,
            item: StreamItem::Event(Event {
                id: EventId::try_from("event".to_owned()).unwrap(),
                is_replay: false,
                timestamp: SystemTime::now(),
                kind: EventKind::DataReferenceInserted(DataReferenceInsertedData {
                    data_reference: DataReference::new(
                        "memory://value".to_owned(),
                        "etag".to_owned(),
                    ),
                }),
                source: Source::Input,
                workflow_id: context.workflow_id.clone(),
                workflow_run_id: context.run_id.clone(),
                workflow_version_id: context.workflow_version_id.clone(),
            }),
        };
        let key = RunKeySpace::from(context.clone()).stream_item(&sequence_id);
        context.store.put_json(key.as_str(), &record).await.unwrap();

        let reader = StreamReader::new(context);
        let read = reader.next(RunCursor::new(sequence_id)).await.unwrap();

        assert_eq!(read.record.unwrap().id, sequence_id);
        assert_eq!(read.next_cursor.next_id, sequence_id.next());

        let empty = reader.next(read.next_cursor).await.unwrap();
        assert!(empty.record.is_none());
        assert_eq!(empty.next_cursor.next_id, sequence_id.next());
    }
}
