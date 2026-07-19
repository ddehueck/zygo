mod handle_event;
mod register;

use crate::models::Event;
use crate::orchestrator_proto::{RegisterWorkflowRequest, RegisterWorkflowResponse};
use crate::store::{StorageProvider, Store};

pub struct Actions<S: StorageProvider> {
    store: Store<S>,
}

impl<S: StorageProvider> Actions<S> {
    pub fn new(store: Store<S>) -> Self {
        Self { store }
    }

    pub async fn register_workflow(
        &self,
        input: RegisterWorkflowRequest,
    ) -> anyhow::Result<RegisterWorkflowResponse> {
        register::register_workflow(&self.store, input).await
    }

    pub async fn handle_event(&self, event: Event) -> anyhow::Result<()> {
        handle_event::handle_event(&self.store, event).await
    }
}
