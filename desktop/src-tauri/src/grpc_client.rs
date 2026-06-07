use tonic::transport::Channel;

pub use zygo_core::orchestrator_proto;
pub use zygo_core::orchestrator_proto::orchestrator_service_client::OrchestratorServiceClient;
pub use zygo_core::orchestrator_proto::{
    GetWorkflowVersionSchemaRequest, GetWorkflowVersionSchemaResponse, ListRunEventsRequest,
    ListRunEventsResponse, ListRunsRequest, ListRunsResponse, ListWorkflowsRequest,
    ListWorkflowsResponse, LivestreamCursor, LivestreamEventCursor, LivestreamRequest,
    LivestreamResponse, LivestreamRunCursor, LivestreamWorkflowCursor, ProtoChannel, ProtoEdge,
    ProtoEvent, ProtoJob, ProtoRun, ProtoWorkflow,
};

pub struct Client {
    inner: OrchestratorServiceClient<Channel>,
}

impl Client {
    pub async fn connect(addr: &str) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(addr.to_string())?.connect().await?;
        Ok(Self {
            inner: OrchestratorServiceClient::new(channel),
        })
    }

    pub async fn list_workflows(
        &mut self,
        workflow_id: Option<String>,
        sort: i32,
        limit: u32,
    ) -> anyhow::Result<ListWorkflowsResponse> {
        let request = tonic::Request::new(ListWorkflowsRequest {
            workflow_id,
            sort,
            limit,
        });
        let response = self.inner.list_workflows(request).await?;
        Ok(response.into_inner())
    }

    pub async fn get_workflow_version_schema(
        &mut self,
        workflow_version_id: String,
    ) -> anyhow::Result<GetWorkflowVersionSchemaResponse> {
        let request = tonic::Request::new(GetWorkflowVersionSchemaRequest {
            workflow_version_id,
        });
        let response = self.inner.get_workflow_version_schema(request).await?;
        Ok(response.into_inner())
    }

    pub async fn list_runs(
        &mut self,
        workflow_id: String,
        run_id: Option<String>,
        sort: i32,
        limit: u32,
    ) -> anyhow::Result<ListRunsResponse> {
        let request = tonic::Request::new(ListRunsRequest {
            workflow_id,
            run_id,
            sort,
            limit,
        });
        let response = self.inner.list_runs(request).await?;
        Ok(response.into_inner())
    }

    pub async fn list_run_events(
        &mut self,
        run_id: String,
        sequence_number: Option<i64>,
        sort: i32,
        limit: u32,
    ) -> anyhow::Result<ListRunEventsResponse> {
        let request = tonic::Request::new(ListRunEventsRequest {
            run_id,
            sequence_number,
            sort,
            limit,
        });
        let response = self.inner.list_run_events(request).await?;
        Ok(response.into_inner())
    }

    pub async fn livestream(
        &mut self,
        cursor: Option<LivestreamCursor>,
    ) -> anyhow::Result<LivestreamResponse> {
        let request = tonic::Request::new(LivestreamRequest { cursor });
        let response = self.inner.livestream(request).await?;
        Ok(response.into_inner())
    }
}
