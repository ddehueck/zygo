"""Training-root store adapter."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, BinaryIO, Literal, TextIO, overload

from zygo.ml.bundle import ModelBundle
from zygo.store import DataUri

if TYPE_CHECKING:
    from types import TracebackType

    from zygo.store import StoreContextManager, StoreProtocol
    from zygo.store.protocol import TmpFileProtocol
    from zygo.store.types import Scope


class ArtifactStore:
    def __init__(self, store: StoreProtocol, root: DataUri) -> None:
        super().__init__()
        self.store: StoreProtocol = store

        self._root = ModelBundle(root)

    def _ref(self, key: str | DataUri, scope: Scope) -> str | DataUri:
        if isinstance(key, DataUri) or scope != "job" or DataUri.is_valid(key):
            return key
        return str(self._root.artifact(key))

    def put(
        self,
        key: str,
        data: bytes,
        *,
        scope: Scope = "job",
        content_type: str | None = None,
    ) -> DataUri:
        return self.store.put(
            str(self._ref(key, scope)), data, scope=scope, content_type=content_type
        )

    @overload
    def get(self, key: str, *, scope: Scope = "job") -> bytes: ...

    @overload
    def get(self, key: DataUri) -> bytes: ...

    def get(self, key: str | DataUri, *, scope: Scope = "job") -> bytes:
        if isinstance(key, DataUri):
            return self.store.get(key)
        return self.store.get(str(self._ref(key, scope)), scope=scope)

    def exists(self, key: str, *, scope: Scope = "job") -> bool:
        return self.store.exists(str(self._ref(key, scope)), scope=scope)

    def delete(self, key: str, *, scope: Scope = "job") -> None:
        self.store.delete(str(self._ref(key, scope)), scope=scope)

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
        return self.store.open(self._ref(ref, scope), mode, scope=scope)

    def open_file(
        self, key: str | DataUri, mode: Literal["r", "w"], *, scope: Scope = "job"
    ) -> StoreContextManager[TmpFileProtocol]:
        return self.store.open_file(self._ref(key, scope), mode, scope=scope)


class TrainingStore(ArtifactStore):
    """A persistent training directory with delegated key-value operations.

    Native saves through path are supported for file:// roots only.
    Context exit releases the workspace without publishing a bundle or
    validating its contents. Writes persist even if the block fails.
    """

    def __init__(self, store: StoreProtocol, root: DataUri) -> None:
        super().__init__(store, root)
        self._entered = False
        self._active = False

    @property
    def path(self) -> Path:
        """The actual local directory, available inside the context."""
        if self._root.uri.protocol != "file":
            raise ValueError("store.path is only available for file:// training stores")
        if not self._active:
            raise RuntimeError("store.path is only available inside its context")
        return Path(self._root.uri.path)

    def __enter__(self) -> TrainingStore:
        if self._entered:
            raise RuntimeError("Training store cannot be entered more than once")
        self._entered = True
        self._active = True
        if self._root.uri.protocol == "file":
            self.path.mkdir(parents=True, exist_ok=True)
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
        /,
    ) -> None:
        self._active = False
