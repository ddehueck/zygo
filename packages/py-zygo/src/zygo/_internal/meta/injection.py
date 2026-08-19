from __future__ import annotations

from functools import wraps
from inspect import signature
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

    from zygo.context import JobContext


# TODO: Use the same function signature as the workflow.job decorator for consistency.
# A job context is injected only when the function declares a ctx parameter.
def build_injected_job_fn[R](
    fn: Callable[..., R],
    *,
    input_data: object,
    ctx: JobContext,
) -> Callable[[], R]:
    accepts_context = "ctx" in signature(fn).parameters

    @wraps(fn)
    def wrapper() -> R:
        if accepts_context:
            return fn(input_data, ctx=ctx)
        return fn(input_data)

    return wrapper
