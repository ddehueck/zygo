import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SortOrder(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    SORT_ORDER_UNSPECIFIED: _ClassVar[SortOrder]
    SORT_ORDER_ASC: _ClassVar[SortOrder]
    SORT_ORDER_DESC: _ClassVar[SortOrder]

class EdgeKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EDGE_KIND_UNSPECIFIED: _ClassVar[EdgeKind]
    EDGE_KIND_INPUT: _ClassVar[EdgeKind]
    EDGE_KIND_OUTPUT: _ClassVar[EdgeKind]
SORT_ORDER_UNSPECIFIED: SortOrder
SORT_ORDER_ASC: SortOrder
SORT_ORDER_DESC: SortOrder
EDGE_KIND_UNSPECIFIED: EdgeKind
EDGE_KIND_INPUT: EdgeKind
EDGE_KIND_OUTPUT: EdgeKind

class RegisterWorkflowRequest(_message.Message):
    __slots__ = ("name", "content_hash", "channels", "jobs")
    NAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_HASH_FIELD_NUMBER: _ClassVar[int]
    CHANNELS_FIELD_NUMBER: _ClassVar[int]
    JOBS_FIELD_NUMBER: _ClassVar[int]
    name: str
    content_hash: str
    channels: _containers.RepeatedCompositeFieldContainer[ChannelSchema]
    jobs: _containers.RepeatedCompositeFieldContainer[JobSchema]
    def __init__(self, name: _Optional[str] = ..., content_hash: _Optional[str] = ..., channels: _Optional[_Iterable[_Union[ChannelSchema, _Mapping]]] = ..., jobs: _Optional[_Iterable[_Union[JobSchema, _Mapping]]] = ...) -> None: ...

class ChannelSchema(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class JobSchema(_message.Message):
    __slots__ = ("name", "content_hash", "input_channel_name", "output_channel_names", "local_entrypoint", "remote_entrypoint")
    NAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_HASH_FIELD_NUMBER: _ClassVar[int]
    INPUT_CHANNEL_NAME_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_CHANNEL_NAMES_FIELD_NUMBER: _ClassVar[int]
    LOCAL_ENTRYPOINT_FIELD_NUMBER: _ClassVar[int]
    REMOTE_ENTRYPOINT_FIELD_NUMBER: _ClassVar[int]
    name: str
    content_hash: str
    input_channel_name: str
    output_channel_names: _containers.RepeatedScalarFieldContainer[str]
    local_entrypoint: LocalEntrypoint
    remote_entrypoint: RemoteEntrypoint
    def __init__(self, name: _Optional[str] = ..., content_hash: _Optional[str] = ..., input_channel_name: _Optional[str] = ..., output_channel_names: _Optional[_Iterable[str]] = ..., local_entrypoint: _Optional[_Union[LocalEntrypoint, _Mapping]] = ..., remote_entrypoint: _Optional[_Union[RemoteEntrypoint, _Mapping]] = ...) -> None: ...

class RegisterWorkflowResponse(_message.Message):
    __slots__ = ("workflow_version_id", "channel_ids_by_name", "job_ids_by_name")
    class ChannelIdsByNameEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class JobIdsByNameEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    CHANNEL_IDS_BY_NAME_FIELD_NUMBER: _ClassVar[int]
    JOB_IDS_BY_NAME_FIELD_NUMBER: _ClassVar[int]
    workflow_version_id: str
    channel_ids_by_name: _containers.ScalarMap[str, str]
    job_ids_by_name: _containers.ScalarMap[str, str]
    def __init__(self, workflow_version_id: _Optional[str] = ..., channel_ids_by_name: _Optional[_Mapping[str, str]] = ..., job_ids_by_name: _Optional[_Mapping[str, str]] = ...) -> None: ...

class DataReference(_message.Message):
    __slots__ = ("uri", "etag", "content_type", "size_bytes")
    URI_FIELD_NUMBER: _ClassVar[int]
    ETAG_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    uri: str
    etag: str
    content_type: str
    size_bytes: int
    def __init__(self, uri: _Optional[str] = ..., etag: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ...) -> None: ...

class HandleJobRunEventRequest(_message.Message):
    __slots__ = ("event",)
    EVENT_FIELD_NUMBER: _ClassVar[int]
    event: JobRunEvent
    def __init__(self, event: _Optional[_Union[JobRunEvent, _Mapping]] = ...) -> None: ...

class HandleJobRunEventResponse(_message.Message):
    __slots__ = ("event",)
    EVENT_FIELD_NUMBER: _ClassVar[int]
    event: JobRunEvent
    def __init__(self, event: _Optional[_Union[JobRunEvent, _Mapping]] = ...) -> None: ...

class JobRunEvent(_message.Message):
    __slots__ = ("id", "workflow_version_id", "workflow_run_id", "input_source", "job_run_source", "data_reference_inserted", "channel_item_inserted", "job_requested", "job_started", "job_succeeded", "job_failed")
    ID_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_RUN_ID_FIELD_NUMBER: _ClassVar[int]
    INPUT_SOURCE_FIELD_NUMBER: _ClassVar[int]
    JOB_RUN_SOURCE_FIELD_NUMBER: _ClassVar[int]
    DATA_REFERENCE_INSERTED_FIELD_NUMBER: _ClassVar[int]
    CHANNEL_ITEM_INSERTED_FIELD_NUMBER: _ClassVar[int]
    JOB_REQUESTED_FIELD_NUMBER: _ClassVar[int]
    JOB_STARTED_FIELD_NUMBER: _ClassVar[int]
    JOB_SUCCEEDED_FIELD_NUMBER: _ClassVar[int]
    JOB_FAILED_FIELD_NUMBER: _ClassVar[int]
    id: str
    workflow_version_id: str
    workflow_run_id: str
    input_source: InputSource
    job_run_source: JobRunSource
    data_reference_inserted: DataReferenceInsertedEvent
    channel_item_inserted: ChannelItemInsertedEvent
    job_requested: JobRequestedEvent
    job_started: JobStartedEvent
    job_succeeded: JobSucceededEvent
    job_failed: JobFailedEvent
    def __init__(self, id: _Optional[str] = ..., workflow_version_id: _Optional[str] = ..., workflow_run_id: _Optional[str] = ..., input_source: _Optional[_Union[InputSource, _Mapping]] = ..., job_run_source: _Optional[_Union[JobRunSource, _Mapping]] = ..., data_reference_inserted: _Optional[_Union[DataReferenceInsertedEvent, _Mapping]] = ..., channel_item_inserted: _Optional[_Union[ChannelItemInsertedEvent, _Mapping]] = ..., job_requested: _Optional[_Union[JobRequestedEvent, _Mapping]] = ..., job_started: _Optional[_Union[JobStartedEvent, _Mapping]] = ..., job_succeeded: _Optional[_Union[JobSucceededEvent, _Mapping]] = ..., job_failed: _Optional[_Union[JobFailedEvent, _Mapping]] = ...) -> None: ...

class InputSource(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class JobRunSource(_message.Message):
    __slots__ = ("job_id", "job_run_id")
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    JOB_RUN_ID_FIELD_NUMBER: _ClassVar[int]
    job_id: str
    job_run_id: str
    def __init__(self, job_id: _Optional[str] = ..., job_run_id: _Optional[str] = ...) -> None: ...

class DataReferenceInsertedEvent(_message.Message):
    __slots__ = ("data_reference",)
    DATA_REFERENCE_FIELD_NUMBER: _ClassVar[int]
    data_reference: DataReference
    def __init__(self, data_reference: _Optional[_Union[DataReference, _Mapping]] = ...) -> None: ...

class ChannelItemInsertedEvent(_message.Message):
    __slots__ = ("channel_id", "data_reference")
    CHANNEL_ID_FIELD_NUMBER: _ClassVar[int]
    DATA_REFERENCE_FIELD_NUMBER: _ClassVar[int]
    channel_id: str
    data_reference: DataReference
    def __init__(self, channel_id: _Optional[str] = ..., data_reference: _Optional[_Union[DataReference, _Mapping]] = ...) -> None: ...

class JobRequestedEvent(_message.Message):
    __slots__ = ("job_id", "data_reference", "requested_at")
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    DATA_REFERENCE_FIELD_NUMBER: _ClassVar[int]
    REQUESTED_AT_FIELD_NUMBER: _ClassVar[int]
    job_id: str
    data_reference: DataReference
    requested_at: _timestamp_pb2.Timestamp
    def __init__(self, job_id: _Optional[str] = ..., data_reference: _Optional[_Union[DataReference, _Mapping]] = ..., requested_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class JobStartedEvent(_message.Message):
    __slots__ = ("started_at",)
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    started_at: _timestamp_pb2.Timestamp
    def __init__(self, started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class JobSucceededEvent(_message.Message):
    __slots__ = ("succeeded_at",)
    SUCCEEDED_AT_FIELD_NUMBER: _ClassVar[int]
    succeeded_at: _timestamp_pb2.Timestamp
    def __init__(self, succeeded_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class JobFailedEvent(_message.Message):
    __slots__ = ("failed_at", "error_message")
    FAILED_AT_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    failed_at: _timestamp_pb2.Timestamp
    error_message: str
    def __init__(self, failed_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., error_message: _Optional[str] = ...) -> None: ...

class LocalEntrypoint(_message.Message):
    __slots__ = ("cwd", "exec")
    CWD_FIELD_NUMBER: _ClassVar[int]
    EXEC_FIELD_NUMBER: _ClassVar[int]
    cwd: str
    exec: str
    def __init__(self, cwd: _Optional[str] = ..., exec: _Optional[str] = ...) -> None: ...

class RemoteEntrypoint(_message.Message):
    __slots__ = ("url", "headers")
    class HeadersEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    URL_FIELD_NUMBER: _ClassVar[int]
    HEADERS_FIELD_NUMBER: _ClassVar[int]
    url: str
    headers: _containers.ScalarMap[str, str]
    def __init__(self, url: _Optional[str] = ..., headers: _Optional[_Mapping[str, str]] = ...) -> None: ...

class GetWorkflowVersionSchemaRequest(_message.Message):
    __slots__ = ("workflow_version_id",)
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    workflow_version_id: str
    def __init__(self, workflow_version_id: _Optional[str] = ...) -> None: ...

class GetWorkflowVersionSchemaResponse(_message.Message):
    __slots__ = ("workflow_version_id", "jobs", "channels", "edges")
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    JOBS_FIELD_NUMBER: _ClassVar[int]
    CHANNELS_FIELD_NUMBER: _ClassVar[int]
    EDGES_FIELD_NUMBER: _ClassVar[int]
    workflow_version_id: str
    jobs: _containers.RepeatedCompositeFieldContainer[ProtoJob]
    channels: _containers.RepeatedCompositeFieldContainer[ProtoChannel]
    edges: _containers.RepeatedCompositeFieldContainer[ProtoEdge]
    def __init__(self, workflow_version_id: _Optional[str] = ..., jobs: _Optional[_Iterable[_Union[ProtoJob, _Mapping]]] = ..., channels: _Optional[_Iterable[_Union[ProtoChannel, _Mapping]]] = ..., edges: _Optional[_Iterable[_Union[ProtoEdge, _Mapping]]] = ...) -> None: ...

class ListRunsRequest(_message.Message):
    __slots__ = ("workflow_id", "run_id", "sort", "limit")
    WORKFLOW_ID_FIELD_NUMBER: _ClassVar[int]
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    SORT_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    workflow_id: str
    run_id: str
    sort: SortOrder
    limit: int
    def __init__(self, workflow_id: _Optional[str] = ..., run_id: _Optional[str] = ..., sort: _Optional[_Union[SortOrder, str]] = ..., limit: _Optional[int] = ...) -> None: ...

class ListRunsResponse(_message.Message):
    __slots__ = ("runs", "next_cursor", "prev_cursor")
    RUNS_FIELD_NUMBER: _ClassVar[int]
    NEXT_CURSOR_FIELD_NUMBER: _ClassVar[int]
    PREV_CURSOR_FIELD_NUMBER: _ClassVar[int]
    runs: _containers.RepeatedCompositeFieldContainer[ProtoRun]
    next_cursor: ListRunsCursor
    prev_cursor: ListRunsCursor
    def __init__(self, runs: _Optional[_Iterable[_Union[ProtoRun, _Mapping]]] = ..., next_cursor: _Optional[_Union[ListRunsCursor, _Mapping]] = ..., prev_cursor: _Optional[_Union[ListRunsCursor, _Mapping]] = ...) -> None: ...

class ListRunsCursor(_message.Message):
    __slots__ = ("workflow_id", "run_id", "sort", "limit")
    WORKFLOW_ID_FIELD_NUMBER: _ClassVar[int]
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    SORT_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    workflow_id: str
    run_id: str
    sort: SortOrder
    limit: int
    def __init__(self, workflow_id: _Optional[str] = ..., run_id: _Optional[str] = ..., sort: _Optional[_Union[SortOrder, str]] = ..., limit: _Optional[int] = ...) -> None: ...

class ListRunEventsRequest(_message.Message):
    __slots__ = ("run_id", "sequence_number", "sort", "limit")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_NUMBER_FIELD_NUMBER: _ClassVar[int]
    SORT_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    sequence_number: int
    sort: SortOrder
    limit: int
    def __init__(self, run_id: _Optional[str] = ..., sequence_number: _Optional[int] = ..., sort: _Optional[_Union[SortOrder, str]] = ..., limit: _Optional[int] = ...) -> None: ...

class ListRunEventsResponse(_message.Message):
    __slots__ = ("events", "next_cursor", "prev_cursor")
    EVENTS_FIELD_NUMBER: _ClassVar[int]
    NEXT_CURSOR_FIELD_NUMBER: _ClassVar[int]
    PREV_CURSOR_FIELD_NUMBER: _ClassVar[int]
    events: _containers.RepeatedCompositeFieldContainer[ProtoEvent]
    next_cursor: ListRunEventsCursor
    prev_cursor: ListRunEventsCursor
    def __init__(self, events: _Optional[_Iterable[_Union[ProtoEvent, _Mapping]]] = ..., next_cursor: _Optional[_Union[ListRunEventsCursor, _Mapping]] = ..., prev_cursor: _Optional[_Union[ListRunEventsCursor, _Mapping]] = ...) -> None: ...

class ListRunEventsCursor(_message.Message):
    __slots__ = ("run_id", "sequence_number", "sort", "limit")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_NUMBER_FIELD_NUMBER: _ClassVar[int]
    SORT_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    sequence_number: int
    sort: SortOrder
    limit: int
    def __init__(self, run_id: _Optional[str] = ..., sequence_number: _Optional[int] = ..., sort: _Optional[_Union[SortOrder, str]] = ..., limit: _Optional[int] = ...) -> None: ...

class ListWorkflowsRequest(_message.Message):
    __slots__ = ("workflow_id", "sort", "limit")
    WORKFLOW_ID_FIELD_NUMBER: _ClassVar[int]
    SORT_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    workflow_id: str
    sort: SortOrder
    limit: int
    def __init__(self, workflow_id: _Optional[str] = ..., sort: _Optional[_Union[SortOrder, str]] = ..., limit: _Optional[int] = ...) -> None: ...

class ListWorkflowsResponse(_message.Message):
    __slots__ = ("workflows", "next_cursor", "prev_cursor")
    WORKFLOWS_FIELD_NUMBER: _ClassVar[int]
    NEXT_CURSOR_FIELD_NUMBER: _ClassVar[int]
    PREV_CURSOR_FIELD_NUMBER: _ClassVar[int]
    workflows: _containers.RepeatedCompositeFieldContainer[ProtoWorkflow]
    next_cursor: ListWorkflowsCursor
    prev_cursor: ListWorkflowsCursor
    def __init__(self, workflows: _Optional[_Iterable[_Union[ProtoWorkflow, _Mapping]]] = ..., next_cursor: _Optional[_Union[ListWorkflowsCursor, _Mapping]] = ..., prev_cursor: _Optional[_Union[ListWorkflowsCursor, _Mapping]] = ...) -> None: ...

class ListWorkflowsCursor(_message.Message):
    __slots__ = ("workflow_id", "sort", "limit")
    WORKFLOW_ID_FIELD_NUMBER: _ClassVar[int]
    SORT_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    workflow_id: str
    sort: SortOrder
    limit: int
    def __init__(self, workflow_id: _Optional[str] = ..., sort: _Optional[_Union[SortOrder, str]] = ..., limit: _Optional[int] = ...) -> None: ...

class LivestreamRequest(_message.Message):
    __slots__ = ("cursor",)
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cursor: LivestreamCursor
    def __init__(self, cursor: _Optional[_Union[LivestreamCursor, _Mapping]] = ...) -> None: ...

class LivestreamResponse(_message.Message):
    __slots__ = ("workflows", "runs", "events", "next_cursor")
    WORKFLOWS_FIELD_NUMBER: _ClassVar[int]
    RUNS_FIELD_NUMBER: _ClassVar[int]
    EVENTS_FIELD_NUMBER: _ClassVar[int]
    NEXT_CURSOR_FIELD_NUMBER: _ClassVar[int]
    workflows: _containers.RepeatedCompositeFieldContainer[ProtoWorkflow]
    runs: _containers.RepeatedCompositeFieldContainer[ProtoRun]
    events: _containers.RepeatedCompositeFieldContainer[ProtoEvent]
    next_cursor: LivestreamCursor
    def __init__(self, workflows: _Optional[_Iterable[_Union[ProtoWorkflow, _Mapping]]] = ..., runs: _Optional[_Iterable[_Union[ProtoRun, _Mapping]]] = ..., events: _Optional[_Iterable[_Union[ProtoEvent, _Mapping]]] = ..., next_cursor: _Optional[_Union[LivestreamCursor, _Mapping]] = ...) -> None: ...

class LivestreamCursor(_message.Message):
    __slots__ = ("workflows", "runs", "events")
    WORKFLOWS_FIELD_NUMBER: _ClassVar[int]
    RUNS_FIELD_NUMBER: _ClassVar[int]
    EVENTS_FIELD_NUMBER: _ClassVar[int]
    workflows: LivestreamWorkflowCursor
    runs: LivestreamRunCursor
    events: LivestreamEventCursor
    def __init__(self, workflows: _Optional[_Union[LivestreamWorkflowCursor, _Mapping]] = ..., runs: _Optional[_Union[LivestreamRunCursor, _Mapping]] = ..., events: _Optional[_Union[LivestreamEventCursor, _Mapping]] = ...) -> None: ...

class LivestreamWorkflowCursor(_message.Message):
    __slots__ = ("workflow_id", "limit")
    WORKFLOW_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    workflow_id: str
    limit: int
    def __init__(self, workflow_id: _Optional[str] = ..., limit: _Optional[int] = ...) -> None: ...

class LivestreamRunCursor(_message.Message):
    __slots__ = ("run_id", "limit")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    limit: int
    def __init__(self, run_id: _Optional[str] = ..., limit: _Optional[int] = ...) -> None: ...

class LivestreamEventCursor(_message.Message):
    __slots__ = ("event_id", "sequence_number", "limit")
    EVENT_ID_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_NUMBER_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    event_id: str
    sequence_number: int
    limit: int
    def __init__(self, event_id: _Optional[str] = ..., sequence_number: _Optional[int] = ..., limit: _Optional[int] = ...) -> None: ...

class ProtoJob(_message.Message):
    __slots__ = ("id", "workflow_version_id", "name", "content_hash")
    ID_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_HASH_FIELD_NUMBER: _ClassVar[int]
    id: str
    workflow_version_id: str
    name: str
    content_hash: str
    def __init__(self, id: _Optional[str] = ..., workflow_version_id: _Optional[str] = ..., name: _Optional[str] = ..., content_hash: _Optional[str] = ...) -> None: ...

class ProtoChannel(_message.Message):
    __slots__ = ("id", "name", "workflow_version_id")
    ID_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    id: str
    name: str
    workflow_version_id: str
    def __init__(self, id: _Optional[str] = ..., name: _Optional[str] = ..., workflow_version_id: _Optional[str] = ...) -> None: ...

class ProtoEdge(_message.Message):
    __slots__ = ("workflow_version_id", "job_id", "channel_id", "kind")
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    CHANNEL_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    workflow_version_id: str
    job_id: str
    channel_id: str
    kind: EdgeKind
    def __init__(self, workflow_version_id: _Optional[str] = ..., job_id: _Optional[str] = ..., channel_id: _Optional[str] = ..., kind: _Optional[_Union[EdgeKind, str]] = ...) -> None: ...

class ProtoRun(_message.Message):
    __slots__ = ("id", "workflow_version_id", "created_at")
    ID_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    id: str
    workflow_version_id: str
    created_at: _timestamp_pb2.Timestamp
    def __init__(self, id: _Optional[str] = ..., workflow_version_id: _Optional[str] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class ProtoWorkflow(_message.Message):
    __slots__ = ("id", "name")
    ID_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    id: str
    name: str
    def __init__(self, id: _Optional[str] = ..., name: _Optional[str] = ...) -> None: ...

class ProtoEvent(_message.Message):
    __slots__ = ("id", "workflow_version_id", "is_replay", "timestamp", "workflow_run_id", "sequence_number", "input_source", "job_run_source", "data_reference_inserted", "channel_item_inserted", "job_requested", "job_started", "job_succeeded", "job_failed")
    ID_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_VERSION_ID_FIELD_NUMBER: _ClassVar[int]
    IS_REPLAY_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_RUN_ID_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_NUMBER_FIELD_NUMBER: _ClassVar[int]
    INPUT_SOURCE_FIELD_NUMBER: _ClassVar[int]
    JOB_RUN_SOURCE_FIELD_NUMBER: _ClassVar[int]
    DATA_REFERENCE_INSERTED_FIELD_NUMBER: _ClassVar[int]
    CHANNEL_ITEM_INSERTED_FIELD_NUMBER: _ClassVar[int]
    JOB_REQUESTED_FIELD_NUMBER: _ClassVar[int]
    JOB_STARTED_FIELD_NUMBER: _ClassVar[int]
    JOB_SUCCEEDED_FIELD_NUMBER: _ClassVar[int]
    JOB_FAILED_FIELD_NUMBER: _ClassVar[int]
    id: str
    workflow_version_id: str
    is_replay: bool
    timestamp: _timestamp_pb2.Timestamp
    workflow_run_id: str
    sequence_number: int
    input_source: InputSource
    job_run_source: JobRunSource
    data_reference_inserted: DataReferenceInsertedEvent
    channel_item_inserted: ChannelItemInsertedEvent
    job_requested: JobRequestedEvent
    job_started: JobStartedEvent
    job_succeeded: JobSucceededEvent
    job_failed: JobFailedEvent
    def __init__(self, id: _Optional[str] = ..., workflow_version_id: _Optional[str] = ..., is_replay: bool = ..., timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., workflow_run_id: _Optional[str] = ..., sequence_number: _Optional[int] = ..., input_source: _Optional[_Union[InputSource, _Mapping]] = ..., job_run_source: _Optional[_Union[JobRunSource, _Mapping]] = ..., data_reference_inserted: _Optional[_Union[DataReferenceInsertedEvent, _Mapping]] = ..., channel_item_inserted: _Optional[_Union[ChannelItemInsertedEvent, _Mapping]] = ..., job_requested: _Optional[_Union[JobRequestedEvent, _Mapping]] = ..., job_started: _Optional[_Union[JobStartedEvent, _Mapping]] = ..., job_succeeded: _Optional[_Union[JobSucceededEvent, _Mapping]] = ..., job_failed: _Optional[_Union[JobFailedEvent, _Mapping]] = ...) -> None: ...
