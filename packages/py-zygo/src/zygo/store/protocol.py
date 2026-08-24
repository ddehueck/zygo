"""Protocol interface for the Store abstraction."""

from __future__ import annotations

from typing import TYPE_CHECKING, BinaryIO, Literal, Protocol, TextIO, overload

if TYPE_CHECKING:
    from pathlib import Path
    from types import TracebackType

    from zygo.store.types import Reference, Scope


class StoreContextManager[T](Protocol):
    @property
    def reference(self) -> Reference:
        """The stored object reference, available after successful context exit."""
        ...

    def __enter__(self) -> T: ...

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
        /,
    ) -> bool | None: ...


class StoreProtocol(Protocol):
    """
    A key-value store with workflow/task-based isolation.
    Built on top of fsspec to provide storage backends (local filesystem, S3, GCS, etc.)
    """

    def put(
        self,
        key: str,
        data: bytes,
        *,
        scope: Scope = "job",
        content_type: str | None = None,
    ) -> Reference:
        """
        Store data under the given key.

        Args:
            key: Logical name for the data.
            data: Raw bytes to store.
            scope: Isolation level - "job" (default), "workflow", or "global".
            content_type: Optional MIME type hint.

        Returns:
            A DataRef with metadata about the stored object.
        """
        ...

    @overload
    def get(self, key: str, *, scope: Scope = "job") -> bytes: ...

    @overload
    def get(self, key: Reference) -> bytes: ...

    def get(self, key: str | Reference, *, scope: Scope = "job") -> bytes:
        """
        Retrieve data by key or by Reference.

        Args:
            key: Logical name for the data, or a Reference obtained from put/ingest.
            scope: Isolation level to look in (default: "job"). Ignored when
                a Reference is passed.

        Returns:
            The stored bytes.

        Raises:
            FileNotFoundError: If the key does not exist.
        """
        ...

    def exists(self, key: str, *, scope: Scope = "job") -> bool:
        """
        Check if a key exists.

        Args:
            key: Logical name for the data.
            scope: Isolation level to look in (default: "job").

        Returns:
            True if the key exists, False otherwise.
        """
        ...

    def delete(self, key: str, *, scope: Scope = "job") -> None:
        """
        Delete data by key.

        Args:
            key: Logical name for the data.
            scope: Isolation level to look in (default: "job").

        Note:
            No-op if the key does not exist.
        """
        ...

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

    def open(
        self,
        ref: str | Reference,
        mode: str = "r",
        *,
        scope: Scope = "job",
    ) -> StoreContextManager[TextIO | BinaryIO]:
        """
        Open a stored object as a file-like handle (context manager).

        Supports both text and binary modes, just like the built-in ``open()``.

        Args:
            ref: A Reference obtained from put/ingest, or a logical key string.
            mode: File mode string (e.g. ``"r"``, ``"w"``, ``"rb"``, ``"wb"``).
            scope: Isolation level (default: "job"). Ignored when a Reference
                is passed.

        Returns:
            A context manager that yields a file-like object and exposes its
            reference after a successful context exit.

        Example::

            with store.open(ref, "r") as f:
                lines = f.readlines()

            output = store.open("output.txt", "w")
            with output as f:
                f.writelines(lines)

            reference = output.reference
        """
        ...

    def open_file(
        self, key: str, mode: Literal["r", "w"], *, scope: Scope = "job"
    ) -> StoreContextManager[TmpFileProtocol]:
        """Return a temporary file.

        When in write mode, the file is published under ``key`` on exit.
        When in read mode, the file is downloaded from the store on enter.

        This isn't as efficient as using open() but it is useful for integrating
        with libraries that expect a real fs.

        Args:
            key: Logical name to read or publish.
            mode: ``"r"`` to download an existing object or ``"w"`` to publish
                the completed file.
            scope: Isolation level (default: "job").
        """
        ...


class TmpFileProtocol(Protocol):
    @property
    def path(self) -> Path:
        """The local path containing the temporary file."""
        ...

    @property
    def reference(self) -> Reference:
        """The stored object reference, available after successful context exit."""
        ...
