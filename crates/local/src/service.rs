use std::{io, path::PathBuf};

use anyhow::Result;
use serde_json::Value;

use zygo_core::{
    Zygo,
    models::{DataReference, WorkflowRunId, WorkflowSchema},
    store::StorageProvider,
};

use crate::{
    ZygoLocalConfig,
    db::{Db, KvRepository, WorkflowRun, WorkflowRunRepository},
    paths,
    repos::Repos,
    stream_processor::LocalStreamProcessor,
};

impl StorageProvider for KvRepository {
    async fn put(&self, entries: &[(&str, &Value)]) -> Result<()> {
        self.upsert_many(entries).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Value>> {
        Ok(self.get_by_key(key).await?.map(|entry| entry.value))
    }

    async fn get_many(&self, keys: &[&str]) -> Result<Vec<Option<Value>>> {
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            values.push(self.get(key).await?);
        }

        Ok(values)
    }
}

pub struct ZygoLocalService {
    pub base: Zygo<KvRepository>,
    pub repos: Repos,
}

impl ZygoLocalService {
    pub fn database_path() -> io::Result<PathBuf> {
        paths::database_path()
    }

    pub fn delete_database() -> io::Result<bool> {
        paths::delete_database()
    }

    pub async fn new(config: ZygoLocalConfig) -> Result<Self> {
        let path = Self::database_path()?.to_string_lossy().into_owned();
        let database = Db::open(&path, config.database_busy_timeout).await?;
        let workflow_runs = WorkflowRunRepository::new(database.clone());
        let kv = KvRepository::new(database);
        let store = zygo_core::store::Store::new(kv.clone());

        Ok(Self {
            base: Zygo::new(store, config.base),
            repos: Repos { kv, workflow_runs },
        })
    }

    pub async fn run(
        &self,
        input: DataReference,
        schema: WorkflowSchema,
        tags: &[(&str, &str)],
    ) -> Result<WorkflowRunId> {
        let content_hash = schema.content_hash.as_ref().to_owned();
        let run_id = self.base.run(input, schema).await?;
        let id = run_id.to_string();

        // todo: the workflow tag should be derived from the schema - otherwise all tags will be
        // recoverable from the event stream/store except this one?
        self.repos
            .workflow_runs
            .insert(&id, &content_hash, tags)
            .await?;

        Ok(run_id)
    }

    pub fn stream_processor(&self, run_id: &WorkflowRunId) -> LocalStreamProcessor {
        LocalStreamProcessor::new(
            self.base.stream(run_id),
            self.repos.workflow_runs.clone(),
            run_id.clone(),
        )
    }

    // todo: don't love that this is here. There's a repository/deps refactor brewing.
    pub async fn list_workflow_runs(
        &self,
        filter: Option<(&str, &str)>,
    ) -> Result<Vec<WorkflowRun>> {
        match filter {
            Some((key, value)) => Ok(self.repos.workflow_runs.list_by_tag(key, value).await?),
            None => Ok(self.repos.workflow_runs.list_all().await?),
        }
    }
}
