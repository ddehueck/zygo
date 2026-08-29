import inspect
from types import FunctionType, NoneType, UnionType
from typing import (
    Union,
    cast,
    get_args,
    get_origin,
    get_type_hints,
)

from zygo.context import JobContext

CONTEXTUAL_JOB_PARAMETER_COUNT = 2


# TODO: Really good error handling here that any programmer can understand
def validate_job(
    func: FunctionType,
    *,
    input_channel_type: type[object],
    output_channel_type: type[object],
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
        input_type=input_channel_type,
        output_type=output_channel_type,
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
    if not _is_valid_return_annotation(return_annotation, output_type=output_type):
        message = f"Job {func.__name__!r} return value must be annotated with the output channel type {output_type!r}, list[{output_type!r}], or a union of those types and None, not {return_annotation!r}"
        raise ValueError(message)


def _is_valid_return_annotation(
    annotation: object,
    *,
    output_type: type[object],
) -> bool:
    if _is_output_annotation(annotation, output_type=output_type):
        return True

    if get_origin(annotation) not in {Union, UnionType}:
        return False

    union_members = cast("tuple[object, ...]", get_args(annotation))
    return any(member is not NoneType for member in union_members) and all(
        member is NoneType
        or _is_output_annotation(member, output_type=output_type)
        for member in union_members
    )


def _is_output_annotation(
    annotation: object,
    *,
    output_type: type[object],
) -> bool:
    if annotation == output_type:
        return True

    return get_origin(annotation) is list and get_args(annotation) == (output_type,)


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
