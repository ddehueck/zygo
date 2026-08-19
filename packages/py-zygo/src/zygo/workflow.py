from collections.abc import Callable
from typing import (
    TYPE_CHECKING,
    Protocol,
    TypeVar,
    cast,
    final,
    overload,
)

if TYPE_CHECKING:
    from types import FunctionType

from zygo._internal.meta.jobs import validate_job
from zygo._internal.utils.hash import hash_to_str
from zygo.channel import Channel
from zygo.context import JobContext
from zygo.jobs import JobRegistry
from zygo.types import WorkflowId

T_workflow_in = TypeVar("T_workflow_in")
T_workflow_out = TypeVar("T_workflow_out")

T_job_in = TypeVar("T_job_in")
T_job_out = TypeVar("T_job_out")
T_job_in_contra = TypeVar("T_job_in_contra", contravariant=True)
T_job_out_co = TypeVar("T_job_out_co", covariant=True)


class _JobWithContext(Protocol[T_job_in_contra, T_job_out_co]):
    def __call__(
        self,
        value: T_job_in_contra,
        *,
        ctx: JobContext,
    ) -> T_job_out_co: ...


class _JobDecorator(Protocol[T_job_in, T_job_out]):
    @overload
    def __call__(
        self,
        fn: _JobWithContext[T_job_in, T_job_out | None],
    ) -> _JobWithContext[T_job_in, T_job_out | None]: ...

    @overload
    def __call__(
        self,
        fn: Callable[[T_job_in], T_job_out | None],
    ) -> Callable[[T_job_in], T_job_out | None]: ...


@final
class Workflow:
    """
    The Zygo Python API for defining and running workflows.
    """

    def __init__(
        self,
        *,
        id: str,
        input: Channel[T_workflow_in],
        output: Channel[T_workflow_out],
    ) -> None:
        self.id = WorkflowId(id)
        self.input_channel = input
        self.output_channel = output
        self.jobs = JobRegistry()

    @property
    def content_hash(self) -> str:
        job_hashes = [bytes(j.hash, "utf-8") for j in self.jobs.entries()]
        return hash_to_str(job_hashes)

    def job(
        self,
        *,
        input: Channel[T_job_in],
        output: Channel[T_job_out],
    ) -> _JobDecorator[T_job_in, T_job_out]:
        """
        Decorator to register a job function with the workflow.

        A job must have an input and output channel and a unique ID (derived from the function name).

        The function's input type must match the type of the input channel.
        The function's return type must be the output channel type or that type unioned with `None`.

        Example:
            @workflow.job(input=channel, output=channel)
            def my_job(value: int, *, ctx: JobContext) -> int:
                return value * 2

        Args:
            input: The input channel for the job
            output: The output channel for the job
        """

        def decorator(
            fn: Callable[[T_job_in], T_job_out | None]
            | _JobWithContext[T_job_in, T_job_out | None],
        ) -> (
            Callable[[T_job_in], T_job_out | None]
            | _JobWithContext[T_job_in, T_job_out | None]
        ):
            job_fn = cast("FunctionType", fn)
            validate_job(
                job_fn,
                input_type=input.value_type,
                output_type=output.value_type,
            )
            self.jobs.set(job_fn)
            return job_fn

        return cast("_JobDecorator[T_job_in, T_job_out]", decorator)
