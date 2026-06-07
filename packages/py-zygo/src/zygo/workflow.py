from collections.abc import Callable
from types import FunctionType
from typing import (
    TypeVar,
    final,
    overload,
)

from zygo._internal.meta.jobs import validate_job
from zygo._internal.runtime import execute_job, start_workflow
from zygo._internal.runtime.mode import (
    RunJobMode,
    StartMode,
    parse_workflow_entrypoint_mode,
)
from zygo._internal.utils.caller import caller_module_path
from zygo._internal.utils.hash import hash_to_str
from zygo.backends.protocol import Backend
from zygo.channel import Channel
from zygo.jobs import JobRegistry
from zygo.types import (
    ChannelName,
    Environment,
)

"""
Workflow Python API
Provides a Python API for defining and running workflows.
"""


F = TypeVar("F", bound=FunctionType)


@final
class Workflow:
    def __init__(self, *, name: str) -> None:
        self.name = name
        self.jobs = JobRegistry()
        self.channels: dict[str, Channel] = {}

    @property
    def content_hash(self) -> str:
        job_hashes = [bytes(j.hash, "utf-8") for j in self.jobs.entries()]
        return hash_to_str(job_hashes)

    @overload
    def job(self, func: F) -> F: ...

    @overload
    def job(
        self, func: None = None, *, env: Environment | None = None
    ) -> Callable[[F], F]: ...

    def job(
        self, func: F | None = None, *, env: Environment | None = None
    ) -> F | Callable[[F], F]:
        """
        Decorator to register a job function with the workflow.

        Can be used with or without parameters:
        - @workflow.job
        - @workflow.job()
        - @workflow.job(env=Environment(cpu=1, memory=1024, gpu="A100", gpu_count=1))

        Args:
            func: The function to register (when used without parentheses)
            env: Optional environment configuration for the job
        """

        def decorator(f: F) -> F:
            validate_job(f)
            self.jobs.set(f, env=env)
            return f

        if func is None:
            return decorator

        return decorator(func)

    def channel(self, *, name: str) -> Channel:
        """Creates a channel and registers it with the workflow"""
        channel = Channel(name=ChannelName(name))
        if name in self.channels:
            raise ValueError(f"Channel {name} already exists")
        self.channels[name] = channel
        return channel

    def run(self, *, channel: Channel, uri: str, backend: Backend) -> None:
        """
        Entrypoint for running a workflow script.
        There are two modes:

        1. StartMode.
            - The workflow is deployed
            - Input data is ingested
            - The initial channel event is emitted to the orchestrator

        2. RunJobMode. Only the targeted job function is executed.
            - The targeted job function is executed with events emitted to the orchestrator.
            - This is called by the orchestrator, through the backend's provided entrypoint.
        """
        match parse_workflow_entrypoint_mode():
            case RunJobMode(run_job_args=run_job_args):
                execute_job(workflow=self, run_job_args=run_job_args, backend=backend)

            case StartMode():
                module_path = caller_module_path(stack_offset=1)
                start_workflow(
                    workflow=self,
                    channel=channel,
                    uri=uri,
                    backend=backend,
                    module_path=module_path,
                )
