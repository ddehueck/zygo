from __future__ import annotations

from dataclasses import dataclass
from functools import wraps
from inspect import Parameter, signature
from typing import TYPE_CHECKING, Annotated, cast, get_args, get_origin

from zygo._internal.meta.dependencies import Dependendable
from zygo._internal.meta.errors import DIError

if TYPE_CHECKING:
    from collections.abc import Callable

    from zygo._internal.meta.container import RunContainer


def build_injected_call[R](
    fn: Callable[..., R], *, container: RunContainer
) -> Callable[[], R]:
    """
    Wrap a function to automatically inject all dependencies from the container.

    All parameters must have Depends(...) as their default value. The framework
    injects all arguments - the returned wrapper takes no arguments.

    Args:
        fn: The function to wrap. All parameters must use Depends(...).
        container: The dependency injection container.

    Returns:
        A zero-argument callable that resolves all dependencies and calls fn.

    Raises:
        DIError: If any parameter does not have a Depends(...) default.
    """
    sig = signature(fn)
    params = list(sig.parameters.values())

    by_defaults = analyze_by_default(params)
    by_annotated = analyze_by_annotated(params)

    analysis = merge_analyses(analyses=[by_defaults, by_annotated])

    # Validate at wrap-time that all parameters are injectable
    if analysis.non_dependables:
        params_str = ", ".join(analysis.non_dependables)
        msg = (
            f"All parameters must use Depends(...). "
            f"Non-injectable parameters in {fn.__name__!r}: {params_str}"
        )
        raise DIError(msg)

    # Use the merged analysis directly — it already maps param names to markers
    named_dependables = analysis.dependables

    @wraps(fn)
    def wrapper() -> R:
        kwargs = {
            name: container.resolve(dependable)
            for name, dependable in named_dependables.items()
        }
        return fn(**kwargs)

    return wrapper


@dataclass(frozen=True)
class DependencyAnalysis:
    """Result of analyzing function parameters for dependency injection.

    Attributes:
        dependables: Ordered mapping of param name -> marker for injectable params.
        non_dependables: List of param names that could not be resolved.
    """

    dependables: dict[str, Dependendable]
    non_dependables: list[str]


def merge_analyses(analyses: list[DependencyAnalysis]) -> DependencyAnalysis:
    """Merge multiple dependency analyses into one.

    A parameter is considered dependable if *any* analysis resolved it.
    It is non-dependable only if *no* analysis resolved it.
    Argument ordering is preserved from the first analysis that lists
    each parameter (either as dependable or non-dependable).
    """
    # Collect all dependable names across every analysis
    all_dependable_names: set[str] = set()
    for a in analyses:
        all_dependable_names.update(a.dependables)

    # Build merged result preserving first-seen order across analyses.
    # We iterate analyses in priority order (first has lowest priority,
    # later analyses can upgrade a param from non-dependable to dependable).
    dependables: dict[str, Dependendable] = {}
    seen_non_dependables: list[str] = []

    for a in analyses:
        for name, dep in a.dependables.items():
            if name not in dependables:
                dependables[name] = dep

    # A param is truly non-dependable only if no analysis found it dependable
    seen: set[str] = set()
    for a in analyses:
        for name in a.non_dependables:
            if name not in all_dependable_names and name not in seen:
                seen_non_dependables.append(name)
                seen.add(name)

    return DependencyAnalysis(
        dependables=dependables, non_dependables=seen_non_dependables
    )


def analyze_by_default(params: list[Parameter]) -> DependencyAnalysis:
    """Analyze parameters that use default-value style: `x: Store = Depends(Store)`."""
    dependables: dict[str, Dependendable] = {}
    non_dependables: list[str] = []

    for param in params:
        param_default = param.default  # pyright: ignore[reportAny]
        if param_default is not Parameter.empty and isinstance(
            param_default, Dependendable
        ):
            dependables[param.name] = param_default
        else:
            non_dependables.append(param.name)

    return DependencyAnalysis(dependables=dependables, non_dependables=non_dependables)


def analyze_by_annotated(params: list[Parameter]) -> DependencyAnalysis:
    """
    Analyze parameters that use Annotated style: `x: Annotated[Store, Depends(Store)]`.
    Returns a DependencyAnalysis summarizing dependables and non-dependables.
    """
    dependables: dict[str, Dependendable] = {}
    non_dependables: list[str] = []

    for param in params:
        annotation = cast("type", param.annotation)
        if annotation is Parameter.empty or get_origin(annotation) is not Annotated:
            non_dependables.append(param.name)
            continue

        args = get_args(annotation)
        if not args:
            non_dependables.append(param.name)
            continue

        # First arg is the type, rest are metadata (Depends, Input, Output, etc.)
        metadata = args[1:] if len(args) > 1 else ()
        dep = next(
            (
                m
                for m in metadata  # pyright: ignore[reportAny]
                if isinstance(m, Dependendable)
            ),
            None,
        )
        if dep is not None:
            dependables[param.name] = dep
        else:
            non_dependables.append(param.name)

    return DependencyAnalysis(dependables=dependables, non_dependables=non_dependables)
