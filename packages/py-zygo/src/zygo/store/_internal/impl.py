"""
This store is built on top of fsspec to enable local and remote data storage access.
The store is a key-value store that can be used to store and retrieve data.

Users bring their own protocol and root directory.
Meanwhile, the store provides data isolation and versioning relative to the orchestration requirements.

This way, task data is easily isolated to avoid data contamination between tasks by default.
A user can still opt-in to a shared store across a workflow run via the `scope` parameter.
"""

from __future__ import annotations

import posixpath
import re
from typing import TYPE_CHECKING, BinaryIO, Literal, TextIO, cast, overload, override

import fsspec  # type: ignore

from zygo._internal.python.fsspec import FsspecUri
from zygo.store import Reference, StoreProtocol

if TYPE_CHECKING:
    from contextlib import AbstractContextManager

    from fsspec.spec import AbstractFileSystem  # type: ignore

    from zygo.store._internal.types import PartitionKey
    from zygo.store.types import Scope, StoreOptions
    from zygo.types import JobRunContext


def _partition(partition_key: PartitionKey, value: str) -> str:
    return f"{partition_key}={value}"


def _normalize_key(key: str) -> str:
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


def _file_metadata(fs: AbstractFileSystem, path: str) -> tuple[str, int | None]:
    """Return ``(etag, size)`` for *path*, falling back to safe defaults."""
    info = fs.info(path)  # type: ignore
    info_dict = (
        cast("dict[str, str | int | None]", info) if isinstance(info, dict) else {}
    )
    etag = info_dict.get("etag")
    size = info_dict.get("size")
    return (str(etag) if etag else "dummy", int(size) if size else None)


class StoreImpl(StoreProtocol):
    """
    A high-level store built on fsspec.
    """

    def __init__(self, *, context: JobRunContext, options: StoreOptions) -> None:
        super().__init__()
        self._context = context
        self._options = options
        self._fs = _build_fs(options)

    def _prefix(self, scope: Scope) -> str:
        """
        Map scope -> a path prefix under the user-provided root.
        """

        # Keep paths POSIX-like even on Windows since many fsspec backends expect that.
        base = posixpath.join(self._options.root_uri.path)

        if scope == "job":
            return posixpath.join(
                base,
                _partition("workflow_run_id", self._context.workflow_run_id),
                _partition("job_run_id", self._context.job_run_id),
            )

        if scope == "workflow":
            return posixpath.join(
                base,
                _partition("workflow_run_id", self._context.workflow_run_id),
                "shared",
            )

        # "global" = shared across runs (still under root, but outside run namespace)
        return posixpath.join(self._options.root_uri.path, "store", "global")

    def _uri_for_key(self, key: str, scope: Scope) -> str:
        key = _normalize_key(key)
        print(f"key: {key}")  # noqa: T201
        prefix = self._prefix(scope)
        print(f"prefix: {prefix}")  # noqa: T201
        return posixpath.join(prefix, key)

    @override
    def put(
        self,
        key: str,
        data: bytes,
        *,
        scope: Scope = "job",
        content_type: str | None = None,
    ) -> Reference:
        uri = self._uri_for_key(key, scope)
        print(f"uri: {uri}")  # noqa: T201

        # Ensure parent directories for local-ish FS that require it
        parent = posixpath.dirname(uri)
        if self._options.root_uri.is_local():
            self._fs.makedirs(parent, exist_ok=True)  # type: ignore

        with self._fs.open(uri, "wb") as f:  # type: ignore
            f.write(data)  # type: ignore

        etag, size = _file_metadata(self._fs, uri)

        return Reference(
            key=key,
            scope=scope,
            uri=FsspecUri(uri),
            etag=etag,
            size=size,
            content_type=content_type,
        )

    @override
    def get(self, key: str | Reference, *, scope: Scope = "job") -> bytes:
        uri_raw = (
            key.uri if isinstance(key, Reference) else self._uri_for_key(key, scope)
        )
        uri_str = str(uri_raw) if not isinstance(uri_raw, str) else uri_raw
        with self._fs.open(uri_str, "rb") as f:  # type: ignore
            return f.read()  # type: ignore

    @override
    def exists(self, key: str, *, scope: Scope = "job") -> bool:
        uri = self._uri_for_key(key, scope)
        return self._fs.exists(uri)  # type: ignore

    @override
    def delete(self, key: str, *, scope: Scope = "job") -> None:
        uri = self._uri_for_key(key, scope)
        if self._fs.exists(uri):  # type: ignore
            self._fs.rm(uri)  # type: ignore

    @overload
    def open(
        self,
        ref: str | Reference,
        mode: Literal["r", "w", "a", "x", "rt", "wt", "at", "xt"] = ...,
        *,
        scope: Scope = ...,
    ) -> AbstractContextManager[TextIO]: ...

    @overload
    def open(
        self,
        ref: str | Reference,
        mode: Literal["rb", "wb", "ab", "xb"],
        *,
        scope: Scope = ...,
    ) -> AbstractContextManager[BinaryIO]: ...

    @overload
    def open(
        self,
        ref: str | Reference,
        mode: str,
        *,
        scope: Scope = ...,
    ) -> AbstractContextManager[TextIO | BinaryIO]: ...

    @override
    def open(
        self,
        ref: str | Reference,
        mode: str = "r",
        *,
        scope: Scope = "job",
    ) -> AbstractContextManager[TextIO | BinaryIO]:
        uri_raw = (
            ref.uri if isinstance(ref, Reference) else self._uri_for_key(ref, scope)
        )
        uri_str = str(uri_raw) if not isinstance(uri_raw, str) else uri_raw

        # Ensure parent directories exist for write/append modes on local FS
        if any(c in mode for c in "wa"):
            parent = posixpath.dirname(uri_str)
            if self._options.root_uri.is_local():
                self._fs.makedirs(parent, exist_ok=True)  # type: ignore

        return self._fs.open(uri_str, mode)  # type: ignore


