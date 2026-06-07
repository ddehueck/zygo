"""gRPC client for communicating with the orchestrator."""

from __future__ import annotations

from typing import TYPE_CHECKING

import grpc
from zygo._internal.grpc.proto.orchestrator_pb2 import HandleJobRunEventRequest
from zygo._internal.grpc.proto.orchestrator_pb2_grpc import OrchestratorServiceStub
from zygo._internal.grpc.utils import build_proto_event

if TYPE_CHECKING:
    from zygo._internal.grpc.proto.orchestrator_pb2 import RegisterWorkflowResponse
    from zygo._internal.grpc.types import EventKind, WorkflowRegistration
    from zygo.types import RunEventContext


class OrchestratorClient:
    """Typed client for the orchestrator gRPC service."""

    def __init__(self, channel: grpc.Channel) -> None:
        super().__init__()
        self.stub = OrchestratorServiceStub(channel)

    def register_workflow(
        self,
        registration: WorkflowRegistration,
    ) -> RegisterWorkflowResponse:
        """Register a workflow and return its version id."""
        return self.stub.RegisterWorkflow(registration.to_proto())

    def emit(
        self,
        *,
        context: RunEventContext,
        event: EventKind,
    ) -> None:
        """Emit a run event to the orchestrator.

        client.emit(
            context=RunEventContext(
                workflow_version_id="wfv_abc",
                workflow_run_id="run_123",
                source=JobRunSource(job_id="j1", job_run_id="jr_42"),
            ),
            event=JobStarted(),
            )
        """
        proto_event = build_proto_event(context, event)
        try:
            self.stub.HandleJobRunEvent(HandleJobRunEventRequest(event=proto_event))
        except grpc.RpcError as e:
            raise RuntimeError(f"Error emitting event: {e}") from e
