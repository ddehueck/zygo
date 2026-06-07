import inspect
from types import FunctionType
from typing import (
    Annotated,
    TypedDict,
    cast,
    get_args,
    get_origin,
)

from zygo._internal.meta.dependencies import (
    Dependendable,
    InputMarker,
    OutputMarker,
)
from zygo._internal.python.fn_hash import local_source_dependency_hash
from zygo.types import (
    ChannelName,
)


class RegisterJobArgs(TypedDict):
    name: str
    hash: str
    input_channel: ChannelName
    output_channels: list[ChannelName]


class JobParameterArgs(TypedDict):
    input_channel: ChannelName
    output_channels: list[ChannelName]


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


def build_register_job_args(job: FunctionType) -> RegisterJobArgs:
    name = job.__name__
    job_id = local_source_dependency_hash(job)

    parameters = _get_job_parameters(job)
    return RegisterJobArgs(
        name=name,
        hash=job_id.hash_str,
        input_channel=parameters["input_channel"],
        output_channels=parameters["output_channels"],
    )


def _get_job_parameters(job: FunctionType) -> JobParameterArgs:
    signature = inspect.signature(job)
    input_channel: ChannelName | None = None
    output_channels: list[ChannelName] = []
    for param in signature.parameters.values():
        for marker in _get_markers(param):
            if isinstance(marker, InputMarker):
                input_channel = ChannelName(marker.channel.name)
            elif isinstance(marker, OutputMarker):
                output_channels.append(ChannelName(marker.channel.name))

    if input_channel is None:
        raise ValueError(f"Job {job.__name__} has no input channel")

    return JobParameterArgs(
        input_channel=input_channel,
        output_channels=output_channels,
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
