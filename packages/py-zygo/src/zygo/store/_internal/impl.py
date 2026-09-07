"""
This store is built on top of fsspec to enable local and remote data storage access.
The store is a key-value store that can be used to store and retrieve data.

Users bring their own protocol and root directory.
Meanwhile, the store provides data isolation and versioning relative to the orchestration requirements.

This way, task data is easily isolated to avoid data contamination between tasks by default.
A user can still opt-in to a shared store across a workflow run via the `scope` parameter.
"""

from __future__ import annotations

from contextlib import AbstractContextManager
from pathlib import Path
import posixpath
import re
import tempfile
from typing import TYPE_CHECKING, BinaryIO, Literal, TextIO, cast, overload, override

import fsspec  # type: ignore

from zygo._internal.fsspec import FsspecUri
from zygo.store import Reference, StoreProtocol
from zygo.store.protocol import TmpFileProtocol

from zygo._internal.ipc.v0.types import (
    DataReferenceCreated,
    DataReference,
    write_stdout_ipc_message,
)

if TYPE_CHECKING:
    from types import TracebackType

    from fsspec.spec import AbstractFileSystem  # type: ignore

    from zygo.store._internal.types import PartitionKey
    from zygo.store.protocol import StoreContextManager
    from zygo.store.types import Scope, StoreOptions
    from zygo.types import JobRunContext


def _partition(partition_key: PartitionKey, value: str) -> str:
    return f"{partition_key}={value}"


def _contains_any_partition_key(key: str, partition_keys: list[PartitionKey]) -> bool:
    return any(f"{pk}=" in key for pk in partition_keys)

def _is_global_uri(value: str) -> bool:
    return "store/global" in value


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


class StoreImpl(StoreProtocol):
    """
    A high-level store built on fsspec.
    """

    def __init__(self, *, context: JobRunContext, options: StoreOptions) -> None:
        super().__init__()
        self._context = context
        self._options = options
        self._fs = _build_fs(options)

    def _is_uri(self, value: str) -> bool:
        """
        Returns True if the value is a URI.
        Useful for allowing URIs to be free passed around.
        """
        # TODO: This should really just check for a protocol prefix.
        if _contains_any_partition_key(value, ["job_run_id", "workflow_run_id"]):
            return True
        return _is_global_uri(value)

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
        # TODO: Better interface for passing URIs directly across the whole store.
        if self._is_uri(key):
            return key

        key = _normalize_key(key)
        prefix = self._prefix(scope)
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

        # Ensure parent directories for local-ish FS that require it
        parent = posixpath.dirname(uri)
        if self._options.root_uri.is_local():
            self._fs.makedirs(parent, exist_ok=True)  # type: ignore

        with self._fs.open(uri, "wb") as f:  # type: ignore
            f.write(data)  # type: ignore

        # Send an IPC message to the parent process to notify it of the new data reference.
        write_stdout_ipc_message(
            DataReferenceCreated(
                data_reference=DataReference(
                    uri=uri,
                    version="0",
                )
            )
        )

        return Reference(
            key=key,
            uri=FsspecUri(uri),
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
    ) -> StoreContextManager[TextIO]: ...

    @overload
    def open(
        self,
        ref: str | Reference,
        mode: Literal["rb", "wb", "ab", "xb"],
        *,
        scope: Scope = ...,
    ) -> StoreContextManager[BinaryIO]: ...

    @overload
    def open(
        self,
        ref: str | Reference,
        mode: str,
        *,
        scope: Scope = ...,
    ) -> StoreContextManager[TextIO | BinaryIO]: ...

    @override
    def open(
        self,
        ref: str | Reference,
        mode: str = "r",
        *,
        scope: Scope = "job",
    ) -> StoreContextManager[TextIO | BinaryIO]:
        uri_raw = (
            ref.uri if isinstance(ref, Reference) else self._uri_for_key(ref, scope)
        )
        uri_str = str(uri_raw) if not isinstance(uri_raw, str) else uri_raw

        # Ensure parent directories exist for write/append modes on local FS
        if any(c in mode for c in "wa"):
            parent = posixpath.dirname(uri_str)
            if self._options.root_uri.is_local():
                self._fs.makedirs(parent, exist_ok=True)  # type: ignore

        context = cast(
            "AbstractContextManager[TextIO | BinaryIO]",
            self._fs.open(uri_str, mode),  # type: ignore
        )
        reference = (
            ref
            if isinstance(ref, Reference)
            else Reference(key=ref, uri=FsspecUri(uri_str))
        )
        return _StoreOpenContext(context, reference)

    @override
    def open_file(
        self, key: str, mode: Literal["r", "w"], *, scope: Scope = "job"
    ) -> StoreContextManager[TmpFileProtocol]:
        reference = Reference(
            key=key,
            uri=FsspecUri(self._uri_for_key(key, scope)),
        )
        return _OpenFileContext(self, reference, mode)


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

    uri = FsspecUri(f"{store_options.root_uri.protocol}://{dest}")

    # Send an IPC message to the parent process to notify it of the new data reference.
    write_stdout_ipc_message(
        DataReferenceCreated(
            data_reference=DataReference(
                uri=str(uri),
                version="0",
            )
        )
    )

    return Reference(
        key=key,
        uri=uri,
    )


