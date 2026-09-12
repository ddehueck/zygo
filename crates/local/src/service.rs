use std::io;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use zygo_core::actor::ActorStateRx;
use zygo_core::models::{ChannelItemInsertedData, JobId, WorkflowRunId, WorkflowSchema};
use zygo_core::{Dependencies, Zygo, ZygoRun};

use crate::ZygoLocalConfig;
use crate::db::{
    CdcRepository, DataReferenceRepository, Db, JobRunRepository, KvRepository, LogsRepository,
    Repos, TagsRepository, WorkflowRepository, WorkflowRunModel, WorkflowRunRepository,
};
use crate::paths;
use crate::stream_processor::LocalStreamProcessor;

#[derive(Clone)]
pub struct ZygoLocalService {
    pub zygo: Zygo<Dependencies<KvRepository, LogsRepository>>,
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
        let no_cdc_database = Db::open(&path, config.database_busy_timeout, false).await?;

        // Everything that changes the core models should use a connection where CDC events are generated.
        let tags = TagsRepository::new(database.clone());
        let data_references = DataReferenceRepository::new(database.clone());
        let workflow_runs = WorkflowRunRepository::new(database.clone());
        let workflows = WorkflowRepository::new(database.clone());
        let job_runs = JobRunRepository::new(database.clone());

        // These repos should not result in any CDC events being generated,
        // so we open a separate database connection for them.
        let logs = LogsRepository::new(no_cdc_database.clone());
        let kv = KvRepository::new(no_cdc_database.clone());
        let cdc = CdcRepository::new(no_cdc_database);

        let dependencies = Dependencies::new(kv.clone(), logs.clone());

        Ok(Self {
            zygo: Zygo::new(dependencies, config.base),
            repos: Repos {
                cdc,
                kv,
                tags,
                data_references,
                workflow_runs,
                workflows,
                job_runs,
                logs,
            },
        })
    }

    pub async fn register(&self, schema: WorkflowSchema) -> Result<ZygoLocalWorkflow> {
        let serialized_schema = serde_json::to_string(&schema)?;
        let name = schema.id.to_string();
        let path = schema.entrypoint.cwd().to_owned();
        let workflow = self
            .repos
            .workflows
            .upsert(&name, &path, &serialized_schema)
            .await?;

        Ok(ZygoLocalWorkflow {
            id: workflow.id,
            schema,
            service: self.clone(),
        })
    }

    pub async fn load(&self, id: i64) -> Result<ZygoLocalWorkflow> {
        let workflow = self
            .repos
            .workflows
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("workflow with id {id} was not found"))?;
        let schema = serde_json::from_str(&workflow.schema)?;

        Ok(ZygoLocalWorkflow {
            id: workflow.id,
            schema,
            service: self.clone(),
        })
    }

    pub async fn run(
        &self,
        inputs: Vec<ChannelItemInsertedData>,
        workflow_id: i64,
        schema: WorkflowSchema,
        disabled_jobs: Option<Vec<JobId>>,
    ) -> Result<ZygoLocalRun> {
        ZygoLocalRun::start(
            inputs,
            workflow_id,
            schema,
            self.zygo.clone(),
            self.repos.clone(),
            disabled_jobs,
        )
        .await
    }

    // todo: don't love that this is here. There's a repository/deps refactor brewing.
    // this requires a cli tui rewrite which is also brewing
    pub async fn list_workflow_runs(
        &self,
        filter: Option<(&str, &str)>,
    ) -> Result<Vec<WorkflowRunModel>> {
        match filter {
            Some((key, value)) => Ok(self.repos.workflow_runs.list_by_tag(key, value).await?),
            None => Ok(self.repos.workflow_runs.list_all().await?),
        }
    }
}

pub struct ZygoLocalWorkflow {
    pub id: i64,
    pub schema: WorkflowSchema,
    service: ZygoLocalService,
}

impl ZygoLocalWorkflow {
    pub async fn run(
        &self,
        inputs: Vec<ChannelItemInsertedData>,
        disabled_jobs: Option<Vec<JobId>>,
    ) -> Result<ZygoLocalRun> {
        self.service
            .run(inputs, self.id, self.schema.clone(), disabled_jobs)
            .await
    }
}

pub struct ZygoLocalRun {
    pub id: WorkflowRunId,
    pub db_id: i64,
    pub workflow_id: i64,
    run: ZygoRun<Dependencies<KvRepository, LogsRepository>>,
    repos: Repos,
}

impl ZygoLocalRun {
    pub async fn start(
        inputs: Vec<ChannelItemInsertedData>,
        workflow_id: i64,
        schema: WorkflowSchema,
        zygo: Zygo<Dependencies<KvRepository, LogsRepository>>,
        repos: Repos,
        disabled_jobs: Option<Vec<JobId>>,
    ) -> Result<Self> {
        let content_hash = schema.content_hash.to_string();

        // Each invocation is a distinct execution attempt. Job result reuse is
        // handled separately by deterministic job run IDs in the result cache.
        let workflow_run_id = WorkflowRunId::new();

        // saves a record of the run before actually running it
        let db_run = repos
            .workflow_runs
            .insert(&workflow_run_id.to_string(), workflow_id, &content_hash)
            .await?;

        let run = zygo
            .run(&workflow_run_id, inputs, schema, disabled_jobs)
            .await?;

        Ok(Self {
            id: workflow_run_id,
            db_id: db_run.id,
            workflow_id,
            run,
            repos,
        })
    }

    pub async fn cancel(&self) -> Result<()> {
        // todo: add db cleanup side effects
        self.run.cancel(&self.id).await
    }

    pub fn stream_processor(&self) -> LocalStreamProcessor {
        LocalStreamProcessor::new(
            self.repos.clone(),
            self.id.clone(),
            self.run.stream(&self.id),
        )
    }

    pub fn subscribe(&self) -> Result<ActorStateRx> {
        self.run.subscribe()
    }
}
