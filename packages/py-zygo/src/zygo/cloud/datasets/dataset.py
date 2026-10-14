"""A versioned data repository exposed as an fsspec filesystem."""

from __future__ import annotations

from io import IOBase
from pathlib import Path
import re
from tempfile import NamedTemporaryFile
from typing import IO, TYPE_CHECKING, cast, override
from urllib.parse import urlsplit
from urllib.request import Request, urlopen

from blake3 import blake3
from fsspec.spec import AbstractBufferedFile, AbstractFileSystem

from zygo.cloud.datasets.api import (
    DataRepositoryApiClient,
    DataRepositoryApiError,
    DirectoryEntry,
    FileEntry,
)
from zygo.cloud.datasets.api.util import is_local_http
from zygo.cloud.datasets.editor import DatasetEditor
from zygo.cloud.datasets.operations import AddFileOperation, RemoveFileOperation

if TYPE_CHECKING:
    from collections.abc import Callable
    from types import TracebackType

    from zygo.cloud.datasets.api import EntriesResponse, Entry

__all__ = ["Dataset"]

_SLUG_PATTERN = re.compile(r"^[a-z0-9_-]+$")
_HTTP_NOT_FOUND = 404


class Dataset(AbstractFileSystem):
    """A repository-scoped filesystem with versioned dataset editing."""

    protocol = "zygo_repo"
    cachable = False

    def __init__(
        self,
        id: str,
        *,
        client: DataRepositoryApiClient | None = None,
        revision: str | None = None,
    ) -> None:
        if not _SLUG_PATTERN.fullmatch(id):
            raise ValueError(
                "Dataset id must contain only lowercase letters, numbers, "
                "underscores, and hyphens"
            )
        self._id = id
        self._client = client
        self._revision = revision
        super().__init__(skip_instance_cache=True)

    @property
    def id(self) -> str:
        """The dataset's repository slug."""

        return self._id

    @property
    def slug(self) -> str:
        """The dataset's repository slug."""

        return self._id

    @property
    def revision(self) -> str | None:
        """The pinned commit ID, or ``None`` to read the current head."""

        return self._revision

    def edit(self) -> DatasetEditor:
        """Create an editor backed by the configured Zygo Cloud API client."""

        return DatasetEditor(repo_slug=self._id, client=self._api_client())

    @override
    def ls(
        self, path: str, detail: bool = True, **kwargs: object
    ) -> list[dict[str, object]] | list[str]:
        del kwargs
        normalized_path = _normalize_path(path, allow_root=True)
        cursor: str | None = None
        revision = self._revision
        entries: list[dict[str, object]] = []

        while True:
            try:
                response = self._api_client().list_entries(
                    self._id,
                    path=normalized_path or None,
                    revision=revision,
                    cursor=cursor,
                )
            except DataRepositoryApiError as error:
                if error.status_code == _HTTP_NOT_FOUND:
                    raise FileNotFoundError(normalized_path) from error
                raise
            if revision is None:
                revision = response.commit_id
            entries.extend(_to_fsspec_entry(entry) for entry in response.entries)
            cursor = response.next_cursor
            if cursor is None:
                break

        if detail:
            return entries
        return [str(entry["name"]) for entry in entries]

    def _info(self, path: str, **kwargs: object) -> dict[str, object]:
        del kwargs
        normalized_path = _normalize_path(path, allow_root=True)
        if not normalized_path:
            self._api_list_entries("", limit=1)
            return {"name": "", "size": 0, "type": "directory"}

        response = self._api_list_entries(normalized_path, limit=1)
        for entry in response.entries:
            if entry.path == normalized_path:
                return _to_fsspec_entry(entry)
        if response.entries:
            return {"name": normalized_path, "size": 0, "type": "directory"}
        raise FileNotFoundError(normalized_path)

    @override
    def _open(
        self,
        path: str,
        mode: str = "rb",
        block_size: int | None = None,
        autocommit: bool = True,
        cache_options: dict[str, object] | None = None,
        **kwargs: object,
    ) -> IO[bytes]:
        del autocommit, cache_options, kwargs
        normalized_path = _normalize_path(path)
        client = self._api_client()

        match mode:
            case "rb":
                try:
                    resolved = client.resolve(
                        self._id, normalized_path, revision=self._revision
                    )
                except DataRepositoryApiError as error:
                    if error.status_code == _HTTP_NOT_FOUND:
                        raise FileNotFoundError(normalized_path) from error
                    raise
                parsed_url = urlsplit(resolved.download_url)
                is_local_http_url = is_local_http(resolved.download_url)
                if parsed_url.scheme != "https" and not is_local_http_url:
                    raise ValueError(
                        "Presigned download URL must use HTTPS or loopback HTTP"
                    )
                return cast(
                    "IO[bytes]",
                    _ResolvedReadFile(
                        self,
                        normalized_path,
                        resolved.download_url,
                        size=resolved.entry.size_bytes,
                        block_size=(
                            block_size if block_size is not None else self.blocksize
                        ),
                    ),
                )
            case "wb":
                if self._revision is not None:
                    raise ValueError("A dataset pinned to a revision is read-only")
                return cast(
                    "IO[bytes]",
                    _DatasetWriteFile(
                        client, self._id, normalized_path, self.invalidate_cache
                    ),
                )
            case _:
                raise ValueError(f"Dataset files do not support mode {mode!r}")

    @override
    def _rm(self, path: str) -> None:
        if self._revision is not None:
            raise ValueError("A dataset pinned to a revision is read-only")
        normalized_path = _normalize_path(path)
        self._api_client().apply(
            self._id,
            (RemoveFileOperation(normalized_path),),
        )
        self.invalidate_cache()

    def _api_list_entries(
        self, path: str, *, limit: int
    ) -> EntriesResponse:
        try:
            return self._api_client().list_entries(
                self._id,
                path=path or None,
                revision=self._revision,
                limit=limit,
            )
        except DataRepositoryApiError as error:
            if error.status_code == _HTTP_NOT_FOUND:
                raise FileNotFoundError(path) from error
            raise

    def _api_client(self) -> DataRepositoryApiClient:
        if self._client is None:
            self._client = DataRepositoryApiClient()
        return self._client


