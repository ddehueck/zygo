pub(crate) mod keyspace;
mod provider_interface;

pub use provider_interface::StorageProvider;

use std::sync::Arc;

use keyspace::KeySpace;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::models::ids::{RunId, WorkflowId, WorkflowVersionId};
use crate::models::{
    Event, JobRunId, ResultCacheItem, Run, SequenceId, StreamItem, StreamRecord, Workflow,
    WorkflowVersion,
};
use crate::store::keyspace::StoreKey;

pub struct Store<S: StorageProvider> {
    storage: Arc<S>,
}

// Manual `Clone` (rather than `#[derive]`) so cloning only bumps the `Arc`
// refcount and never requires `S: Clone`. The clone shares the same underlying
// storage, which lets the supervisor hand a cheap `Store` handle to each
// per-workflow actor.
impl<S: StorageProvider> Clone for Store<S> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}

pub(crate) struct StoreWriteSet {
    entries: Vec<(String, Vec<u8>)>,
}

impl StoreWriteSet {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn add_json<T: Serialize>(
        &mut self,
        key: &StoreKey,
        value: &T,
    ) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(value)?;
        self.add_bytes(key.as_string(), bytes);
        Ok(())
    }

    fn add_bytes(&mut self, key: String, bytes: Vec<u8>) {
        self.entries.push((key, bytes));
    }

    pub(crate) fn extend(&mut self, other: StoreWriteSet) {
        self.entries.extend(other.entries);
    }

    pub(crate) fn into_entries(self) -> Vec<(String, Vec<u8>)> {
        self.entries
    }
}

impl<S: StorageProvider> Store<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    pub fn workflows(&self) -> Workflows<'_, S> {
        Workflows { store: self }
    }

    pub fn versions(&self) -> Versions<'_, S> {
        Versions { store: self }
    }

    pub fn runs(&self) -> Runs<'_, S> {
        Runs { store: self }
    }

    pub fn stream_items(&self) -> StreamItems<'_, S> {
        StreamItems { store: self }
    }

    pub fn results_cache(&self) -> ResultsCache<'_, S> {
        ResultsCache { store: self }
    }

    // ---------- generic JSON-over-KV helpers ----------

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        key: &StoreKey,
    ) -> anyhow::Result<Option<T>> {
        match self.storage.get(key.as_str()).await? {
            Some(v) => Ok(Some(serde_json::from_slice(&v)?)),
            None => Ok(None),
        }
    }

    pub(crate) async fn get_many_json<T: DeserializeOwned>(
        &self,
        keys: &[&str],
    ) -> anyhow::Result<Vec<Option<T>>> {
        let values = self.storage.get_many(keys).await?;
        values
            .into_iter()
            .map(|v| match v {
                Some(v) => Ok(Some(serde_json::from_slice(&v)?)),
                None => Ok(None),
            })
            .collect()
    }

    pub(crate) async fn put_json<T: Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(value)?;
        self.storage.put(key, &bytes).await
    }

    /// Pre-serialized JSON payloads; used when a single atomic batch mixes types.
    pub(crate) async fn put_many_bytes(&self, entries: &[(String, Vec<u8>)]) -> anyhow::Result<()> {
        let raw_entries: Vec<(&str, &[u8])> = entries
            .iter()
            .map(|(key, bytes)| (key.as_str(), bytes.as_slice()))
            .collect();
        self.storage.put_many(&raw_entries).await
    }

    pub(crate) async fn commit_write_set(&self, write_set: StoreWriteSet) -> anyhow::Result<()> {
        let entries = write_set.into_entries();
        self.put_many_bytes(&entries).await
    }

    pub(crate) async fn delete_key(&self, key: &str) -> anyhow::Result<()> {
        self.storage.delete(key).await
    }

    pub(crate) async fn delete_prefix(&self, prefix: &str) -> anyhow::Result<()> {
        self.storage.delete_range(prefix).await
    }

    pub(crate) async fn list_json<T: DeserializeOwned>(
        &self,
        prefix: &str,
        after: Option<&str>,
        limit: u32,
    ) -> anyhow::Result<Vec<T>> {
        self.storage
            .list_range(prefix, after, limit as usize)
            .await?
            .into_iter()
            .map(|(_, v)| serde_json::from_slice(&v).map_err(anyhow::Error::from))
            .collect()
    }
}

// ---------- workflows ----------

pub struct Workflows<'s, S: StorageProvider> {
    store: &'s Store<S>,
}

