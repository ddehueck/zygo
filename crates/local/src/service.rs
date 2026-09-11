use std::io;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use zygo_core::actor::ActorStateRx;
use zygo_core::models::{DataReference, WorkflowRunId, WorkflowSchema};
use zygo_core::{Dependencies, Zygo, ZygoRun};

use crate::ZygoLocalConfig;
use crate::db::{
    CdcRepository, DataReferenceRepository, Db, JobRunRepository, KvRepository, LogsRepository,
    Repos, TagsRepository, WorkflowRepository, WorkflowRunRepository,
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
        inputs: Vec<DataReference>,
        workflow_id: i64,
        schema: WorkflowSchema,
    ) -> Result<ZygoLocalRun> {
        ZygoLocalRun::start(
            inputs,
            workflow_id,
            schema,
            self.zygo.clone(),
            self.repos.clone(),
        )
        .await
    }
}

pub struct ZygoLocalWorkflow {
    pub id: i64,
    pub schema: WorkflowSchema,
    service: ZygoLocalService,
}

impl ZygoLocalWorkflow {
    pub async fn run(&self, inputs: Vec<DataReference>) -> Result<ZygoLocalRun> {
        self.service.run(inputs, self.id, self.schema.clone()).await
    }
}

pub struct ZygoLocalRun {
    pub id: WorkflowRunId,
    pub workflow_id: i64,
    run: ZygoRun<Dependencies<KvRepository, LogsRepository>>,
    repos: Repos,
}

impl ZygoLocalRun {
    pub async fn start(
        inputs: Vec<DataReference>,
        workflow_id: i64,
        schema: WorkflowSchema,
        zygo: Zygo<Dependencies<KvRepository, LogsRepository>>,
        repos: Repos,
    ) -> Result<Self> {
        let content_hash = schema.content_hash.to_string();

        // Each invocation is a distinct execution attempt. Job result reuse is
        // handled separately by deterministic job run IDs in the result cache.
        let workflow_run_id = WorkflowRunId::new();

        // saves a record of the run before actually running it
        repos
            .workflow_runs
            .insert(&workflow_run_id.to_string(), workflow_id, &content_hash)
            .await?;

        let run = zygo.run(&workflow_run_id, inputs, schema).await?;

        Ok(Self {
            id: workflow_run_id,
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
