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
import tempfile
from typing import (
    TYPE_CHECKING,
    BinaryIO,
    Literal,
    TextIO,
    assert_never,
    cast,
    overload,
    override,
)

from zygo.cli.v0.types import DataReferenceCreated
from zygo.store import StoreProtocol
from zygo.store._internal.util import build_fs, normalize_key, partition
from zygo.store.types import DataUri
from zygo.store.protocol import TmpFileProtocol

if TYPE_CHECKING:
    from types import TracebackType

    from zygo.cli.v0.transport import IpcTransport
    from zygo.store.protocol import StoreContextManager
    from zygo.store.types import Scope
    from zygo.types import JobRunContext


class StoreImpl(StoreProtocol):
    """
    A high-level store built on fsspec.
    """

    def __init__(
        self,
        *,
        context: JobRunContext,
        root: DataUri,
        ipc_transport: IpcTransport,
        kwargs: dict[str, str | int | float | bool | None] | None = None,
    ) -> None:
        super().__init__()
        self._context = context
        self._root = root
        self._ipc_transport = ipc_transport
        self._fs = build_fs(root, kwargs)

    def _is_uri(self, value: str) -> bool:
        """
        Returns True if the value is a URI.
        Useful for allowing URIs to be free passed around.
        """
        return DataUri.is_valid(value)

    def _prefix(self, scope: Scope) -> str:
        """
        Map scope -> a path prefix under the user-provided root.
        """

        # Keep paths POSIX-like even on Windows since many fsspec backends expect that.
        base = posixpath.join(self._root.path)

        match scope:
            case "job":
                return posixpath.join(
                    base,
                    partition("wr", self._context.workflow_run_id),
                    partition("jr", self._context.job_run_id),
                )
            case "workflow":
                return posixpath.join(
                    base,
                    partition("wr", self._context.workflow_run_id),
                    "shared",
                )
            case "cache":
                return posixpath.join(base, "cache")
            case _:
                assert_never(scope)

    def _uri_for_key(self, key: str, scope: Scope) -> DataUri:
        # TODO: Better interface for passing URIs directly across the whole store.
        if self._is_uri(key):
            return DataUri(key)

        path = posixpath.join(self._prefix(scope), normalize_key(key))
        return DataUri(f"{self._root.protocol}://{path}")

    @override
    def put(
        self,
        key: str,
        data: bytes,
        *,
        scope: Scope = "job",
        content_type: str | None = None,
    ) -> DataUri:
        uri = self._uri_for_key(key, scope)

        # Ensure parent directories for local-ish FS that require it
        parent = posixpath.dirname(uri.path)
        if self._root.is_local():
            self._fs.makedirs(parent, exist_ok=True)  # type: ignore

        with self._fs.open(uri, "wb") as f:  # type: ignore
            f.write(data)  # type: ignore

        # Send an IPC message to the parent process to notify it of the new data URI.
        self._ipc_transport.emit(
            DataReferenceCreated(type="data_reference_created", data_reference=str(uri))
        )

        return uri

    @override
    def get(self, key: str | DataUri, *, scope: Scope = "job") -> bytes:
        uri = key if isinstance(key, DataUri) else self._uri_for_key(key, scope)

        assert uri.protocol == self._root.protocol, f"Protocol mismatch: expected {self._root.protocol}, got {uri.protocol}"

        with self._fs.open(str(uri), "rb") as f:  # type: ignore
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
        ref: str | DataUri,
        mode: Literal["r", "w", "a", "x", "rt", "wt", "at", "xt"] = ...,
        *,
        scope: Scope = ...,
    ) -> StoreContextManager[TextIO]: ...

    @overload
    def open(
        self,
        ref: str | DataUri,
        mode: Literal["rb", "wb", "ab", "xb"],
        *,
        scope: Scope = ...,
    ) -> StoreContextManager[BinaryIO]: ...

    @overload
    def open(
        self,
        ref: str | DataUri,
        mode: str,
        *,
        scope: Scope = ...,
    ) -> StoreContextManager[TextIO | BinaryIO]: ...

    @override
    def open(
        self,
        ref: str | DataUri,
        mode: str = "r",
        *,
        scope: Scope = "job",
    ) -> StoreContextManager[TextIO | BinaryIO]:
        uri = ref if isinstance(ref, DataUri) else self._uri_for_key(ref, scope)

        # Ensure parent directories exist for write/append modes on local FS
        if any(c in mode for c in "wa"):
            parent = posixpath.dirname(str(uri))
            if self._root.is_local():
                self._fs.makedirs(parent, exist_ok=True)  # type: ignore

        context = cast(
            "AbstractContextManager[TextIO | BinaryIO]",
            self._fs.open(str(uri), mode),  # type: ignore
        )
        return _StoreOpenContext(context, uri)

    @override
    def open_file(
        self, key: str | DataUri, mode: Literal["r", "w"], *, scope: Scope = "job"
    ) -> StoreContextManager[TmpFileProtocol]:
        uri = key if isinstance(key, DataUri) else self._uri_for_key(key, scope)
        return _OpenFileContext(self, uri, mode)


class _StoreOpenContext[T](AbstractContextManager[T]):
    def __init__(
        self,
        context: AbstractContextManager[T],
        uri: DataUri,
    ) -> None:
        super().__init__()
        self._context = context
        self._target_uri = uri
        self._uri: DataUri | None = None

    @property
    def uri(self) -> DataUri:
        if self._uri is None:
            raise RuntimeError(
                "URI is only available after successful context exit"
            )
        return self._uri

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
            self._uri = self._target_uri
        return suppress


class _OpenFileContext(AbstractContextManager[TmpFileProtocol]):
    def __init__(
        self,
        store: StoreImpl,
        uri: DataUri,
        mode: Literal["r", "w"],
    ) -> None:
        super().__init__()

        if mode not in {"r", "w"}:
            raise ValueError(f"Invalid mode: {mode}")

        self._store = store
        self._target_uri = uri
        self._mode: Literal["r", "w"] = mode
        self._directory: tempfile.TemporaryDirectory[str] | None = None
        self._path: Path | None = None
        self._uri: DataUri | None = None

    @property
    def path(self) -> Path:
        if self._path is None:
            raise RuntimeError("Temporary file is only available inside its context")
        return self._path

    @property
    def uri(self) -> DataUri:
        if self._uri is None:
            raise RuntimeError(
                "URI is only available after successful context exit"
            )
        return self._uri

    @override
    def __enter__(self) -> TmpFileProtocol:
        super().__enter__()
        if self._directory is not None:
            raise RuntimeError(
                "Temporary file context cannot be entered more than once"
            )

        initial_data = (
            self._store.get(self._target_uri) if self._mode == "r" else None
        )
        self._directory = tempfile.TemporaryDirectory()

        # NB: It's important for to preserve the initial data file's name/extension.
        # Some libs will validate files by extension, so we need to preserve it.
        if initial_data is not None:
            self._path = Path(self._directory.name) / posixpath.basename(
                self._target_uri.key
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
                        self._uri = self._store.put(
                            self._target_uri.key, self.path.read_bytes()
                        )
                    case "r":
                        self._uri = self._target_uri
        finally:
            directory.cleanup()
