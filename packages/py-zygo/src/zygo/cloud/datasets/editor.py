"""Context manager for collecting and preparing dataset edits."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Self

from blake3 import blake3
from pathspec import GitIgnoreSpec

from zygo.cloud.datasets.operations import (
    AddFileOperation,
    RemoveDirOperation,
    RemoveFileOperation,
)
from zygo.cloud.datasets.progress import ProgressTracker

if TYPE_CHECKING:
    from collections.abc import Sequence
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
            raise ValueError(
                "repo_slug is required when a data repository client is set"
            )
        self._repo_slug = repo_slug
        self._client = client
        self._pending_operations: list[Operation | _PendingAdd] = []
        self._operations: tuple[Operation, ...] = ()
        self._commit_result: CommitResponse | None = None
        self._progress_tracker: ProgressTracker | None = None
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
        exc_val: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        del exc_val, traceback
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

    def put(self, src: str | Path, dest: str | None = None) -> None:
        """Queue a local file to be added at ``dest`` in the dataset."""

        self._ensure_editing()
        source = Path(src)
        if not source.is_file():
            raise FileNotFoundError(source)
        self._pending_operations.append(
            _PendingAdd(
                source=source, destination=_normalize_repo_path(dest or source.name)
            )
        )

    def put_dir(
        self,
        src: str | Path,
        dest: str | None = None,
        *,
        ignore: Sequence[str] | None = None,
    ) -> None:
        """Queue files from a directory, skipping paths matched by ``ignore``.

        ``ignore`` is an ordered sequence of gitignore-style patterns applied
        to POSIX paths relative to ``src``. For example, ``*.tmp`` matches
        files with that suffix at any depth, ``/data/*.csv`` anchors at the
        source root, and ``cache/`` skips matching directories and their
        contents. Prefix a pattern with ``!`` to re-include a path. As in
        ``.gitignore``, a child cannot be re-included when its parent directory
        is ignored.
        """

        self._ensure_editing()
        source = Path(src)
        if not source.is_dir():
            raise NotADirectoryError(source)

        destination_root = _normalize_repo_path(dest or source.name)
        ignore_spec = GitIgnoreSpec.from_lines(ignore or ())
        files_to_add, ignored_paths = _collect_directory_paths(source, ignore_spec)
        for relative_path, is_directory in sorted(ignored_paths):
            if is_directory:
                self._log_ignored_dir(relative_path)
            else:
                self._log_ignored_file(relative_path)

        for path in sorted(files_to_add):
            relative_path = path.relative_to(source).as_posix()
            destination = _normalize_repo_path(f"{destination_root}/{relative_path}")
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

    def _log_ignored_file(self, relative_path: str) -> None:
        tracker = self._progress_tracker
        if tracker is None:
            tracker = ProgressTracker()
            self._progress_tracker = tracker
        tracker.log_ignored_file(relative_path)

    def _log_ignored_dir(self, relative_path: str) -> None:
        tracker = self._progress_tracker
        if tracker is None:
            tracker = ProgressTracker()
            self._progress_tracker = tracker
        tracker.log_ignored_dir(relative_path)

    def _finalize_operations(self) -> None:
        operations: list[Operation] = []
        tracker = self._progress_tracker or ProgressTracker()
        self._progress_tracker = tracker
        additions = sum(
            isinstance(item, _PendingAdd) for item in self._pending_operations
        )
        file_removals = sum(
            isinstance(item, RemoveFileOperation) for item in self._pending_operations
        )
        directory_removals = sum(
            isinstance(item, RemoveDirOperation) for item in self._pending_operations
        )
        if self._pending_operations:
            tracker.display_data_change_summary(
                additions=additions,
                file_removals=file_removals,
                directory_removals=directory_removals,
            )

        hashed_additions = self._hash_additions(additions, tracker)

        hashed = iter(hashed_additions)
        for pending in self._pending_operations:
            if isinstance(pending, _PendingAdd):
                digest, size = next(hashed)
                operations.append(
                    AddFileOperation(
                        path=pending.destination,
                        hash=digest,
                        size=size,
                        source=pending.source,
                    )
                )
            else:
                operations.append(pending)
        self._operations = tuple(operations)
        self._pending_operations.clear()
        if self._client is not None and self._operations:
            if self._repo_slug is None:
                raise RuntimeError("repo_slug is required to commit dataset edits")
            self._commit_result = self._client.apply(
                self._repo_slug, self._operations, progress=tracker
            )

    def _hash_additions(
        self, additions: int, tracker: ProgressTracker
    ) -> list[tuple[str, int]]:
        if not additions:
            return []

        hashed_additions: list[tuple[str, int]] = []
        with tracker.stage(
            f"Hash data ({additions} files)", total=additions, unit="files"
        ):
            for pending in self._pending_operations:
                if not isinstance(pending, _PendingAdd):
                    continue
                hashed_additions.append(_hash_file(pending.source))
                tracker.advance_stage()
        return hashed_additions


def _hash_file(path: Path) -> tuple[str, int]:
    digest = blake3(max_threads=blake3.AUTO)
    size = 0
    with path.open("rb") as source_file:
        while chunk := source_file.read(1024 * 1024):
            digest.update(chunk)
            size += len(chunk)
    return digest.hexdigest(), size


def _collect_directory_paths(
    source: Path,
    ignore_spec: GitIgnoreSpec,
) -> tuple[list[Path], list[tuple[str, bool]]]:
    files_to_add: list[Path] = []
    ignored_paths: list[tuple[str, bool]] = []
    for directory, child_dirs, filenames in source.walk(top_down=True):
        relative_directory = directory.relative_to(source)
        ignored_child_dirs = [
            child_dir
            for child_dir in child_dirs
            if _is_ignored(
                ignore_spec,
                f"{(relative_directory / child_dir).as_posix()}/",
            )
        ]
        ignored_paths.extend(
            ((relative_directory / child_dir).as_posix(), True)
            for child_dir in ignored_child_dirs
        )
        child_dirs[:] = [
            child_dir for child_dir in child_dirs if child_dir not in ignored_child_dirs
        ]
        for filename in filenames:
            path, ignored_path = _classify_directory_file(
                source, directory / filename, ignore_spec
            )
            if ignored_path is not None:
                ignored_paths.append(ignored_path)
            if path is not None:
                files_to_add.append(path)
    return sorted(files_to_add), ignored_paths


def _classify_directory_file(
    source: Path,
    path: Path,
    ignore_spec: GitIgnoreSpec,
) -> tuple[Path | None, tuple[str, bool] | None]:
    relative_path = path.relative_to(source).as_posix()
    if path.is_dir():
        if _is_ignored(ignore_spec, f"{relative_path}/"):
            return None, (relative_path, True)
        return None, None
    if not path.is_file():
        return None, None
    if _is_ignored(ignore_spec, relative_path):
        return None, (relative_path, False)
    return path, None


def _is_ignored(ignore_spec: GitIgnoreSpec, relative_path: str) -> bool:
    return ignore_spec.match_file(relative_path)


def _normalize_repo_path(path: str) -> str:
    normalized = path.replace("\\", "/").strip("/")
    parts = normalized.split("/")
    if not normalized or any(part in {"", ".", ".."} for part in parts):
        raise ValueError(f"Invalid dataset path: {path!r}")
    return normalized
