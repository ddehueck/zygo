"""Filesystem operations shared by workflow and model stores."""

from __future__ import annotations

from contextlib import AbstractContextManager
from pathlib import Path
import posixpath
import tempfile
from typing import TYPE_CHECKING, BinaryIO, Literal, TextIO, cast, overload, override

from zygo.cli.v0.types import DataReferenceInserted
from zygo.store._internal.util import build_fs, normalize_key
from zygo.store.protocol import TmpFileProtocol
from zygo.store.types import DataUri

if TYPE_CHECKING:
    from types import TracebackType

    from zygo.cli.v0.transport import IpcTransport
    from zygo.store.protocol import StoreContextManager


class BaseStore:
    """A key-value store rooted at an fsspec directory or prefix."""

    def __init__(
        self,
        *,
        root: DataUri,
        ipc_transport: IpcTransport,
        kwargs: dict[str, object] | None = None,
    ) -> None:
        super().__init__()
        if not str(root).endswith("/"):
            raise ValueError("Store root must be a prefix ending in '/'")
        self._root = root
        self._ipc_transport = ipc_transport
        self._kwargs = dict(kwargs) if kwargs is not None else None
        self._fs = build_fs(root, self._kwargs)

    @property
    def root(self) -> DataUri:
        return self._root

    @property
    def kwargs(self) -> dict[str, object] | None:
        return dict(self._kwargs) if self._kwargs is not None else None

    def _uri_for_key(self, key: str) -> DataUri:
        if DataUri.is_valid(key):
            return DataUri(key)
        return DataUri(f"{self._root}{normalize_key(key)}")

    def _resolve_uri(self, key: str | DataUri) -> DataUri:
        return key if isinstance(key, DataUri) else self._uri_for_key(key)

    def put(self, key: str | DataUri, data: bytes) -> DataUri:
        uri = self._resolve_uri(key)
        if self._root.is_local():
            self._fs.makedirs(posixpath.dirname(uri.path), exist_ok=True)
        with self._fs.open(str(uri), "wb") as f:
            f.write(data)
        self._ipc_transport.emit(
            DataReferenceInserted(
                type="data_reference_inserted", data_reference=str(uri)
            )
        )
        return uri

    def get(self, key: str | DataUri) -> bytes:
        uri = self._resolve_uri(key)
        if uri.protocol != self._root.protocol:
            raise ValueError(
                f"Protocol mismatch: expected {self._root.protocol}, got {uri.protocol}"
            )
        with self._fs.open(str(uri), "rb") as f:
            return f.read()

    def exists(self, key: str | DataUri) -> bool:
        return self._fs.exists(str(self._resolve_uri(key)))

    def delete(self, key: str | DataUri) -> None:
        uri = self._resolve_uri(key)
        if self._fs.exists(str(uri)):
            self._fs.rm(str(uri))

    @overload
    def open(
        self,
        ref: str | DataUri,
        mode: Literal["r", "w", "a", "x", "rt", "wt", "at", "xt"] = ...,
    ) -> StoreContextManager[TextIO]: ...

    @overload
    def open(
        self,
        ref: str | DataUri,
        mode: Literal["rb", "wb", "ab", "xb"],
    ) -> StoreContextManager[BinaryIO]: ...

    @overload
    def open(
        self,
        ref: str | DataUri,
        mode: str,
    ) -> StoreContextManager[TextIO | BinaryIO]: ...

    def open(
        self,
        ref: str | DataUri,
        mode: str = "r",
    ) -> StoreContextManager[TextIO | BinaryIO]:
        uri = self._resolve_uri(ref)
        if any(c in mode for c in "wax") and self._root.is_local():
            self._fs.makedirs(posixpath.dirname(uri.path), exist_ok=True)
        context = cast(
            "AbstractContextManager[TextIO | BinaryIO]",
            self._fs.open(str(uri), mode),
        )
        return _StoreOpenContext(context, uri)

    def open_file(
        self, key: str | DataUri, mode: Literal["r", "w"]
    ) -> StoreContextManager[TmpFileProtocol]:
        return _OpenFileContext(self, self._resolve_uri(key), mode)


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
            raise RuntimeError("URI is only available after successful context exit")
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
        store: BaseStore,
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
            raise RuntimeError("URI is only available after successful context exit")
        return self._uri

    @override
    def __enter__(self) -> TmpFileProtocol:
        super().__enter__()
        if self._directory is not None:
            raise RuntimeError(
                "Temporary file context cannot be entered more than once"
            )
        initial_data = self._store.get(self._target_uri) if self._mode == "r" else None
        self._directory = tempfile.TemporaryDirectory()
        # Preserve the extension for libraries that identify formats by filename.
        if initial_data is not None:
            self._path = Path(self._directory.name) / self._target_uri.key
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
                if self._mode == "w":
                    self._uri = self._store.put(
                        self._target_uri, self.path.read_bytes()
                    )
                else:
                    self._uri = self._target_uri
        finally:
            directory.cleanup()
