"""Public API for the gRPC layer.

External consumers should access types via this namespace::

    from zygo import grpc

    registration = grpc.WorkflowRegistration(...)
    client = grpc.OrchestratorClient(channel)
"""

from zygo._internal.grpc.client import OrchestratorClient
from zygo._internal.grpc.proto.orchestrator_pb2 import RegisterWorkflowResponse
from zygo._internal.grpc.types import (
    ChannelItemInserted,
    ChannelSchema,
    DataReference,
    DataReferenceInserted,
    EventKind,
    EventSource,
    InputSource,
    JobFailed,
    JobRequested,
    JobRunSource,
    JobSchema,
    JobStarted,
    JobSucceeded,
    WorkflowRegistration,
)

__all__ = [
    "ChannelItemInserted",
    "ChannelSchema",
    "DataReference",
    "DataReferenceInserted",
    "EventKind",
    "EventSource",
    "InputSource",
    "JobFailed",
    "JobRequested",
    "JobRunSource",
    "JobSchema",
    "JobStarted",
    "JobSucceeded",
    "OrchestratorClient",
    "RegisterWorkflowResponse",
    "WorkflowRegistration",
]
