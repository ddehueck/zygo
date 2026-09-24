from __future__ import annotations

import re
from typing import TYPE_CHECKING, cast

import fsspec  # type: ignore

if TYPE_CHECKING:
    from fsspec.spec import AbstractFileSystem  # type: ignore

    from zygo.store._internal.types import PartitionKey
    from zygo.store.types import StoreOptions


def _partition(partition_key: PartitionKey, value: str) -> str:
    return f"{partition_key}={value}"


def _contains_any_partition_key(key: str, partition_keys: list[PartitionKey]) -> bool:
    return any(f"{pk}=" in key for pk in partition_keys)


def _normalize_key(key: str) -> str:
    # todo: log if key was modified
    # This regex replaces any character that is not alphanumeric, underscore, hyphen, or period with an underscore.
    # Fix: don't allow a dash/hyphen at the first or last position, don't allow repeated underscores or dots.
    key = re.sub(r"[^\w\.-]", "_", key)
    key = re.sub(r"_+", "_", key)  # Replace multiple underscores with one
    key = re.sub(r"\.+", ".", key)  # Replace multiple dots with one
    return key.strip("-.")


def _build_fs(options: StoreOptions) -> AbstractFileSystem:
    extra = options.kwargs or {}
    fs = fsspec.filesystem(options.root_uri.protocol or "file", **extra)  # type: ignore
    return cast("AbstractFileSystem", fs)
