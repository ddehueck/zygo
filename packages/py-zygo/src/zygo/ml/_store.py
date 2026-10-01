"""Internal store proxy for tracking written artifacts."""

from __future__ import annotations

from typing import TYPE_CHECKING, BinaryIO, Literal, TextIO, overload

from zygo.store import DataUri, StoreContextManager, StoreProtocol

if TYPE_CHECKING:
    from types import TracebackType

    from zygo.store.protocol import TmpFileProtocol
    from zygo.store.types import Scope


class ArtifactStore:
    def __init__(self, store: StoreProtocol) -> None:
        super().__init__()
        self.store: StoreProtocol = store
        self.writes: set[DataUri] = set()

    def put(
        self,
        key: str,
        data: bytes,
        *,
        scope: Scope = "job",
        content_type: str | None = None,
    ) -> DataUri:
        uri = self.store.put(key, data, scope=scope, content_type=content_type)
        self.writes.add(uri)
        return uri

    @overload
    def get(self, key: str, *, scope: Scope = "job") -> bytes: ...

    @overload
    def get(self, key: DataUri) -> bytes: ...

    def get(self, key: str | DataUri, *, scope: Scope = "job") -> bytes:
        if isinstance(key, DataUri):
            return self.store.get(key)
        return self.store.get(key, scope=scope)

    def exists(self, key: str, *, scope: Scope = "job") -> bool:
        return self.store.exists(key, scope=scope)

    def delete(self, key: str, *, scope: Scope = "job") -> None:
        self.store.delete(key, scope=scope)

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

    def open(
        self,
        ref: str | DataUri,
        mode: str = "r",
        *,
        scope: Scope = "job",
    ) -> StoreContextManager[TextIO | BinaryIO]:
        context = self.store.open(ref, mode, scope=scope)
        return _ArtifactContext(context, self.writes, any(c in mode for c in "wax+"))

    def open_file(
        self, key: str | DataUri, mode: Literal["r", "w"], *, scope: Scope = "job"
    ) -> StoreContextManager[TmpFileProtocol]:
        context = self.store.open_file(key, mode, scope=scope)
        return _ArtifactContext(context, self.writes, mode == "w")


class _ArtifactContext[T]:
    def __init__(
        self,
        context: StoreContextManager[T],
        writes: set[DataUri],
        write: bool,
    ) -> None:
        super().__init__()
        self._context = context
        self._writes = writes
        self._write = write

    @property
    def uri(self) -> DataUri:
        return self._context.uri

    def __enter__(self) -> T:
        return self._context.__enter__()

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
        /,
    ) -> bool | None:
        suppress = self._context.__exit__(exc_type, exc_value, traceback)
        if self._write and exc_type is None:
            self._writes.add(self._context.uri)
        return suppress
