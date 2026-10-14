"""HTTP client for data repository upload preparation and commits."""

from __future__ import annotations

import hashlib
from typing import TYPE_CHECKING, Literal
from urllib.error import HTTPError
from urllib.parse import quote, urlsplit
from urllib.request import Request, urlopen

from zygo.cloud.datasets.api.errors import DataRepositoryApiError, error_message
from zygo.cloud.datasets.api.types import (
    ApiAddFileOperation,
    ApiBlob,
    ApiRemoveDirOperation,
    ApiRemoveFileOperation,
    CommitRequest,
    CommitResponse,
    UploadRequest,
    UploadResponse,
)
from zygo.cloud.datasets.api.util import is_local_http
from zygo.cloud.datasets.operations import AddFileOperation, RemoveFileOperation
from zygo.cloud.env import api_host as get_api_host, api_key as get_api_key

if TYPE_CHECKING:
    from pydantic import BaseModel

    from zygo.cloud.datasets.api.types import ApiOperation
    from zygo.cloud.datasets.operations import Operation

__all__ = ["DataRepositoryApiClient"]


class DataRepositoryApiClient:
    """Call the data repository prepare-upload and commit endpoints."""

    def __init__(
        self,
        api_host: str | None = None,
        api_bearer_auth: str | None = None,
        *,
        timeout: float = 30,
    ) -> None:
        resolved_host = api_host if api_host is not None else get_api_host()
        resolved_key = api_bearer_auth if api_bearer_auth is not None else get_api_key()
        if not resolved_host:
            raise ValueError("api_host must be provided")
        if not resolved_key:
            raise ValueError("Set ZYGO_CLOUD_API_KEY or provide api_bearer_auth")

        if urlsplit(resolved_host).scheme != "https" and not is_local_http(
            resolved_host
        ):
            raise ValueError("api_host must use HTTPS or loopback HTTP")

        self._api_host = resolved_host.rstrip("/")
        self._api_bearer_auth = resolved_key
        self._timeout = timeout

    def prepare_upload(self, repo_slug: str, request: UploadRequest) -> UploadResponse:
        """Ask which content blobs are missing and read the current repository head."""

        response = self._post(repo_slug, "upload", request)
        return UploadResponse.model_validate_json(response)

    def commit(self, repo_slug: str, request: CommitRequest) -> CommitResponse:
        """Commit ordered repository path operations using the expected head."""

        response = self._post(repo_slug, "commit", request)
        return CommitResponse.model_validate_json(response)

    def apply(
        self, repo_slug: str, operations: tuple[Operation, ...]
    ) -> CommitResponse | None:
        """Upload missing file blobs, then atomically commit the operations."""

        if not operations:
            return None

        additions: dict[str, AddFileOperation] = {}
        blobs_by_hash: dict[str, ApiBlob] = {}
        for operation in operations:
            if not isinstance(operation, AddFileOperation):
                continue
            blob = ApiBlob(sha256=operation.hash, size_bytes=operation.size)
            previous_blob = blobs_by_hash.get(operation.hash)
            if previous_blob is not None and previous_blob.size_bytes != operation.size:
                raise ValueError(f"Hash {operation.hash} has conflicting file sizes")
            blobs_by_hash[operation.hash] = blob
            additions.setdefault(operation.hash, operation)

        upload_response = self.prepare_upload(
            repo_slug,
            UploadRequest(blobs=list(blobs_by_hash.values())),
        )
        for upload in upload_response.uploads:
            operation = additions.get(upload.sha256)
            if operation is None:
                raise RuntimeError(
                    f"Upload response requested an unknown blob hash: {upload.sha256}"
                )
            # TODO: parallelize uploads?
            self._upload_blob(upload.upload_url, operation)

        request = CommitRequest(
            expected_head_commit_id=upload_response.head_commit_id,
            operations=[_to_api_operation(operation) for operation in operations],
        )
        return self.commit(repo_slug, request)

    def _post(
        self, repo_slug: str, action: Literal["upload", "commit"], body: BaseModel
    ) -> bytes:
        slug = quote(repo_slug, safe="")
        request = Request(  # noqa: S310
            f"{self._api_host}/api/data-repositories/{slug}/{action}",
            data=body.model_dump_json().encode(),
            headers={
                "Authorization": f"Bearer {self._api_bearer_auth}",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        try:
            with urlopen(request, timeout=self._timeout) as response:  # noqa: S310
                return response.read()
        except HTTPError as error:
            raise DataRepositoryApiError(
                error.code, error_message(error.read())
            ) from error

    def _upload_blob(self, url: str, operation: AddFileOperation) -> None:
        if urlsplit(url).scheme != "https" and not is_local_http(url):
            raise ValueError("Presigned upload URL must use HTTPS or loopback HTTP")

        data = operation.source.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        if digest != operation.hash or len(data) != operation.size:
            raise RuntimeError(
                f"Local file changed after it was hashed: {operation.source}"
            )
        request = Request(  # noqa: S310
            url,
            data=data,
            headers={"Content-Length": str(len(data))},
            method="PUT",
        )
        try:
            with urlopen(request, timeout=self._timeout) as response:  # noqa: S310
                response.read()
        except HTTPError as error:
            raise DataRepositoryApiError(
                error.code, error_message(error.read())
            ) from error


def _to_api_operation(operation: Operation) -> ApiOperation:
    if isinstance(operation, AddFileOperation):
        return ApiAddFileOperation(
            type="add_file",
            path=operation.path,
            sha256=operation.hash,
            size_bytes=operation.size,
        )
    if isinstance(operation, RemoveFileOperation):
        return ApiRemoveFileOperation(type="remove_file", path=operation.path)
    return ApiRemoveDirOperation(type="remove_dir", path=operation.path)
