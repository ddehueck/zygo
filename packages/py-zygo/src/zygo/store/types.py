"""Type definitions for the Store abstraction."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from zygo._internal.fsspec import FsspecUri

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
    uri: FsspecUri

    def to_dict(self) -> dict[str, str]:
        return {
            "key": self.key,
            "uri": str(self.uri),
        }

    @staticmethod
    def from_dict(data: dict[str, str]) -> Reference:
        if "uri" not in data:
            raise TypeError(f"Expected str, got {type(data['uri'])}")
        if "key" not in data:
            raise TypeError(f"Expected str, got {type(data['key'])}")

        return Reference(key=data["key"], uri=FsspecUri(data["uri"]))
