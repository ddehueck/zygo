use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::models::{
    ChannelItemInsertedData, DataReferenceUri, Entrypoint, Event, EventId, EventKind, JobId,
    Source, WorkflowRunId, WorkflowSchema,
};
use crate::{ActorStateRx, RunContext, RunHandle};
use crate::{LocalRuntime, RunOptions, ZygoLocalConfig};
use anyhow::{Result, anyhow};

use crate::db::{
    CdcRepository, DataReferenceRepository, Db, JobRunRepository, LogsRepository, Repos,
    TagsRepository, WorkflowRepository, WorkflowRunModel, WorkflowRunRepository,
};
use crate::paths;

#[derive(Clone)]
pub struct ZygoLocalService {
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

        // Application read-model writes generate CDC changes for desktop sync.
        let tags = TagsRepository::new(database.clone());
        let data_references = DataReferenceRepository::new(database.clone());
        let workflow_runs = WorkflowRunRepository::new(database.clone());
        let workflows = WorkflowRepository::new(database.clone());
        let job_runs = JobRunRepository::new(database.clone());

        // These repos should not result in any CDC events being generated,
        // so we open a separate database connection for them.
        let logs = LogsRepository::new(no_cdc_database.clone());
        let cdc = CdcRepository::new(no_cdc_database);

        Ok(Self {
            repos: Repos {
                cdc,
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
        inputs: Vec<DataReferenceUri>,
        workflow_id: i64,
        schema: WorkflowSchema,
        options: RunOptions,
    ) -> Result<ZygoLocalRun> {
        ZygoLocalRun::start(inputs, workflow_id, schema, options, self.repos.clone()).await
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
        inputs: Vec<DataReferenceUri>,
        options: RunOptions,
    ) -> Result<ZygoLocalRun> {
        self.service
            .run(inputs, self.id, self.schema.clone(), options)
            .await
    }

    /// Runs a single job by creating a job-scoped schema via [`WorkflowSchema::to_job_run`].
    pub async fn run_job(
        &self,
        inputs: Vec<DataReferenceUri>,
        job_id: &JobId,
        options: RunOptions,
    ) -> Result<ZygoLocalRun> {
        let schema = self
            .schema
            .to_job_run(job_id)
            .ok_or_else(|| anyhow!("job `{job_id}` was not found in workflow schema"))?;
        self.service.run(inputs, self.id, schema, options).await
    }
}

pub struct ZygoLocalRun {
    pub id: WorkflowRunId,
    pub db_id: i64,
    pub workflow_id: i64,
    actor: RunHandle,
    runtime: LocalRuntime,
}

impl ZygoLocalRun {
    pub async fn start(
        inputs: Vec<DataReferenceUri>,
        workflow_id: i64,
        schema: WorkflowSchema,
        options: RunOptions,
        repos: Repos,
    ) -> Result<Self> {
        anyhow::ensure!(
            options.num_workers > 0,
            "at least one local worker is required"
        );
        let _disable_cache = options.disable_cache;

        let content_hash = schema.content_hash.to_string();
        let serialized_schema = serde_json::to_string(&schema)?;

        anyhow::ensure!(
            !inputs.is_empty(),
            "a workflow run requires at least one input"
        );
        // Each invocation is a distinct execution attempt.
        // todo: add caching at the runtime layer.
        let workflow_run_id = WorkflowRunId::new();

        // saves a record of the run before actually running it
        let db_run = repos
            .workflow_runs
            .insert(
                &workflow_run_id.to_string(),
                workflow_id,
                &content_hash,
                &serialized_schema,
            )
            .await?;

        let (events_tx, events_rx) = tokio::sync::mpsc::unbounded_channel();
        let Entrypoint::Python(python_cli) = schema.entrypoint.clone();

        let runtime = LocalRuntime::new(
            python_cli,
            workflow_run_id.clone(),
            events_tx.clone(),
            repos.logs.clone(),
            options.num_workers,
        );

        for input in inputs {
            events_tx
                .send(Event {
                    id: EventId::new(),
                    is_replay: false,
                    timestamp: SystemTime::now(),
                    kind: EventKind::ChannelItemInserted(ChannelItemInsertedData {
                        channel_id: schema.input_channel_id.clone(),
                        item: input,
                    }),
                    source: Source::Input,
                    run_id: workflow_run_id.clone(),
                })
                .map_err(|_| anyhow!("workflow actor stopped before accepting inputs"))?;
        }

        let actor = RunHandle::spawn(RunContext {
            events: events_rx,
            runtime: runtime.clone(),
            repos,
            run_id: workflow_run_id.clone(),
            schema,
        });

        Ok(Self {
            id: workflow_run_id,
            db_id: db_run.id,
            workflow_id,
            actor,
            runtime,
        })
    }

    pub async fn cancel(&self) -> Result<()> {
        // todo: add db cleanup side effects
        self.actor.cancel().await;
        self.runtime.cancel().await
    }

    pub fn subscribe(&self) -> Result<ActorStateRx> {
        Ok(self.actor.state_rx.clone())
    }
}