impl<S: StorageProvider> Workflows<'_, S> {
    pub async fn get(&self, id: &WorkflowId) -> anyhow::Result<Option<Workflow>> {
        self.store
            .get_json(&KeySpace::workflow(id.clone()).meta())
            .await
    }

    pub async fn put(&self, workflow: &Workflow) -> anyhow::Result<()> {
        self.store
            .put_json(
                &KeySpace::workflow(workflow.id.clone()).meta().as_string(),
                workflow,
            )
            .await
    }

    pub async fn delete(&self, id: &WorkflowId) -> anyhow::Result<()> {
        self.store
            .delete_key(&KeySpace::workflow(id.clone()).meta().as_string())
            .await
    }

    pub async fn list(
        &self,
        after: Option<&WorkflowId>,
        limit: u32,
    ) -> anyhow::Result<Vec<Workflow>> {
        let after = after.map(|id| KeySpace::workflow(id.clone()).meta().as_string());
        self.store
            .list_json(&KeySpace::workflows_prefix(), after.as_deref(), limit)
            .await
    }
}

// ---------- versions ----------

pub struct Versions<'s, S: StorageProvider> {
    store: &'s Store<S>,
}

impl<S: StorageProvider> Versions<'_, S> {
    pub async fn get(
        &self,
        wf: &WorkflowId,
        ver: &WorkflowVersionId,
    ) -> anyhow::Result<Option<WorkflowVersion>> {
        self.store
            .get_json(&KeySpace::workflow(wf.clone()).version(ver.clone()).meta())
            .await
    }

    pub async fn put(&self, version: &WorkflowVersion) -> anyhow::Result<()> {
        let key = KeySpace::workflow(version.workflow_id.clone())
            .version(version.id.clone())
            .meta();
        self.store.put_json(&key.as_string(), version).await
    }

    pub async fn delete(&self, wf: &WorkflowId, ver: &WorkflowVersionId) -> anyhow::Result<()> {
        self.store
            .delete_key(
                &KeySpace::workflow(wf.clone())
                    .version(ver.clone())
                    .meta()
                    .as_string(),
            )
            .await
    }

    pub async fn list(
        &self,
        after: Option<(&WorkflowId, &WorkflowVersionId)>,
        limit: u32,
    ) -> anyhow::Result<Vec<WorkflowVersion>> {
        let after = after.map(|(wf, ver)| {
            KeySpace::workflow(wf.clone())
                .version(ver.clone())
                .meta()
                .as_string()
        });
        self.store
            .list_json(&KeySpace::versions_prefix(), after.as_deref(), limit)
            .await
    }
}

// ---------- runs ----------

pub struct Runs<'s, S: StorageProvider> {
    store: &'s Store<S>,
}

impl<S: StorageProvider> Runs<'_, S> {
    pub async fn get(
        &self,
        workflow_id: &WorkflowId,
        version_id: &WorkflowVersionId,
        run_id: &RunId,
    ) -> anyhow::Result<Option<Run>> {
        self.store
            .get_json(
                &KeySpace::workflow(workflow_id.clone())
                    .version(version_id.clone())
                    .run(run_id.clone())
                    .meta(),
            )
            .await
    }

    /// Persist a run under the given workflow. The version is read from
    /// `run.workflow_version_id` so callers cannot accidentally file a run
    /// under a mismatched version key.
    pub async fn put(&self, workflow_id: &WorkflowId, run: &Run) -> anyhow::Result<()> {
        let key = KeySpace::workflow(workflow_id.clone())
            .version(run.workflow_version_id.clone())
            .run(run.id.clone())
            .meta();
        self.store.put_json(&key.as_string(), run).await
    }

    pub async fn list(
        &self,
        workflow_id: &WorkflowId,
        version_id: &WorkflowVersionId,
        after: Option<&RunId>,
        limit: u32,
    ) -> anyhow::Result<Vec<Run>> {
        let after = after.map(|run_id| {
            KeySpace::workflow(workflow_id.clone())
                .version(version_id.clone())
                .run(run_id.clone())
                .meta()
                .as_string()
        });
        let version_scope = KeySpace::workflow(workflow_id.clone()).version(version_id.clone());
        self.store
            .list_json(&version_scope.runs_prefix(), after.as_deref(), limit)
            .await
    }
}

// ---------- stream items ----------

pub struct StreamItems<'s, S: StorageProvider> {
    store: &'s Store<S>,
}

