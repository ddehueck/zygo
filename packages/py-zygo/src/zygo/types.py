from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, NewType

if TYPE_CHECKING:
    from zygo.store import Reference

WorkflowId = NewType("WorkflowId", str)
WorkflowRunId = NewType("WorkflowRunId", str)
ChannelId = NewType("ChannelId", str)
DataId = NewType("DataId", str)
JobId = NewType("JobId", str)
JobRunId = NewType("JobRunId", str)
JobFnName = NewType("JobFnName", str)
JobHash = NewType("JobHash", str)


@dataclass(frozen=True)
class JobRunContext:
    """Context for a running workflow job."""

    workflow_run_id: WorkflowRunId
    job_run_id: JobRunId
    data_ref: Reference


@dataclass(frozen=True)
class GPUConfig:
    type: str | None = None
    count: int | None = None


@dataclass(frozen=True)
class JobResourceConfig:
    cpu_cores: float | None = None
    memory_gb: int | None = None
    gpu: GPUConfig | None = None
