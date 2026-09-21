use crate::{
    api::interface::{RunJobArgs, RunJobResult},
    models::{Event, JobRunSource, SequenceId},
};

#[derive(Clone)]
pub struct Dependencies<S, R> {
    store: S,
    runtime: R,
}

impl<S, R> Dependencies<S, R> {
    pub fn new(store: S, runtime: R) -> Self {
        Self { store, runtime }
    }
}

pub trait AppDeps: Clone + Send + Sync + 'static {
    type EventStream: EventStream;
    type JobRuntime: JobRuntime;

    fn stream(&self) -> &Self::EventStream;
    fn runtime(&self) -> &Self::JobRuntime;
}

impl<S, R> AppDeps for Dependencies<S, R>
where
    S: EventStream + Clone + Send + Sync + 'static,
    R: JobRuntime + Clone + Send + Sync + 'static,
{
    type EventStream = S;
    type JobRuntime = R;

    fn stream(&self) -> &S {
        &self.store
    }

    fn runtime(&self) -> &R {
        &self.runtime
    }
}

pub trait EventStream: Clone + Send + Sync + 'static {
    fn append(&self, events: Vec<Event>) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
    fn get(
        &self,
        id: SequenceId,
    ) -> impl Future<Output = Result<Option<Event>, anyhow::Error>> + Send;
}

pub trait JobRuntime: Clone + Send + Sync + 'static {
    fn run_job(
        &self,
        args: RunJobArgs,
    ) -> impl Future<Output = Result<RunJobResult, anyhow::Error>> + Send;
    fn stop_job(
        &self,
        source: JobRunSource,
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
}