impl<S: StorageProvider> StreamItems<'_, S> {
    pub async fn get(
        &self,
        workflow_id: &WorkflowId,
        version_id: &WorkflowVersionId,
        run_id: &RunId,
        sequence_id: &SequenceId,
    ) -> anyhow::Result<Option<StreamRecord>> {
        self.store
            .get_json(
                &KeySpace::workflow(workflow_id.clone())
                    .version(version_id.clone())
                    .run(run_id.clone())
                    .stream_item(&sequence_id.to_string()),
            )
            .await
    }

    /// Persist a stream item under the given run at a specific sequence id.
    /// Returns the fully-qualified key the item was stored at so callers
    /// can track it (e.g. to flush into the result cache).
    pub async fn put(
        &self,
        workflow_id: &WorkflowId,
        version_id: &WorkflowVersionId,
        run_id: &RunId,
        sequence_id: &SequenceId,
        record: &StreamRecord,
    ) -> anyhow::Result<StoreKey> {
        let key = KeySpace::workflow(workflow_id.clone())
            .version(version_id.clone())
            .run(run_id.clone())
            .stream_item(&sequence_id.to_string());
        self.store.put_json(&key.as_string(), record).await?;
        Ok(key)
    }

    pub async fn delete(
        &self,
        workflow_id: &WorkflowId,
        version_id: &WorkflowVersionId,
        run_id: &RunId,
        sequence_id: &SequenceId,
    ) -> anyhow::Result<()> {
        let key = KeySpace::workflow(workflow_id.clone())
            .version(version_id.clone())
            .run(run_id.clone())
            .stream_item(&sequence_id.to_string());
        self.store.delete_key(&key.as_string()).await
    }

    pub async fn list(
        &self,
        workflow_id: &WorkflowId,
        version_id: &WorkflowVersionId,
        run_id: &RunId,
        after: Option<&SequenceId>,
        limit: u32,
    ) -> anyhow::Result<Vec<StreamRecord>> {
        let run_scope = KeySpace::workflow(workflow_id.clone())
            .version(version_id.clone())
            .run(run_id.clone());

        let after = after.map(|sequence_id| {
            run_scope
                .stream_item(&sequence_id.clone().to_string())
                .as_string()
        });

        self.store
            .list_json(&run_scope.stream_items_prefix(), after.as_deref(), limit)
            .await
    }
}

// ---------- results cache ----------

pub struct ResultsCache<'s, S: StorageProvider> {
    store: &'s Store<S>,
}

impl<S: StorageProvider> ResultsCache<'_, S> {
    pub async fn get(
        &self,
        workflow_id: &WorkflowId,
        job_run_id: &JobRunId,
    ) -> anyhow::Result<Option<ResultCacheItem>> {
        self.store
            .get_json(&KeySpace::workflow(workflow_id.clone()).cache_result(job_run_id.as_ref()))
            .await
    }

    pub async fn get_events(&self, cache_item: &ResultCacheItem) -> anyhow::Result<Vec<Event>> {
        let keys: Vec<&str> = cache_item.event_keys.iter().map(String::as_str).collect();

        let stream_items: Vec<StreamRecord> = self
            .store
            .get_many_json(&keys)
            .await?
            .into_iter()
            .flatten()
            .collect();

        let events: Vec<Event> = stream_items
            .into_iter()
            .filter_map(|record| match record.item {
                StreamItem::Event(event) => Some(event),
                StreamItem::Command(_) => None,
            })
            .collect();

        // If we don't have all the events, return None
        // Maybe this should be an error?
        assert!(
            events.len() == cache_item.event_keys.len(),
            "events.len() != cache_item.event_keys.len()"
        );

        Ok(events)
    }

    pub async fn put(
        &self,
        workflow_id: &WorkflowId,
        job_run_id: &JobRunId,
        result_cache_item: &ResultCacheItem,
    ) -> anyhow::Result<()> {
        let key = KeySpace::workflow(workflow_id.clone()).cache_result(job_run_id.as_ref());
        self.store
            .put_json(&key.as_string(), result_cache_item)
            .await
    }

    pub async fn delete(
        &self,
        workflow_id: &WorkflowId,
        job_run_id: &JobRunId,
    ) -> anyhow::Result<()> {
        let key = KeySpace::workflow(workflow_id.clone()).cache_result(job_run_id.as_ref());
        self.store.delete_key(&key.as_string()).await
    }
}
