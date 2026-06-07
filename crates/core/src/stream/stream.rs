use crate::models::{SequenceId, StreamAppendCursor, StreamItem, StreamRecord};
use crate::store::keyspace::{KeySpace, RunKeySpace, StoreKey};
use crate::store::{StorageProvider, Store, StoreWriteSet};

use crate::engine::{RunCursor, RunScope};

pub struct Stream<S: StorageProvider> {
    store: Store<S>,
    keyspace: RunKeySpace,
}

impl<S: StorageProvider> Stream<S> {
    pub fn new(store: Store<S>, scope: RunScope) -> Self {
        let run_keyspace = KeySpace::workflow(scope.workflow_id.clone())
            .version(scope.workflow_version_id.clone())
            .run(scope.run_id.clone());

        Self {
            store,
            keyspace: run_keyspace,
        }
    }

    pub async fn next(
        &self,
        cursor: &RunCursor,
    ) -> Result<Option<(StoreKey, StreamRecord)>, anyhow::Error> {
        let key = self.keyspace.stream_item(&cursor.next_id.to_string());

        let Some(record) = self.store.get_json(&key).await? else {
            return Ok(None);
        };

        Ok(Some((key, record)))
    }

    async fn read_append_cursor(&self) -> Result<StreamAppendCursor, anyhow::Error> {
        let key = self.stream_append_cursor_key();
        Ok(self
            .store
            .get_json::<StreamAppendCursor>(&key)
            .await?
            .unwrap_or(StreamAppendCursor {
                next_append_sequence_id: SequenceId::new(0),
            }))
    }

    pub async fn append(&self, items: Vec<StreamItem>) -> Result<StoreWriteSet, anyhow::Error> {
        if items.is_empty() {
            return Ok(StoreWriteSet::with_capacity(0));
        }

        // Load the tail cursor to get the next sequence id to append to.
        // TODO: Can we just always keep this in memory?
        let cursor = self.read_append_cursor().await?;
        let start_sequence_id = cursor.next_append_sequence_id;
        let mut next_sequence_id = start_sequence_id;

        // Existing slot check is necessary to prevent append skew.
        let start_key = self.stream_item_key(start_sequence_id);
        let existing_slot = self.store.get_json::<StreamRecord>(&start_key).await?;
        assert!(
            existing_slot.is_none(),
            "append skew at {}",
            start_key.as_str()
        );

        // Build a write set that will be flushed to storage in one atomic operation.
        let mut write_set = StoreWriteSet::with_capacity(items.len() + 1);
        for item in items {
            let key = self.stream_item_key(next_sequence_id);
            let record = StreamRecord {
                id: next_sequence_id,
                item,
            };
            write_set.add_json(&key, &record)?;
            next_sequence_id = next_sequence_id.next();
        }

        // We also want to update/persist the cursor at the next sequence id.
        write_set.add_json(
            &self.stream_append_cursor_key(),
            &StreamAppendCursor {
                next_append_sequence_id: next_sequence_id,
            },
        )?;

        Ok(write_set)
    }

    fn stream_append_cursor_key(&self) -> StoreKey {
        self.keyspace.stream_append_cursor()
    }

    fn stream_item_key(&self, sequence_id: SequenceId) -> StoreKey {
        self.keyspace.stream_item(&sequence_id.to_string())
    }
}
