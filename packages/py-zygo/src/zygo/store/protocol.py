"""Interfaces for store file context managers."""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from pathlib import Path
    from types import TracebackType

    from zygo.store.fsspec import FsspecUri as DataUri


class StoreContextManager[T](Protocol):
    @property
    def uri(self) -> DataUri:
        """The stored object URI, available after successful context exit."""
        ...

    def __enter__(self) -> T: ...

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
        /,
    ) -> bool | None: ...


class TmpFileProtocol(Protocol):
    @property
    def path(self) -> Path:
        """The local path containing the temporary file."""
        ...

    @property
    def uri(self) -> DataUri:
        """The stored object uri, available after successful context exit."""
        ...
