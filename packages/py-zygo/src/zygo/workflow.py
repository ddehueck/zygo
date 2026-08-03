from collections.abc import Callable
from types import FunctionType
from typing import (
    TypeVar,
    final,
    overload,
)

from zygo._internal.meta.jobs import validate_job
from zygo._internal.utils.hash import hash_to_str
from zygo.channel import Channel
from zygo.jobs import JobRegistry
from zygo.types import (
    ChannelId,
    JobId,
    WorkflowId,
)

F = TypeVar("F", bound=FunctionType)


@final
class Workflow:
    """
    The Zygo Python API for defining and running workflows.
    """

    def __init__(self, *, id: str) -> None:
        self.id = WorkflowId(id)
        self.jobs = JobRegistry()
        self.channels: dict[ChannelId, Channel] = {}

    @property
    def content_hash(self) -> str:
        job_hashes = [bytes(j.hash, "utf-8") for j in self.jobs.entries()]
        return hash_to_str(job_hashes)

    @overload
    def job(self, func: F) -> F: ...

    @overload
    def job(self, func: None = None, *, id: str | None = None) -> Callable[[F], F]: ...

    def job(
        self, func: F | None = None, *, id: str | None = None
    ) -> F | Callable[[F], F]:
        """
        Decorator to register a job function with the workflow.

        Can be used with or without parameters:
        - @workflow.job
        - @workflow.job()
        - @workflow.job(id="my_job")

        Args:
            func: The function to register (when used without parentheses)
            env: Optional environment configuration for the job
        """

        def decorator(f: F) -> F:
            validate_job(f)
            self.jobs.set(f, id=JobId(id) if id else None)
            return f

        if func is None:
            return decorator

        return decorator(func)

    def channel(self, *, id: str) -> Channel:
        """Create a channel and register it with the workflow."""
        channel_id = ChannelId(id)
        if channel_id in self.channels:
            raise ValueError(f"Channel {id} already exists")

        channel = Channel(id=channel_id)
        self.channels[channel_id] = channel
        return channel
