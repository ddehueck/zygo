use crate::{
    actors::ActorTx,
    models::WorkflowRunId,
    store::{StorageProvider, Store},
    stream::StreamWriter,
    workers::WorkerPool,
};

pub struct ServiceContext<S: StorageProvider> {
    pub store: Store<S>,
    pub worker_pool: WorkerPool,
}

impl<S: StorageProvider> ServiceContext<S> {
    pub fn new(store: Store<S>, worker_pool: WorkerPool) -> Self {
        Self { store, worker_pool }
    }
}

impl<S: StorageProvider> Clone for ServiceContext<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            worker_pool: self.worker_pool.clone(),
        }
    }
}

pub struct RunContext<S: StorageProvider> {
    pub store: Store<S>,
    pub worker_pool: WorkerPool,
    pub run_id: WorkflowRunId,
}

impl<S: StorageProvider> RunContext<S> {
    pub fn new(context: &ServiceContext<S>, run_id: &WorkflowRunId) -> Self {
        Self {
            store: context.store.clone(),
            worker_pool: context.worker_pool.clone(),
            run_id: run_id.clone(),
        }
    }
}

impl<S: StorageProvider> From<&RunContext<S>> for ServiceContext<S> {
    fn from(context: &RunContext<S>) -> Self {
        Self {
            store: context.store.clone(),
            worker_pool: context.worker_pool.clone(),
        }
    }
}

impl<S: StorageProvider> Clone for RunContext<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            worker_pool: self.worker_pool.clone(),
            run_id: self.run_id.clone(),
        }
    }
}

pub struct ActorContext<S: StorageProvider> {
    pub store: Store<S>,
    pub worker_pool: WorkerPool,
    pub run_id: WorkflowRunId,
    pub actor_tx: ActorTx,
    pub stream_writer: StreamWriter,
}

impl<S: StorageProvider> ActorContext<S> {
    pub fn from(context: &RunContext<S>, actor_tx: ActorTx, stream_writer: StreamWriter) -> Self {
        Self {
            store: context.store.clone(),
            worker_pool: context.worker_pool.clone(),
            run_id: context.run_id.clone(),
            actor_tx,
            stream_writer,
        }
    }
}

impl<S: StorageProvider> From<&ActorContext<S>> for RunContext<S> {
    fn from(context: &ActorContext<S>) -> Self {
        Self {
            store: context.store.clone(),
            worker_pool: context.worker_pool.clone(),
            run_id: context.run_id.clone(),
        }
    }
}

impl<S: StorageProvider> Clone for ActorContext<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            worker_pool: self.worker_pool.clone(),
            run_id: self.run_id.clone(),
            actor_tx: self.actor_tx.clone(),
            stream_writer: self.stream_writer.clone(),
        }
    }
}
