"""Repository-backed zygo:// filesystem for workflow artifacts."""
from __future__ import annotations

import time
from typing import override

from fsspec.spec import AbstractFileSystem

from zygo.cloud.datasets.api import DataRepositoryApiClient, DataRepositoryApiError, UploadRequest
from zygo.cloud.datasets.dataset import Dataset


class _WorkflowRepositoryClient(DataRepositoryApiClient):
    def commit(self, repo_slug, request):
        # Reapply path operations against the latest head. Distinct job paths merge,
        # while writes to the same shared path use last-committer-wins semantics.
        for attempt in range(8):
            try:
                return super().commit(repo_slug, request)
            except DataRepositoryApiError as error:
                if error.status_code != 409 or attempt == 7:
                    raise
                time.sleep(min(0.05 * 2**attempt, 1))
                head = self.prepare_upload(repo_slug, UploadRequest(blobs=[]))
                request = request.model_copy(update={"expected_head_commit_id": head.head_commit_id})
        raise AssertionError("unreachable")


class ZygoFileSystem(AbstractFileSystem):
    """Resolve zygo://<repository-slug>/<path> using the repository API.

    Explicit connection kwargs override ZYGO_CLOUD_API_HOST and
    ZYGO_CLOUD_API_KEY. Successful write close uploads and commits the file.
    """

    protocol = "zygo"
    root_marker = ""
    cachable = False

    def __init__(self, **kwargs):
        api_host = kwargs.pop("api_host", None)
        api_bearer_auth = kwargs.pop("api_bearer_auth", None)
        kwargs.pop("verbose", None)
        super().__init__(**kwargs)
        self._client = _WorkflowRepositoryClient(api_host, api_bearer_auth)

    def _resolve(self, path):
        path = self._strip_protocol(path)
        repository, separator, key = path.partition("/")
        if not repository or repository in {".", ".."}:
            raise ValueError("A zygo URI must identify a repository")
        return Dataset(repository, client=self._client), key

    @override
    def _open(self, path, mode="rb", **kwargs):
        dataset, key = self._resolve(path)
        return dataset.open(key, mode, **kwargs)

    @override
    def ls(self, path, detail=True, **kwargs):
        dataset, key = self._resolve(path)
        entries = dataset.ls(key, detail=True, **kwargs)
        entries = [{**entry, "name": f"{dataset.id}/{entry['name']}"} for entry in entries]
        return entries if detail else [entry["name"] for entry in entries]

    @override
    def info(self, path, **kwargs):
        dataset, key = self._resolve(path)
        entry = dataset._info(key, **kwargs)
        return {**entry, "name": f"{dataset.id}/{entry['name']}".rstrip("/")}

    @override
    def _rm(self, path):
        dataset, key = self._resolve(path)
        dataset.rm(key)
