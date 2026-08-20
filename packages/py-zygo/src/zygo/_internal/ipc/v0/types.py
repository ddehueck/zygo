"""
These types define the contract between this python lib and the workflow engine.
A future update should add codegen for interface consistency across the two programs.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
import json
import os
from pathlib import Path
import sys
from typing import TYPE_CHECKING, Literal, TextIO

if TYPE_CHECKING:
    from zygo.store import Reference

STDOUT_IPC_PREFIX: str = "ZYGO_IPC="


@dataclass(frozen=True)
class DataReference:
    """The data-reference shape consumed by the Rust IPC parser."""

    uri: str
    version: str

    @classmethod
    def from_reference(cls, reference: Reference) -> DataReference:
        """Convert a store reference to the cross-language IPC representation."""
        return cls(
            uri=str(reference.uri),
            version=reference.etag,
        )


@dataclass(frozen=True)
class DataReferenceCreated:
    type: Literal["data_reference_created"] = field(
        default="data_reference_created", init=False
    )
    data_reference: DataReference


@dataclass(frozen=True)
class ChannelItemInserted:
    type: Literal["channel_item_inserted"] = field(
        default="channel_item_inserted", init=False
    )
    channel_id: str
    data_reference: DataReference


@dataclass(frozen=True)
class TagInserted:
    type: Literal["tag_inserted"] = field(default="tag_inserted", init=False)
    name: str
    value: str
    data_reference: DataReference | None = None


type StdoutIPCMessage = DataReferenceCreated | ChannelItemInserted | TagInserted


def _serialize_stdout_ipc_message(message: StdoutIPCMessage) -> str:
    """Serialize an IPC message, including the prefix expected by Rust."""
    payload = json.dumps(asdict(message), separators=(",", ":"))
    return f"{STDOUT_IPC_PREFIX}{payload}"


def write_stdout_ipc_message(message: StdoutIPCMessage) -> None:
    """Write one flushed, parseable IPC message line to stdout.

    A closed IPC reader leaves Python's stdout buffer pointing at a broken
    pipe. Replace it with ``os.devnull`` after handling that condition so
    interpreter shutdown does not report a second flush error.
    """
    serialized = _serialize_stdout_ipc_message(message)
    stdout: TextIO = sys.stdout
    try:
        stdout.write(f"{serialized}\n")
        stdout.flush()
    except BrokenPipeError:
        # Keep stdout open for interpreter shutdown, but detach it from the pipe.
        sys.stdout = Path(os.devnull).open("w", encoding="utf-8")  # ruff: ignore[open-file-with-context-handler]


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


@dataclass
class JobRunArgs:
    job_id: str
    data_reference_uri: str
    data_reference_version: str
    workflow_run_id: str
    job_run_id: str
