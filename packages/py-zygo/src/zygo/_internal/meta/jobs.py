import inspect
from types import FunctionType
from typing import (
    Annotated,
    TypedDict,
    cast,
    get_args,
    get_origin,
    get_type_hints,
)

from zygo._internal.fn_hash import local_source_dependency_hash
from zygo._internal.meta.dependencies import (
    Dependendable,
    InputMarker,
    OutputMarker,
)
from zygo.context import JobContext
from zygo.types import ChannelId


class JobDefinition(TypedDict):
    name: str
    hash: str
    input_channel_id: ChannelId
    output_channel_ids: list[ChannelId]


class JobParameterIds(TypedDict):
    input_channel_id: ChannelId
    output_channel_ids: list[ChannelId]


CONTEXTUAL_JOB_PARAMETER_COUNT = 2


# TODO: Really good error handling here that any programmer can understand
def validate_job(
    func: FunctionType,
    *,
    input_type: type[object],
    output_type: type[object],
) -> None:
    """Validate a given job function.

    Requirements:
        - The function must accept exactly one input parameter.
        - The input parameter must be positional.
        - The function must not have a default value for the input parameter.
        - If using another arg, it must be a keyword-only 'ctx: JobContext' parameter.
        - Does runtime validation of input channel and output channel types
    """
    parameters = tuple(inspect.signature(func).parameters.values())

    if len(parameters) not in {1, 2}:
        message = f"Job {func.__name__!r} must accept exactly one input parameter and may optionally accept a keyword-only 'ctx: JobContext' parameter"
        raise ValueError(message)

    input_parameter = parameters[0]
    if input_parameter.kind not in {
        inspect.Parameter.POSITIONAL_ONLY,
        inspect.Parameter.POSITIONAL_OR_KEYWORD,
    }:
        message = f"Job {func.__name__!r} input parameter {input_parameter.name!r} must be positional"
        raise ValueError(message)
    input_default = cast("object", input_parameter.default)
    if input_default is not inspect.Parameter.empty:
        message = f"Job {func.__name__!r} input parameter {input_parameter.name!r} must not have a default value"
        raise ValueError(message)

    type_hints = _resolve_type_hints(func)
    _validate_channel_types(
        func,
        input_parameter=input_parameter,
        type_hints=type_hints,
        input_type=input_type,
        output_type=output_type,
    )

    if len(parameters) == CONTEXTUAL_JOB_PARAMETER_COUNT:
        _validate_context_parameter(
            func,
            context_parameter=parameters[1],
            type_hints=type_hints,
        )


def _resolve_type_hints(func: FunctionType) -> dict[str, object]:
    try:
        return cast("dict[str, object]", get_type_hints(func))
    except (NameError, TypeError) as error:
        raise ValueError(
            f"Could not resolve annotations for job {func.__name__!r}: {error}"
        ) from error


def _validate_channel_types(
    func: FunctionType,
    *,
    input_parameter: inspect.Parameter,
    type_hints: dict[str, object],
    input_type: type[object],
    output_type: type[object],
) -> None:
    input_annotation = type_hints.get(input_parameter.name, inspect.Parameter.empty)
    if input_annotation != input_type:
        message = f"Job {func.__name__!r} input parameter {input_parameter.name!r} must be annotated with the input channel type {input_type!r}, not {input_annotation!r}"
        raise ValueError(message)

    return_annotation = type_hints.get("return", inspect.Signature.empty)
    is_optional_output = return_annotation == output_type | None
    if return_annotation != output_type and not is_optional_output:
        message = f"Job {func.__name__!r} return value must be annotated with the output channel type {output_type!r} or {output_type!r} | None, not {return_annotation!r}"
        raise ValueError(message)


def _validate_context_parameter(
    func: FunctionType,
    *,
    context_parameter: inspect.Parameter,
    type_hints: dict[str, object],
) -> None:
    if (
        context_parameter.name != "ctx"
        or context_parameter.kind is not inspect.Parameter.KEYWORD_ONLY
    ):
        message = f"Job {func.__name__!r} context parameter must be declared as '*, ctx: JobContext'"
        raise ValueError(message)

    context_default = cast("object", context_parameter.default)
    if context_default is not inspect.Parameter.empty:
        message = f"Job {func.__name__!r} context parameter 'ctx' must not have a default value"
        raise ValueError(message)

    if type_hints.get("ctx", inspect.Parameter.empty) is not JobContext:
        raise ValueError(
            f"Job {func.__name__!r} context parameter must be annotated as JobContext e.g. 'my_func(my_input: InputType, *, ctx: JobContext)'"
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
