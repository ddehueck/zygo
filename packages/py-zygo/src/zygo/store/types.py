"""Type definitions for the Store abstraction."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from zygo._internal.python.fsspec import FsspecUri

Scope = Literal["job", "workflow", "global"]


@dataclass(frozen=True)
class StoreOptions:
    """Configuration for the store backend."""

    root_uri: FsspecUri
    kwargs: dict[str, str | int | float | bool | None] | None = None  # Jsonable?


@dataclass(frozen=True)
class Reference:
    """A stable reference to a data object stored in the Store."""

    key: str
    scope: Scope
    uri: FsspecUri
    etag: str
    size: int | None = None
    content_type: str | None = None
