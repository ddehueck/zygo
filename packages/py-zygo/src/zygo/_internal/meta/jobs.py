import inspect
from types import FunctionType
from typing import (
    Annotated,
    TypedDict,
    cast,
    get_args,
    get_origin,
)

from zygo._internal.fn_hash import local_source_dependency_hash
from zygo._internal.meta.dependencies import (
    Dependendable,
    InputMarker,
    OutputMarker,
)
from zygo.types import ChannelId


class JobDefinition(TypedDict):
    name: str
    hash: str
    input_channel_id: ChannelId
    output_channel_ids: list[ChannelId]


class JobParameterIds(TypedDict):
    input_channel_id: ChannelId
    output_channel_ids: list[ChannelId]


# TODO: Really good error handling here
def validate_job(func: FunctionType) -> None:
    """Validate that all job parameters are injectable via the meta system."""
    signature = inspect.signature(func)
    for param in signature.parameters.values():
        default = param.default  # pyright: ignore[reportAny]
        if default is inspect.Parameter.empty:
            continue
        if not isinstance(default, Dependendable):
            raise ValueError(
                f"Parameter '{param.name}' must use Depends(), Input(), or Output()"
            )


def build_job_definition(job: FunctionType) -> JobDefinition:
    """Build CLI-inspectable metadata for a workflow job."""
    parameters = _get_job_parameter_ids(job)
    return JobDefinition(
        name=job.__name__,
        hash=local_source_dependency_hash(job).hash_str,
        input_channel_id=parameters["input_channel_id"],
        output_channel_ids=parameters["output_channel_ids"],
    )


def _get_job_parameter_ids(job: FunctionType) -> JobParameterIds:
    signature = inspect.signature(job)
    input_channel_id: ChannelId | None = None
    output_channel_ids: list[ChannelId] = []
    for param in signature.parameters.values():
        for marker in _get_markers(param):
            if isinstance(marker, InputMarker):
                input_channel_id = marker.channel.id
            elif isinstance(marker, OutputMarker):
                output_channel_ids.append(marker.channel.id)

    if input_channel_id is None:
        raise ValueError(f"Job {job.__name__} has no input channel")

    return JobParameterIds(
        input_channel_id=input_channel_id,
        output_channel_ids=output_channel_ids,
    )


# TODO: Reconcile with build_injected_call - should have shared functionality
def _get_markers(param: inspect.Parameter) -> list[Dependendable]:
    """Extract dependency markers from both default-value and Annotated styles."""
    markers: list[Dependendable] = []

    # Check default-value style: x: Store = Depends(Store)
    default = param.default  # pyright: ignore[reportAny]
    if default is not inspect.Parameter.empty and isinstance(default, Dependendable):
        markers.append(default)

    # Check Annotated style: x: Annotated[Store, Depends(Store)]
    annotation = cast("type", param.annotation)
    if (
        annotation is not inspect.Parameter.empty
        and get_origin(annotation) is Annotated
    ):
        args = get_args(annotation)
        markers.extend(
            arg
            for arg in args[1:]  # pyright: ignore[reportAny]
            if isinstance(arg, Dependendable)
        )

    return markers
