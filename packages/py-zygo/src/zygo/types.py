from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, NewType

if TYPE_CHECKING:
    from zygo._internal import grpc
    from zygo.store import Reference

WorkflowId = NewType("WorkflowId", str)
WorkflowVersionId = NewType("WorkflowVersionId", str)

JobId = NewType("JobId", str)
JobFnName = NewType("JobFnName", str)
JobName = NewType("JobName", str)
JobHash = NewType("JobHash", str)
RunId = NewType("RunId", str)


ChannelName = NewType("ChannelName", str)
ChannelId = NewType("ChannelId", str)
DataId = NewType("DataId", str)


@dataclass(frozen=True)
class RunJobArgs:
    """Arguments for running a job and injected by the orchestrator at runtime."""

    run_id: RunId
    workflow_version_id: WorkflowVersionId
    job_fn_name: JobFnName
    job_id: JobId
    data_reference_uri: str
    data_reference_etag: str
    channel_ids_by_name: dict[ChannelName, ChannelId]
    job_run_id: str


@dataclass(frozen=True)
class JobRunContext:
    """Context for a running workflow job."""

    workflow_run_id: str
    job_run_id: str
    data_ref: Reference


@dataclass(frozen=True)
class RunEventContext:
    """Common envelope for every event emitted during a workflow run."""

    workflow_version_id: str
    workflow_run_id: str
    source: grpc.EventSource  # hmmm


@dataclass(frozen=True)
class GPUConfig:
    type: str | None = None
    count: int | None = None


@dataclass(frozen=True)
class JobResourceConfig:
    cpu_cores: float | None = None
    memory_gb: int | None = None
    gpu: GPUConfig | None = None


@dataclass(frozen=True)
class Environment:
    """
    This is a superset of all the common ways to specify a job environment.
    A backend can/should choose to support only a subset of this.

    TODO: More spec file support. e.g. requirements.txt, poetry.lock, environment.yaml, dockerfile, lockfiles, list of packages, etc.
    """

    python_version: str
    """Required Python version, e.g. "3.12".  Backends use this to pin the
    interpreter; when ``None`` the backend falls back to its own default."""

    cpu: float | None = None
    """ The number of CPU cores to request. """
    memory: int | None = None
    """ The amount of memory to request in megabytes. """
    gpu: str | None = None
    """ The type of GPU to use. e.g. "A100" """
    gpu_count: int | None = None
    """ The number of GPUs to use. """

    uv_lock: str | None = None
    """The path to the uv.lock file."""
    packages: list[str] | None = None
    """The packages to install. e.g. ["numpy", "pandas==2.2.0"]"""

    config: dict[str, str] | None = None
    """Any additional configuration for the job. Useful to set custom arguments to a backend."""

    # TODO: Better validation.
    def __post_init__(self) -> None:
        if self.cpu is not None and self.cpu < 0:
            raise ValueError("CPU cores must be positive")
        if self.memory is not None and self.memory < 0:
            raise ValueError("Memory must be positive")

        if self.gpu_count:
            if self.gpu is None:
                raise ValueError("GPU type must be specified if GPU count is specified")
            if self.gpu_count < 0:
                raise ValueError("GPU count must be positive")
