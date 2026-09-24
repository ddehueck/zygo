"""Protocol models are generated from protocol/v0/schema.json."""

from dataclasses import asdict
import json

from zygo.cli.v0.generated import (
    ChannelItemInserted,
    ChannelMetadata,
    DataReferenceCreated,
    HttpConfig,
    HttpIPCMessage,
    IpcMessage,
    JobMetadata,
    JobRunArgs,
    StoreConfig,
    TagInserted,
    WorkflowMetadata,
)

__all__ = [
    "STDOUT_IPC_PREFIX",
    "ChannelItemInserted",
    "ChannelMetadata",
    "DataReferenceCreated",
    "HttpConfig",
    "HttpIPCMessage",
    "IpcMessage",
    "JobMetadata",
    "JobRunArgs",
    "StoreConfig",
    "TagInserted",
    "WorkflowMetadata",
    "serialize_http_ipc_message",
    "serialize_ipc_message",
]

STDOUT_IPC_PREFIX = "ZYGO_IPC="


def serialize_http_ipc_message(message: HttpIPCMessage) -> str:
    """Serialize an HTTP IPC envelope to a compact JSON payload."""
    return json.dumps(asdict(message), separators=(",", ":"))


def serialize_ipc_message(message: IpcMessage) -> str:
    """Serialize an IPC message to a compact JSON payload."""
    return json.dumps(asdict(message), separators=(",", ":"))
