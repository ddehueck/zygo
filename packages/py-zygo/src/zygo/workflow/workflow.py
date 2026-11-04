from collections.abc import Callable
import inspect
from typing import (
    TYPE_CHECKING,
    Any,
    ClassVar,
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
from zygo.workflow.channel import Channel
from zygo.workflow.codecs import Csv, Txt
from zygo.workflow.context import JobContext
from zygo.workflow.jobs import JobRegistry
from zygo.workflow.types import WorkflowId

T_workflow_in = TypeVar("T_workflow_in")
T_workflow_out = TypeVar("T_workflow_out")

T_job_in = TypeVar("T_job_in")
T_job_out = TypeVar("T_job_out")
T_job_in_contra = TypeVar("T_job_in_contra", contravariant=True)
T_job_out_co = TypeVar("T_job_out_co", covariant=True)
T_file_job_out = TypeVar("T_file_job_out")


class _JobWithContext(Protocol[T_job_in_contra, T_job_out_co]):
    def __call__(
        self,
        value: T_job_in_contra,
        /,
        *,
        ctx: JobContext,
    ) -> T_job_out_co: ...


type _JobReturn[T] = T | list[T] | None


class _JobDecorator(Protocol[T_job_in, T_job_out]):
    @overload
    def __call__(
        self,
        fn: _JobWithContext[T_job_in, _JobReturn[T_job_out]],
    ) -> _JobWithContext[T_job_in, _JobReturn[T_job_out]]: ...

    @overload
    def __call__(
        self,
        fn: Callable[[T_job_in], _JobReturn[T_job_out]],
    ) -> Callable[[T_job_in], _JobReturn[T_job_out]]: ...


class _FileJobDecorator(Protocol[T_job_in]):
    """Job decorator for codecs that can serialize multiple Python types."""

    @overload
    def __call__(
        self,
        fn: _JobWithContext[T_job_in, T_file_job_out],
    ) -> _JobWithContext[T_job_in, T_file_job_out]: ...

    @overload
    def __call__(
        self,
        fn: Callable[[T_job_in], T_file_job_out],
    ) -> Callable[[T_job_in], T_file_job_out]: ...


@final
class Workflow:
    """
    The Zygo Python API for defining and running workflows.
    """

    conventional_names: ClassVar[tuple[str, ...]] = ("workflow", "wf")

    def __init__(
        self,
        *,
        id: str,
        input: Channel[T_workflow_in],
        output: Channel[T_workflow_out],
    ) -> None:
        frame = inspect.currentframe()
        caller = frame.f_back if frame is not None else None
        module_name = caller.f_globals.get("__name__") if caller else None
        module_file = caller.f_globals.get("__file__") if caller else None
        self._definition_module_name = (
            module_name if isinstance(module_name, str) else None
        )
        self._definition_file = module_file if isinstance(module_file, str) else None
        del caller, frame

        self.id = WorkflowId(id)
        self.input_channel = input
        self.output_channel = output
        self.jobs = JobRegistry()

    @property
    def content_hash(self) -> str:
        job_hashes = [bytes(entry.hash, "utf-8") for entry in self.jobs]
        return hash_to_str(job_hashes)

    @overload
    def job(
        self,
        *,
        input: Channel[T_job_in],
        output: Channel[Csv],
    ) -> _FileJobDecorator[T_job_in]: ...

    @overload
    def job(
        self,
        *,
        input: Channel[T_job_in],
        output: Channel[Txt],
    ) -> _FileJobDecorator[T_job_in]: ...

    @overload
    def job(
        self,
        *,
        input: Channel[T_job_in],
        output: Channel[T_job_out],
    ) -> _JobDecorator[T_job_in, T_job_out]: ...

    def job(
        self,
        *,
        input: Channel[T_job_in],
        output: Channel[Any],  # pyright: ignore[reportExplicitAny]
    ) -> _FileJobDecorator[T_job_in] | _JobDecorator[
        T_job_in, Any
    ]:  # pyright: ignore[reportExplicitAny]
        """
        Decorator to register a job function with the workflow.

        A job must have an input and output channel and a unique ID (derived from the function name).

        The function's input type must match the type of the input channel.
        The function's return type must be:
            - the output channel type
            - a list of the output channel type `list[T_job_out]` (useful for fan-out)
            - any union of the above and `None`, such as `T_job_out | list[T_job_out] | None`

        Example:
            @workflow.job(input=channel, output=channel)
            def my_job(value: int, *, ctx: JobContext) -> int:
                return value * 2

        Args:
            input: The input channel for the job
            output: The output channel for the job
        """

        def decorator(
            fn: Callable[[T_job_in], object]
            | _JobWithContext[T_job_in, object],
        ) -> (
            Callable[[T_job_in], object]
            | _JobWithContext[T_job_in, object]
        ):
            job_fn = cast("FunctionType", fn)
            validate_job(
                job_fn,
                input=input,
                output=output,
            )
            self.jobs.set(
                job=job_fn,
                input_channel=input,
                output_channel=output,
            )
            return job_fn

        return cast(
            "_FileJobDecorator[T_job_in] | _JobDecorator[T_job_in, Any]",
            decorator,
        )  # pyright: ignore[reportExplicitAny]
