"""HTTP client for data repository upload preparation and commits."""

from __future__ import annotations

from contextlib import nullcontext
from typing import TYPE_CHECKING, Literal
from urllib.error import HTTPError
from urllib.parse import quote, urlencode, urlsplit
from urllib.request import Request, urlopen

from blake3 import blake3

from zygo.cloud.datasets.api.errors import DataRepositoryApiError, error_message
from zygo.cloud.datasets.api.types import (
    ApiAddFileOperation,
    ApiBlob,
    ApiRemoveDirOperation,
    ApiRemoveFileOperation,
    CommitRequest,
    CommitResponse,
    EntriesResponse,
    ResolveResponse,
    UploadRequest,
    UploadResponse,
)
from zygo.cloud.datasets.api.util import is_local_http
from zygo.cloud.datasets.operations import AddFileOperation, RemoveFileOperation
from zygo.cloud.env import api_host as get_api_host, api_key as get_api_key

if TYPE_CHECKING:
    from collections.abc import Callable, Iterator, Mapping

    from pydantic import BaseModel

    from zygo.cloud.datasets.api.types import ApiOperation
    from zygo.cloud.datasets.operations import Operation
    from zygo.cloud.datasets.progress import ProgressTracker

__all__ = ["DataRepositoryApiClient"]


