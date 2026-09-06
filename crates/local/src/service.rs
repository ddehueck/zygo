use std::io;
use std::path::PathBuf;

use anyhow::Result;
use zygo_core::models::{DataReference, WorkflowRunId, WorkflowSchema};
use zygo_core::{Dependencies, Zygo};

use crate::ZygoLocalConfig;
use crate::db::{
    CdcRepository, Db, JobRunRepository, KvRepository, LogsRepository, TagsRepository,
    WorkflowRunRepository, WorkflowRunRow,
};
use crate::paths;
use crate::repos::Repos;
use crate::stream_processor::LocalStreamProcessor;

pub struct ZygoLocalService {
    pub base: Zygo<Dependencies<KvRepository, LogsRepository>>,
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
        let database = Db::open(&path, config.database_busy_timeout, true).await?;

        let cdc = CdcRepository::new(database.clone());
        let tags = TagsRepository::new(database.clone());
        let workflow_runs = WorkflowRunRepository::new(database.clone());
        let job_runs = JobRunRepository::new(database.clone());
        let logs = LogsRepository::new(database.clone());
        let kv = KvRepository::new(database);

        let dependencies = Dependencies::new(kv.clone(), logs.clone());

        Ok(Self {
            base: Zygo::new(dependencies, config.base),
            repos: Repos {
                cdc,
                kv,
                tags,
                workflow_runs,
                job_runs,
                logs,
            },
        })
    }

    pub async fn run(
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
            .insert(&workflow_run_id.to_string(), &workflow_id, &content_hash)
            .await?;

        self.base.run(&workflow_run_id, inputs, schema).await?;

        Ok(workflow_run_id)
    }

    pub async fn cancel(&self, run_id: &WorkflowRunId) -> Result<()> {
        self.base.cancel(run_id).await
    }

    pub fn stream_processor(&self, run_id: &WorkflowRunId) -> LocalStreamProcessor {
        LocalStreamProcessor::new(self.repos.clone(), run_id.clone(), self.base.stream(run_id))
    }

    // todo: don't love that this is here. There's a repository/deps refactor brewing.
    // this requires a cli tui rewrite which is also brewing
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
