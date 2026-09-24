from __future__ import annotations

from dataclasses import dataclass
from typing import NewType

from zygo.store import DataUri

WorkflowId = NewType("WorkflowId", str)
WorkflowRunId = NewType("WorkflowRunId", str)
ChannelId = NewType("ChannelId", str)
DataId = NewType("DataId", str)
JobId = NewType("JobId", str)
JobRunId = NewType("JobRunId", str)
JobFnName = NewType("JobFnName", str)
JobHash = NewType("JobHash", str)


@dataclass(frozen=True)
class RunJobArgs:
    """Arguments injected by the orchestrator when running a job."""

    run_id: WorkflowRunId
    job_id: JobId
    data_reference_uri: str
    job_run_id: str


@dataclass(frozen=True)
class JobRunContext:
    """Context for a running workflow job."""

    workflow_run_id: WorkflowRunId
    job_run_id: JobRunId
    input: DataUri


@dataclass(frozen=True)
class GPUConfig:
    type: str | None = None
    count: int | None = None


@dataclass(frozen=True)
class JobResourceConfig:
    cpu_cores: float | None = None
    memory_gb: int | None = None
    gpu: GPUConfig | None = None