_COPY_CHUNK = 8 * 1024 * 1024  # 8 MiB


def ingest(*, data_uri: FsspecUri, store_options: StoreOptions) -> Reference:
    """
    Ingest local data into the store's global scope and return its Reference.

    Typically used at workflow trigger time to import local input data into the
    backend store prior to any job execution.

    Args:
        data_uri: Local fsspec-compatible URI, such as ``file://./data.csv`` or
            ``memory://input.bin``.
        store_options: Target store configuration.
    """
    if not data_uri.is_local():
        raise ValueError("Local input URI is required")

    input_fs: AbstractFileSystem = fsspec.filesystem(data_uri.protocol or "file")  # type: ignore[arg-type]
    if not input_fs.exists(data_uri.path):  # type: ignore[arg-type]
        raise FileNotFoundError(f"Input URI does not exist: {data_uri.uri}")

    store_fs = _build_fs(store_options)

    key = data_uri.key
    dest = posixpath.join(store_options.root_uri.path, key)

    if store_options.root_uri.is_local():
        store_fs.makedirs(posixpath.dirname(dest), exist_ok=True)  # type: ignore

    with (
        input_fs.open(data_uri.path, "rb") as source,  # type: ignore[arg-type]
        store_fs.open(dest, "wb") as sink,  # type: ignore[arg-type]
    ):
        while True:
            chunk: bytes = source.read(_COPY_CHUNK)  # type: ignore[reportAny]
            if not chunk:
                break
            sink.write(chunk)  # type: ignore[reportAny]

    etag, size = _file_metadata(store_fs, dest)

    return Reference(
        key=key,
        scope="global",
        uri=FsspecUri(f"{store_options.root_uri.protocol}://{dest}"),
        etag=etag,
        size=size,
    )
