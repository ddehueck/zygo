"""Type definitions for the Store abstraction."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from zygo.store.fsspec import FsspecUri as DataUri

__all__ = ["DataUri", "Scope", "StoreOptions"]

# Scopes represent the data visibility for different run contexts.
# - job:      data visible only within the current job
# - workflow: data visible across all jobs in the current workflow run
# - cache:    data visible across all workflow runs
# By default, the scope is "job". This way data is isolated by default.
Scope = Literal["job", "workflow", "cache"]


@dataclass(frozen=True)
class StoreOptions:
    """Configuration for the store backend."""

    root_uri: DataUri
    kwargs: dict[str, str | int | float | bool | None] | None = None  # Jsonable?
