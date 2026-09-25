from __future__ import annotations

import re
from typing import (
    IO,
    TYPE_CHECKING,
    BinaryIO,
    Literal,
    Protocol,
    TextIO,
    cast,
    overload,
)

import fsspec  # type: ignore

if TYPE_CHECKING:
    from zygo.store._internal.types import PartitionKey
    from zygo.store.types import DataUri

__all__ = [
    "StoreFileSystem",
    "build_fs",
    "contains_any_partition_key",
    "normalize_key",
    "partition",
]


def partition(partition_key: PartitionKey, value: str) -> str:
    return f"{partition_key}={value}"


def contains_any_partition_key(key: str, partition_keys: list[PartitionKey]) -> bool:
    return any(f"{pk}=" in key for pk in partition_keys)


def normalize_key(key: str) -> str:
    # todo: log if key was modified
    # This regex replaces any character that is not alphanumeric, underscore, hyphen, or period with an underscore.
    # Fix: don't allow a dash/hyphen at the first or last position, don't allow repeated underscores or dots.
    key = re.sub(r"[^\w\.-]", "_", key)
    key = re.sub(r"_+", "_", key)  # Replace multiple underscores with one
    key = re.sub(r"\.+", ".", key)  # Replace multiple dots with one
    return key.strip("-.")


class StoreFileSystem(Protocol):
    @overload
    def open(self, path: str, mode: Literal["wb", "rb"]) -> BinaryIO: ...

    @overload
    def open(self, path: str, mode: str) -> IO[bytes] | TextIO: ...

    def exists(self, path: str) -> bool: ...

    def rm(self, path: str) -> None: ...

    def makedirs(self, path: str, *, exist_ok: bool = False) -> None: ...


def build_fs(
    root: DataUri,
    kwargs: dict[str, str | int | float | bool | None] | None = None,
) -> StoreFileSystem:
    fs = fsspec.filesystem(root.protocol, **(kwargs or {}))  # type: ignore
    return cast("StoreFileSystem", fs)