class DataRepositoryApiClient:
    """Read, upload, and commit changes to data repositories."""

    def __init__(
        self,
        api_host: str | None = None,
        api_bearer_auth: str | None = None,
        *,
        timeout: float = 120,
        upload_batch_size: int = 50,
    ) -> None:
        resolved_host = api_host if api_host is not None else get_api_host()
        resolved_key = api_bearer_auth if api_bearer_auth is not None else get_api_key()
        if not resolved_host:
            raise ValueError("api_host must be provided")
        if not resolved_key:
            raise ValueError("Set ZYGO_CLOUD_API_KEY or provide api_bearer_auth")
        if (
            isinstance(upload_batch_size, bool)
            or not isinstance(upload_batch_size, int)
            or upload_batch_size <= 0
        ):
            raise ValueError("upload_batch_size must be a positive integer")

        if urlsplit(resolved_host).scheme != "https" and not is_local_http(
            resolved_host
        ):
            raise ValueError("api_host must use HTTPS or loopback HTTP")

        self._api_host = resolved_host.rstrip("/")
        self._api_bearer_auth = resolved_key
        self._timeout = timeout
        self._upload_batch_size = upload_batch_size

    def prepare_upload(self, repo_slug: str, request: UploadRequest) -> UploadResponse:
        """Ask which content blobs are missing and read the current repository head."""

        response = self._post(repo_slug, "upload", request)
        return UploadResponse.model_validate_json(response)

    def commit(self, repo_slug: str, request: CommitRequest) -> CommitResponse:
        """Commit ordered repository path operations using the expected head."""

        response = self._post(repo_slug, "commit", request)
        return CommitResponse.model_validate_json(response)

    def list_entries(
        self,
        repo_slug: str,
        *,
        path: str | None = None,
        revision: str | None = None,
        cursor: str | None = None,
        limit: int = 1000,
    ) -> EntriesResponse:
        """List one directory level from a repository revision."""

        query: dict[str, str | int] = {"limit": limit}
        if path is not None:
            query["path"] = path
        if revision is not None:
            query["revision"] = revision
        if cursor is not None:
            query["cursor"] = cursor
        response = self._get(repo_slug, "entries", query=query)
        return EntriesResponse.model_validate_json(response)

    def resolve(
        self, repo_slug: str, path: str, *, revision: str | None = None
    ) -> ResolveResponse:
        """Resolve a repository file to a short-lived download URL."""

        query = {"path": path}
        if revision is not None:
            query["revision"] = revision
        response = self._get(repo_slug, "resolve", query=query)
        return ResolveResponse.model_validate_json(response)

    def apply(
        self,
        repo_slug: str,
        operations: tuple[Operation, ...],
        *,
        progress: ProgressTracker | None = None,
    ) -> CommitResponse | None:
        """Upload missing file blobs, then atomically commit the operations."""

        if not operations:
            return None

        additions: dict[str, AddFileOperation] = {}
        blobs_by_hash: dict[str, ApiBlob] = {}
        for operation in operations:
            if not isinstance(operation, AddFileOperation):
                continue
            blob = ApiBlob(blake3=operation.hash, size_bytes=operation.size)
            previous_blob = blobs_by_hash.get(operation.hash)
            if previous_blob is not None and previous_blob.size_bytes != operation.size:
                raise ValueError(f"Hash {operation.hash} has conflicting file sizes")
            blobs_by_hash[operation.hash] = blob
            additions.setdefault(operation.hash, operation)

        upload_stage = progress.stage("Upload data") if progress else nullcontext()
        with upload_stage:
            if progress:
                progress.configure_upload_batches(len(blobs_by_hash))

            blob_items = list(blobs_by_hash.items())
            batch_count = max(
                1,
                (len(blob_items) + self._upload_batch_size - 1)
                // self._upload_batch_size,
            )
            head_commit_id: str | None = None
            for batch_index in range(batch_count):
                start = batch_index * self._upload_batch_size
                batch = blob_items[start : start + self._upload_batch_size]
                batch_additions = {
                    blob_hash: additions[blob_hash] for blob_hash, _ in batch
                }
                upload_response = self.prepare_upload(
                    repo_slug,
                    UploadRequest(blobs=[blob for _, blob in batch]),
                )
                if batch_index == 0:
                    head_commit_id = upload_response.head_commit_id

                uploads = []
                for upload in upload_response.uploads:
                    operation = batch_additions.get(upload.blake3)
                    if operation is None:
                        raise RuntimeError(
                            "Upload response requested an unknown blob hash: "
                            f"{upload.blake3}"
                        )
                    uploads.append((upload, operation))

                if progress:
                    progress.record_uploads_already_present(
                        len(batch) - len(uploads)
                    )

                for upload, operation in uploads:
                    file_progress = (
                        progress.upload_file(operation.path, operation.size)
                        if progress
                        else nullcontext(lambda _: None)
                    )
                    with file_progress as on_progress:
                        self._upload_blob(
                            upload.upload_url, operation, on_progress=on_progress
                        )

        request = CommitRequest(
            expected_head_commit_id=head_commit_id,
            operations=[_to_api_operation(operation) for operation in operations],
        )
        commit_stage = progress.stage("Commit changes") if progress else nullcontext()
        with commit_stage:
            return self.commit(repo_slug, request)

    def _post(
        self, repo_slug: str, action: Literal["upload", "commit"], body: BaseModel
    ) -> bytes:
        slug = quote(repo_slug, safe="")
        request = Request(  # ruff: ignore[suspicious-url-open-usage]
            f"{self._api_host}/api/data-repositories/{slug}/{action}",
            data=body.model_dump_json().encode(),
            headers={
                "Authorization": f"Bearer {self._api_bearer_auth}",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        try:
            with urlopen(request, timeout=self._timeout) as response:  # ruff: ignore[suspicious-url-open-usage]
                return response.read()
        except HTTPError as error:
            raise DataRepositoryApiError(
                error.code, error_message(error.read())
            ) from error

    def _get(
        self,
        repo_slug: str,
        action: Literal["entries", "resolve"],
        *,
        query: Mapping[str, str | int],
    ) -> bytes:
        slug = quote(repo_slug, safe="")
        query_string = urlencode(query)
        request = Request(  # ruff: ignore[suspicious-url-open-usage]
            f"{self._api_host}/api/data-repositories/{slug}/{action}?{query_string}",
            headers={"Authorization": f"Bearer {self._api_bearer_auth}"},
            method="GET",
        )
        try:
            with urlopen(request, timeout=self._timeout) as response:  # ruff: ignore[suspicious-url-open-usage]
                return response.read()
        except HTTPError as error:
            raise DataRepositoryApiError(
                error.code, error_message(error.read())
            ) from error

    def _upload_blob(
        self,
        url: str,
        operation: AddFileOperation,
        *,
        on_progress: Callable[[int], None] | None = None,
    ) -> None:
        if urlsplit(url).scheme != "https" and not is_local_http(url):
            raise ValueError("Presigned upload URL must use HTTPS or loopback HTTP")

        with operation.source.open("rb") as source_file:
            digest = blake3(max_threads=blake3.AUTO)
            size = 0
            while chunk := source_file.read(1024 * 1024):
                digest.update(chunk)
                size += len(chunk)
            if digest.hexdigest() != operation.hash or size != operation.size:
                raise RuntimeError(
                    f"Local file changed after it was hashed: {operation.source}"
                )

            source_file.seek(0)
            upload_digest = blake3(max_threads=blake3.AUTO)
            uploaded_size = 0

            def chunks() -> Iterator[bytes]:
                nonlocal uploaded_size
                while chunk := source_file.read(1024 * 1024):
                    upload_digest.update(chunk)
                    uploaded_size += len(chunk)
                    yield chunk
                    if on_progress is not None:
                        on_progress(len(chunk))

            request = Request(  # ruff: ignore[suspicious-url-open-usage]
                url,
                data=chunks(),
                headers={"Content-Length": str(operation.size)},
                method="PUT",
            )
            try:
                with urlopen(request, timeout=self._timeout) as response:  # ruff: ignore[suspicious-url-open-usage]
                    response.read()
            except HTTPError as error:
                raise DataRepositoryApiError(
                    error.code, error_message(error.read())
                ) from error

            if (
                upload_digest.hexdigest() != operation.hash
                or uploaded_size != operation.size
            ):
                raise RuntimeError(
                    f"Local file changed after it was hashed: {operation.source}"
                )


def _to_api_operation(operation: Operation) -> ApiOperation:
    if isinstance(operation, AddFileOperation):
        return ApiAddFileOperation(
            type="add_file",
            path=operation.path,
            blake3=operation.hash,
            size_bytes=operation.size,
        )
    if isinstance(operation, RemoveFileOperation):
        return ApiRemoveFileOperation(type="remove_file", path=operation.path)
    return ApiRemoveDirOperation(type="remove_dir", path=operation.path)
