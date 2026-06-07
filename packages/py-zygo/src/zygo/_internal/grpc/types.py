"""Friendly Python types for building gRPC requests.

Provides:
- Registration types (WorkflowRegistration, ChannelSchema, JobSchema, ...)
- Event types with discriminated unions for source and kind
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING

from zygo._internal.grpc.proto import orchestrator_pb2 as pb
from zygo.backends.protocol import (
    LocalEntrypoint as BackendLocalEntrypoint,
    RemoteEntrypoint as BackendRemoteEntrypoint,
)

if TYPE_CHECKING:
    from zygo.backends.protocol import Entrypoint
    from zygo.store.types import Reference


# ============================================================================
# Registration types
# ============================================================================


@dataclass(frozen=True)
class ChannelSchema:
    """A named channel declaration for a workflow."""

    name: str

    def to_proto(self) -> pb.ChannelSchema:
        return pb.ChannelSchema(name=self.name)


@dataclass(frozen=True)
class JobSchema:
    """Schema for a single job in the workflow."""

    name: str
    content_hash: str
    input_channel_name: str
    output_channel_names: list[str] = field(default_factory=list)
    entrypoint: Entrypoint | None = None

    def to_proto(self) -> pb.JobSchema:
        local_ep = None
        remote_ep = None
        if isinstance(self.entrypoint, BackendLocalEntrypoint):
            local_ep = pb.LocalEntrypoint(
                cwd=str(self.entrypoint.cwd),
                # "exec" is a Python builtin so we pass it via dict-splat
                **{"exec": self.entrypoint.exec},
            )
        elif isinstance(self.entrypoint, BackendRemoteEntrypoint):
            remote_ep = pb.RemoteEntrypoint(
                url=self.entrypoint.url,
                headers=self.entrypoint.headers,
            )
        return pb.JobSchema(
            name=self.name,
            content_hash=self.content_hash,
            input_channel_name=self.input_channel_name,
            output_channel_names=self.output_channel_names,
            local_entrypoint=local_ep,
            remote_entrypoint=remote_ep,
        )


@dataclass(frozen=True)
class WorkflowRegistration:
    """All the pieces needed to register a workflow."""

    name: str
    content_hash: str
    channels: list[ChannelSchema]
    jobs: list[JobSchema]

    def to_proto(self) -> pb.RegisterWorkflowRequest:
        return pb.RegisterWorkflowRequest(
            name=self.name,
            content_hash=self.content_hash,
            channels=[ch.to_proto() for ch in self.channels],
            jobs=[job.to_proto() for job in self.jobs],
        )


# ============================================================================
# Event source — discriminated union
# ============================================================================


@dataclass(frozen=True)
class InputSource:
    """Event originated from an external input (workflow trigger)."""


@dataclass(frozen=True)
class JobRunSource:
    """Event originated from within a specific job run."""

    job_id: str
    job_run_id: str


type EventSource = InputSource | JobRunSource


# ============================================================================
# Data reference
# ============================================================================


@dataclass(frozen=True)
class DataReference:
    """Metadata for a stored data object (mirrors the proto DataReference)."""

    uri: str
    etag: str
    content_type: str | None = None
    size_bytes: int | None = None

    @classmethod
    def from_store_ref(cls, ref: Reference) -> DataReference:
        """Create a DataReference from a store Reference."""
        return cls(
            uri=str(ref.uri),
            etag=ref.etag,
            content_type=ref.content_type,
            size_bytes=ref.size,
        )

    def to_proto(self) -> pb.DataReference:
        return pb.DataReference(
            uri=self.uri,
            etag=self.etag,
            content_type=self.content_type or "",
            size_bytes=self.size_bytes or 0,
        )


# ============================================================================
# Event kind — discriminated union
# ============================================================================


@dataclass(frozen=True)
class DataReferenceInserted:
    """A data reference was persisted in the store."""

    data_reference: DataReference


@dataclass(frozen=True)
class ChannelItemInserted:
    """An item was linked to a channel via its data reference."""

    channel_id: str
    data_reference: DataReference


@dataclass(frozen=True)
class JobRequested:
    """Execution of a job was requested with a given data reference."""

    job_id: str
    data_reference: DataReference


@dataclass(frozen=True)
class JobStarted:
    """A job began executing."""


@dataclass(frozen=True)
class JobSucceeded:
    """A job completed successfully."""


@dataclass(frozen=True)
class JobFailed:
    """A job terminated with an error."""

    error_message: str


type EventKind = (
    DataReferenceInserted
    | ChannelItemInserted
    | JobRequested
    | JobStarted
    | JobSucceeded
    | JobFailed
)