class _StoreOpenContext[T](AbstractContextManager[T]):
    def __init__(
        self,
        context: AbstractContextManager[T],
        reference: Reference,
    ) -> None:
        super().__init__()
        self._context = context
        self._target_reference = reference
        self._reference: Reference | None = None

    @property
    def reference(self) -> Reference:
        if self._reference is None:
            raise RuntimeError(
                "Reference is only available after successful context exit"
            )
        return self._reference

    @override
    def __enter__(self) -> T:
        super().__enter__()
        return self._context.__enter__()

    @override
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> bool | None:
        suppress = self._context.__exit__(exc_type, exc_value, traceback)
        if exc_type is None:
            self._reference = self._target_reference
        return suppress


class _OpenFileContext(AbstractContextManager[TmpFileProtocol]):
    def __init__(
        self,
        store: StoreImpl,
        reference: Reference,
        mode: Literal["r", "w"],
    ) -> None:
        super().__init__()

        if mode not in {"r", "w"}:
            raise ValueError(f"Invalid mode: {mode}")

        self._store = store
        self._target_reference = reference
        self._mode: Literal["r", "w"] = mode
        self._directory: tempfile.TemporaryDirectory[str] | None = None
        self._path: Path | None = None
        self._reference: Reference | None = None

    @property
    def path(self) -> Path:
        if self._path is None:
            raise RuntimeError("Temporary file is only available inside its context")
        return self._path

    @property
    def reference(self) -> Reference:
        if self._reference is None:
            raise RuntimeError(
                "Reference is only available after successful context exit"
            )
        return self._reference

    @override
    def __enter__(self) -> TmpFileProtocol:
        super().__enter__()
        if self._directory is not None:
            raise RuntimeError(
                "Temporary file context cannot be entered more than once"
            )

        initial_data = (
            self._store.get(self._target_reference) if self._mode == "r" else None
        )
        self._directory = tempfile.TemporaryDirectory()

        # NB: It's important for to preserve the initial data file's name/extension.
        # Some libs will validate files by extension, so we need to preserve it.
        if initial_data is not None:
            self._path = Path(self._directory.name) / posixpath.basename(
                self._target_reference.key
            )
            self._path.write_bytes(initial_data)
        else:
            with tempfile.NamedTemporaryFile(
                dir=self._directory.name,
                delete=False,
            ) as temporary_file:
                self._path = Path(temporary_file.name)
        return self

    @override
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        _exc_value: BaseException | None,
        _traceback: TracebackType | None,
    ) -> None:
        directory = self._directory
        if directory is None:
            raise RuntimeError("Temporary file context has not been entered")

        try:
            if exc_type is None:
                match self._mode:
                    case "w":
                        self._reference = self._store.put(
                            self._target_reference.key, self.path.read_bytes()
                        )
                    case "r":
                        self._reference = self._target_reference
        finally:
            directory.cleanup()
