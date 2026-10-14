"""Context manager for collecting and preparing dataset edits."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
from pathlib import Path
from typing import TYPE_CHECKING, Self

from zygo.cloud.datasets.operations import (
    AddFileOperation,
    RemoveDirOperation,
    RemoveFileOperation,
)

if TYPE_CHECKING:
    from types import TracebackType

    from zygo.cloud.datasets.api import CommitResponse, DataRepositoryApiClient
    from zygo.cloud.datasets.operations import Operation

__all__ = ["DatasetEditor"]


@dataclass(frozen=True)
class _PendingAdd:
    source: Path
    destination: str


class DatasetEditor:
    """Collect dataset changes and prepare their operation records."""

    def __init__(
        self,
        *,
        repo_slug: str | None = None,
        client: DataRepositoryApiClient | None = None,
    ) -> None:
        if client is not None and repo_slug is None:
            raise ValueError("repo_slug is required when a data repository client is set")
        self._repo_slug = repo_slug
        self._client = client
        self._pending_operations: list[Operation | _PendingAdd] = []
        self._operations: tuple[Operation, ...] = ()
        self._commit_result: CommitResponse | None = None
        self._entered = False
        self._finished = False

    def __enter__(self) -> Self:
        if self._entered:
            raise RuntimeError("A dataset editor cannot be entered more than once")
        self._entered = True
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        del exc_value, traceback
        self._finished = True
        if exc_type is None:
            self._finalize_operations()
        else:
            self._pending_operations.clear()
            self._operations = ()

    @property
    def operations(self) -> tuple[Operation, ...]:
        """The prepared operations, available after a successful context exit."""

        if not self._finished:
            raise RuntimeError("Dataset operations are available after the edit exits")
        return self._operations

    @property
    def commit_result(self) -> CommitResponse | None:
        """The API commit response, or ``None`` when no client was configured."""

        if not self._finished:
            raise RuntimeError("The commit result is available after the edit exits")
        return self._commit_result

    def put(self, *, src: str | Path, dest: str) -> None:
        """Queue a local file to be added at ``dest`` in the dataset."""

        self._ensure_editing()
        source = Path(src)
        if not source.is_file():
            raise FileNotFoundError(source)
        self._pending_operations.append(
            _PendingAdd(source=source, destination=_normalize_repo_path(dest))
        )

    def put_dir(self, src: str | Path, dest: str | None = None) -> None:
        """Queue every file in a local directory, preserving its tree."""

        self._ensure_editing()
        source = Path(src)
        if not source.is_dir():
            raise NotADirectoryError(source)

        destination_root = _normalize_repo_path(dest or source.name)
        for path in sorted(source.rglob("*")):
            if path.is_file():
                relative_path = path.relative_to(source).as_posix()
                destination = _normalize_repo_path(
                    f"{destination_root}/{relative_path}"
                )
                self._pending_operations.append(
                    _PendingAdd(source=path, destination=destination)
                )

    def remove_file(self, path: str) -> None:
        """Queue a file removal by its dataset path."""

        self._ensure_editing()
        self._pending_operations.append(RemoveFileOperation(_normalize_repo_path(path)))

    def remove_dir(self, path: str) -> None:
        """Queue a directory removal by its dataset path."""

        self._ensure_editing()
        self._pending_operations.append(RemoveDirOperation(_normalize_repo_path(path)))

    def _ensure_editing(self) -> None:
        if not self._entered or self._finished:
            raise RuntimeError("Dataset changes must be added inside the edit context")

    def _finalize_operations(self) -> None:
        operations: list[Operation] = []
        for pending in self._pending_operations:
            if not isinstance(pending, _PendingAdd):
                operations.append(pending)
                continue
            digest = hashlib.sha256()
            size = 0
            with pending.source.open("rb") as source_file:
                while chunk := source_file.read(1024 * 1024):
                    digest.update(chunk)
                    size += len(chunk)
            operations.append(
                AddFileOperation(
                    path=pending.destination,
                    hash=digest.hexdigest(),
                    size=size,
                    source=pending.source,
                )
            )
        self._operations = tuple(operations)
        self._pending_operations.clear()
        if self._client is not None and self._operations:
            if self._repo_slug is None:
                raise RuntimeError("repo_slug is required to commit dataset edits")
            self._commit_result = self._client.apply(self._repo_slug, self._operations)


def _normalize_repo_path(path: str) -> str:
    normalized = path.replace("\\", "/").strip("/")
    parts = normalized.split("/")
    if not normalized or any(part in {"", ".", ".."} for part in parts):
        raise ValueError(f"Invalid dataset path: {path!r}")
    return normalized
