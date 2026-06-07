use tonic::{Request, Response, Status};

use crate::engine::RunScope;
use crate::models::OrchestratorMode;
use crate::orchestrator_proto::orchestrator_service_server::OrchestratorService as OrchestratorServiceTrait;
use crate::orchestrator_proto::{
    HandleEventRequest, HandleEventResponse, RegisterWorkflowRequest, RegisterWorkflowResponse,
};
use crate::store::{StorageProvider, Store};

use super::actions::Actions;
use super::parse::parse_job_run_event;

pub struct OrchestratorService<S: StorageProvider> {
    pub store: Store<S>,
    pub mode: OrchestratorMode,
}

impl<S: StorageProvider + 'static> OrchestratorService<S> {
    pub fn new(store: Store<S>, mode: OrchestratorMode) -> Self {
        Self { store, mode }
    }
}

#[tonic::async_trait]
impl<S: StorageProvider + 'static> OrchestratorServiceTrait for OrchestratorService<S> {
    async fn register_workflow(
        &self,
        request: Request<RegisterWorkflowRequest>,
    ) -> Result<Response<RegisterWorkflowResponse>, Status> {
        let input = request.into_inner();
        let output = Actions::new(self.store.clone())
            .register_workflow(input)
            .await
            .map_err(|e| Status::internal(format!("failed to register workflow: {e}")))?;

        Ok(Response::new(output))
    }

    async fn handle_event(
        &self,
        request: Request<HandleEventRequest>,
    ) -> Result<Response<HandleEventResponse>, Status> {
        let request = request.into_inner();
        let proto_event = request
            .event
            .ok_or_else(|| Status::invalid_argument("event is required"))?;

        let parsed = parse_job_run_event(proto_event.clone())?;
        let scope = RunScope::new(
            parsed.workflow_id,
            parsed.workflow_version_id,
            parsed.event.source.workflow_run_id().clone(),
        );

        Actions::new(self.store.clone())
            .handle_event(scope, parsed.event)
            .await
            .map_err(|e| Status::internal(format!("failed to handle event: {e}")))?;

        Ok(Response::new(HandleEventResponse {
            event: Some(proto_event),
        }))
    }
}
