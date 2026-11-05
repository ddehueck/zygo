"""Repository-backed zygo:// filesystem for workflow artifacts."""

from __future__ import annotations

from http import HTTPStatus
import time
from typing import IO, TYPE_CHECKING, cast, override

from fsspec.spec import AbstractFileSystem

from zygo.cloud.datasets.api import (
    DataRepositoryApiClient,
    DataRepositoryApiError,
    UploadRequest,
)
from zygo.cloud.datasets.dataset import Dataset

if TYPE_CHECKING:
    from zygo.cloud.datasets.api import CommitRequest, CommitResponse


class _WorkflowRepositoryClient(DataRepositoryApiClient):
    @override
    def commit(self, repo_slug: str, request: CommitRequest) -> CommitResponse:
        # Reapply path operations against the latest head. Distinct job paths merge,
        # while writes to the same shared path use last-committer-wins semantics.
        max_attempts = 8
        for attempt in range(max_attempts):
            try:
                return super().commit(repo_slug, request)
            except DataRepositoryApiError as error:
                if (
                    error.status_code != HTTPStatus.CONFLICT
                    or attempt == max_attempts - 1
                ):
                    raise
                time.sleep(min(0.05 * 2**attempt, 1))
                head = self.prepare_upload(repo_slug, UploadRequest(blobs=[]))
                request = request.model_copy(
                    update={"expected_head_commit_id": head.head_commit_id}
                )
        raise AssertionError("unreachable")


class ZygoFileSystem(AbstractFileSystem):
    """Resolve zygo://<repository-slug>/<path> using the repository API.

    Explicit connection kwargs override ZYGO_CLOUD_API_HOST and
    ZYGO_CLOUD_API_KEY. Successful write close uploads and commits the file.
    """

    protocol = "zygo"
    root_marker = ""
    cachable = False

    def __init__(self, **kwargs: object) -> None:
        api_host = cast("str | None", kwargs.pop("api_host", None))
        api_bearer_auth = cast("str | None", kwargs.pop("api_bearer_auth", None))
        kwargs.pop("verbose", None)
        super().__init__(**kwargs)
        self._client = _WorkflowRepositoryClient(api_host, api_bearer_auth)

    def _resolve(self, path: str) -> tuple[Dataset, str]:
        path = self._strip_protocol(path)
        repository, _, key = path.partition("/")
        if not repository or repository in {".", ".."}:
            raise ValueError("A zygo URI must identify a repository")
        return Dataset(repository, client=self._client), key

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
        dataset, key = self._resolve(path)
        return dataset.open(
            key,
            mode,
            block_size=block_size,
            autocommit=autocommit,
            cache_options=cache_options,
            **kwargs,
        )

    @override
    def ls(
        self, path: str, detail: bool = True, **kwargs: object
    ) -> list[dict[str, object]] | list[str]:
        dataset, key = self._resolve(path)
        entries = cast(
            "list[dict[str, object]]", dataset.ls(key, detail=True, **kwargs)
        )
        entries = [
            {**entry, "name": f"{dataset.id}/{entry['name']}"} for entry in entries
        ]
        return entries if detail else [str(entry["name"]) for entry in entries]

    @override
    def info(self, path: str, **kwargs: object) -> dict[str, object]:
        dataset, key = self._resolve(path)
        entry = dataset._info(key, **kwargs)
        return {**entry, "name": f"{dataset.id}/{entry['name']}".rstrip("/")}

    @override
    def _rm(self, path: str) -> None:
        dataset, key = self._resolve(path)
        dataset.rm(key)
