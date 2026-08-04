"""
These types define the contract between this python lib and the workflow engine.
A future update should add codegen for interface consistency across the two programs.
"""

from dataclasses import dataclass
from typing import Literal


@dataclass
class ChannelMetadata:
    id: str


@dataclass
class JobMetadata:
    id: str
    content_hash: str


@dataclass
class EdgeMetadata:
    job_id: str
    channel_id: str
    kind: Literal["input", "output"]


@dataclass
class WorkflowMetadata:
    id: str
    content_hash: str
    channels: list[ChannelMetadata]
    jobs: list[JobMetadata]
    edges: list[EdgeMetadata]


@dataclass
class JobRunArgs:
    job_id: str
    data_reference_uri: str
    data_reference_etag: str
    workflow_run_id: str
    job_run_id: str
