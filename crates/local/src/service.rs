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
    db::{
        Db, JobRunSummaryRepository, KvRepository, WorkflowRunRepository, WorkflowRunRow,
        WorkflowRunSummaryRepository,
    },
    paths,
    repos::Repos,
    stream_processor::LocalStreamProcessor,
};

const WORKFLOW_ID_TAG_NAME: &str = "sys.workflow";

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
        let workflow_run_summaries = WorkflowRunSummaryRepository::new(database.clone());
        let job_run_summaries = JobRunSummaryRepository::new(database.clone());
        let kv = KvRepository::new(database);
        let store = zygo_core::store::Store::new(kv.clone());

        Ok(Self {
            base: Zygo::new(store, config.base),
            repos: Repos {
                kv,
                workflow_runs,
                workflow_run_summaries,
                job_run_summaries,
            },
        })
    }

    pub async fn run(&self, input: DataReference, schema: WorkflowSchema) -> Result<WorkflowRunId> {
        self.run_many(vec![input], schema).await
    }

    pub async fn run_many(
        &self,
        inputs: Vec<DataReference>,
        schema: WorkflowSchema,
    ) -> Result<WorkflowRunId> {
        anyhow::ensure!(
            !inputs.is_empty(),
            "a workflow run requires at least one input"
        );

        let workflow_id = schema.id.to_string();
        let content_hash = schema.content_hash.to_string();

        // Each invocation is a distinct execution attempt. Job result reuse is
        // handled separately by deterministic job run IDs in the result cache.
        let workflow_run_id = WorkflowRunId::new();

        // saves a record of the run before actually running it
        // we save the workflow id as a tag so we can filter runs by workflow
        self.repos
            .workflow_runs
            .insert(
                &workflow_run_id.to_string(),
                &workflow_id,
                &content_hash,
                &[(WORKFLOW_ID_TAG_NAME, workflow_id.as_str())],
            )
            .await?;

        self.base.run_many(&workflow_run_id, inputs, schema).await?;

        Ok(workflow_run_id)
    }

    pub async fn cancel(&self, run_id: &WorkflowRunId) -> Result<()> {
        self.base.cancel(run_id).await
    }

    pub fn stream_processor(&self, run_id: &WorkflowRunId) -> LocalStreamProcessor {
        LocalStreamProcessor::new(self.repos.clone(), run_id.clone(), self.base.stream(run_id))
    }

    // todo: don't love that this is here. There's a repository/deps refactor brewing.
    pub async fn list_workflow_runs(
        &self,
        filter: Option<(&str, &str)>,
    ) -> Result<Vec<WorkflowRunRow>> {
        match filter {
            Some((key, value)) => Ok(self.repos.workflow_runs.list_by_tag(key, value).await?),
            None => Ok(self.repos.workflow_runs.list_all().await?),
        }
    }
}