class _DatasetWriteFile(IOBase):
    def __init__(
        self,
        client: DataRepositoryApiClient,
        repo_slug: str,
        path: str,
        invalidate_cache: Callable[[], None],
    ) -> None:
        super().__init__()
        self._client = client
        self._repo_slug = repo_slug
        self._path = path
        self._invalidate_cache = invalidate_cache
        self._discard = False
        self._file = NamedTemporaryFile(mode="w+b", delete=False)  # ruff: ignore[open-file-with-context-handler]
        self._source = Path(self._file.name)

    @override
    def writable(self) -> bool:
        return not self.closed

    @override
    def readable(self) -> bool:
        return not self.closed

    @override
    def seekable(self) -> bool:
        return not self.closed

    @override
    def write(self, data: bytes) -> int:
        return self._file.write(data)

    @override
    def read(self, size: int = -1) -> bytes:
        return self._file.read(size)

    def readinto(self, buffer: bytearray) -> int:
        return cast("int", self._file.readinto(buffer))

    @override
    def seek(self, offset: int, whence: int = 0) -> int:
        return self._file.seek(offset, whence)

    @override
    def tell(self) -> int:
        return self._file.tell()

    @override
    def flush(self) -> None:
        if self.closed:
            raise ValueError("I/O operation on closed dataset file")
        if not self._file.closed:
            self._file.flush()

    @override
    def close(self) -> None:
        if self.closed:
            return
        try:
            if not self._discard:
                self.flush()
                self._file.close()
                self._commit_staged_file()
        finally:
            IOBase.close(self)
            self._file.close()
            self._source.unlink(missing_ok=True)

    def _commit_staged_file(self) -> None:
        digest = blake3(max_threads=blake3.AUTO)
        size = 0
        with self._source.open("rb") as source:
            while chunk := source.read(1024 * 1024):
                digest.update(chunk)
                size += len(chunk)
        self._client.apply(
            self._repo_slug,
            (
                AddFileOperation(
                    path=self._path,
                    hash=digest.hexdigest(),
                    size=size,
                    source=self._source,
                ),
            ),
        )
        self._invalidate_cache()

    @override
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        self._discard = exc_type is not None
        super().__exit__(exc_type, exc_val, exc_tb)


class _ResolvedReadFile(AbstractBufferedFile):
    def __init__(
        self,
        fs: Dataset,
        path: str,
        url: str,
        *,
        size: int,
        block_size: int,
    ) -> None:
        self._url = url
        super().__init__(
            fs,
            path,
            mode="rb",
            size=size,
            block_size=block_size,
            cache_type="bytes",
        )

    @override
    def _fetch_range(self, start: int, end: int) -> bytes:
        if end <= start:
            return b""
        request = Request(  # ruff: ignore[suspicious-url-open-usage]
            self._url,
            headers={"Range": f"bytes={start}-{end - 1}"},
            method="GET",
        )
        with urlopen(request, timeout=120) as response:  # ruff: ignore[suspicious-url-open-usage]
            return response.read()


def _normalize_path(path: str, *, allow_root: bool = False) -> str:
    normalized = path.replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    normalized = normalized.strip("/")
    if allow_root and normalized in {"", "."}:
        return ""
    parts = normalized.split("/")
    if not normalized or any(part in {"", ".", ".."} for part in parts):
        raise ValueError(f"Invalid dataset path: {path!r}")
    return normalized


def _to_fsspec_entry(entry: Entry) -> dict[str, object]:
    if isinstance(entry, FileEntry):
        return {
            "name": entry.path,
            "size": entry.size_bytes,
            "type": "file",
            "blake3": entry.blake3,
        }
    if isinstance(entry, DirectoryEntry):
        return {"name": entry.path, "size": 0, "type": "directory"}
    raise TypeError(f"Unsupported repository entry: {entry!r}")


