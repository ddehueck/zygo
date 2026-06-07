"""Convert Python domain types to proto messages."""

from __future__ import annotations

from typing import TYPE_CHECKING

from google.protobuf.timestamp_pb2 import Timestamp

from zygo._internal.grpc.proto import orchestrator_pb2 as pb
from zygo._internal.grpc.types import (
    ChannelItemInserted,
    DataReferenceInserted,
    InputSource,
    JobFailed,
    JobRequested,
    JobRunSource,
    JobStarted,
    JobSucceeded,
)
from zygo._internal.utils.id import UUID7

if TYPE_CHECKING:
    from zygo._internal.grpc.types import EventKind
    from zygo.types import RunEventContext


def get_timestamp() -> Timestamp:
    """Return the current time as a protobuf Timestamp."""
    timestamp = Timestamp()
    timestamp.GetCurrentTime()
    return timestamp


def build_source(source: InputSource | JobRunSource) -> pb.JobRunEvent:
    """Return a partial JobRunEvent with the source oneof set."""
    event = pb.JobRunEvent()
    match source:
        case InputSource():
            event.input_source.CopyFrom(pb.InputSource())
        case JobRunSource(job_id=job_id, job_run_id=job_run_id):
            event.job_run_source.CopyFrom(
                pb.JobRunSource(job_id=job_id, job_run_id=job_run_id)
            )
    return event


def set_event_kind(proto: pb.JobRunEvent, event: EventKind) -> None:
    """Set the event-kind oneof on a proto JobRunEvent."""
    timestamp = get_timestamp()
    match event:
        case DataReferenceInserted(data_reference=ref):
            proto.data_reference_inserted.CopyFrom(
                pb.DataReferenceInsertedEvent(
                    data_reference=ref.to_proto(),
                )
            )
        case ChannelItemInserted(channel_id=channel_id, data_reference=ref):
            proto.channel_item_inserted.CopyFrom(
                pb.ChannelItemInsertedEvent(
                    channel_id=channel_id,
                    data_reference=ref.to_proto(),
                )
            )
        case JobRequested(job_id=job_id, data_reference=ref):
            proto.job_requested.CopyFrom(
                pb.JobRequestedEvent(
                    job_id=job_id,
                    data_reference=ref.to_proto(),
                    requested_at=timestamp,
                )
            )
        case JobStarted():
            proto.job_started.CopyFrom(pb.JobStartedEvent(started_at=timestamp))
        case JobSucceeded():
            proto.job_succeeded.CopyFrom(pb.JobSucceededEvent(succeeded_at=timestamp))
        case JobFailed(error_message=msg):
            proto.job_failed.CopyFrom(
                pb.JobFailedEvent(failed_at=timestamp, error_message=msg)
            )


def build_proto_event(context: RunEventContext, event: EventKind) -> pb.JobRunEvent:
    """Build a complete proto JobRunEvent from Python types."""
    proto = build_source(context.source)
    proto.id = str(UUID7.generate())
    proto.workflow_version_id = context.workflow_version_id
    proto.workflow_run_id = context.workflow_run_id
    set_event_kind(proto, event)
    return proto
