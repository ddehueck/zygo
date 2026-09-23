"""
These types define the contract between this python lib and the workflow engine.
A future update should add codegen for interface consistency across the two programs.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
import json
from typing import Literal

STDOUT_IPC_PREFIX: str = "ZYGO_IPC="


@dataclass(frozen=True)
class DataReferenceCreated:
    type: Literal["data_reference_created"] = field(
        default="data_reference_created", init=False
    )
    data_reference: str


@dataclass(frozen=True)
class ChannelItemInserted:
    type: Literal["channel_item_inserted"] = field(
        default="channel_item_inserted", init=False
    )
    channel_id: str
    data_reference: str


@dataclass(frozen=True)
class TagInserted:
    type: Literal["tag_inserted"] = field(default="tag_inserted", init=False)
    value: str
    data_reference: str | None = None


type IpcMessage = DataReferenceCreated | ChannelItemInserted | TagInserted


def serialize_ipc_message(message: IpcMessage) -> str:
    """Serialize an IPC message to a compact JSON payload."""
    return json.dumps(asdict(message), separators=(",", ":"))


@dataclass
class ChannelMetadata:
    id: str
    accepted_file_extensions: list[str]


@dataclass
class JobMetadata:
    id: str
    content_hash: str
    input_channel_id: str
    output_channel_id: str


@dataclass
class WorkflowMetadata:
    id: str
    content_hash: str
    input_channel_id: str
    output_channel_id: str
    jobs: list[JobMetadata]
    channels: list[ChannelMetadata]


@dataclass
class JobRunArgs:
    job_id: str
    data_reference_uri: str
    workflow_run_id: str
    job_run_id: str
