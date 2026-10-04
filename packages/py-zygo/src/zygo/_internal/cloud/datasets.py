"""
zygo.cloud dataset utilities.
"""

from collections.abc import Iterator
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict
import hashlib
import json
import logging
from pathlib import Path
from typing import TYPE_CHECKING, cast
from urllib.parse import urlsplit
from urllib.request import Request, urlopen

from pydantic import TypeAdapter

from zygo import Dataset, DataUri
from zygo._internal.cloud.api import (
    ConfirmUploadDatasetSuccessRequest,
    CreateDatasetUploadSessionRequest,
    CreateDatasetUploadSessionResponse,
    UploadDatasetManifestItem,
    build_headers,
)
from zygo._internal.cloud.api_key import load_api_key
from zygo._internal.cloud.upload_progress import UploadProgressLogger
from zygo.dataset.manifest import FileManifestEntry

if TYPE_CHECKING:
    from concurrent.futures import Future
    from typing import IO

_logger = logging.getLogger(__name__)
_UPLOAD_CHUNK_SIZE = 8 * 1024 * 1024


def push_dataset[T](
    dataset: str | DataUri | Dataset[T],
    *,
    dataset_id: str,
    api_key: str | None = None,
    concurrency: int = 4,
) -> None:
    """
    Pushes dataset to a Zygo Cloud workspace.

    Parameters:
        dataset: The dataset to push. Can be a local path string, a DataUri, or a Dataset instance.
        dataset_id: A user-defined identifier e.g. "my-dataset-xyz". Uploads to the same id follows last write wins behavior.
        api_key: The API key for authentication. If omitted, tries to load from environment variable.
        concurrency: Number of parallel upload operations to use.
    """
    if concurrency < 1:
        raise ValueError("concurrency must be at least 1")

    if api_key is None:
        api_key = load_api_key()
        if api_key is None:
            raise ValueError(
                "API key must be provided via 'api_key' parameter or ZYGO_CLOUD_API_KEY environment variable"
            )

    resolved_dataset = _resolve_dataset(dataset)

    # Uploads are executed by first creating an upload session given a file manifest.
    # The server responds with a presigned URL for each file in the manifest.
    # The client then uploads the files in parallel using the provided concurrency level.
    metadata = resolved_dataset.local_content_manifest
    manifest, total_size = _build_upload_manifest(metadata)
    local_paths = {entry.filename: path for path, entry in metadata.items()}
    client = _UploadHttpClient(api_key, concurrency)
    client.upload(dataset_id, manifest, total_size, local_paths=local_paths)


def _resolve_dataset(dataset: object) -> Dataset:
    if isinstance(dataset, (str, DataUri)):
        return Dataset.open(dataset)
    if not isinstance(dataset, Dataset):
        raise TypeError(
            f"dataset must be a Dataset instance, got {type(dataset).__name__}"
        )
    return cast("Dataset[object]", dataset)


def _build_upload_manifest(
    metadata: dict[Path, FileManifestEntry],
) -> tuple[list[UploadDatasetManifestItem], int]:
    items = [
        UploadDatasetManifestItem(
            filename=entry.filename,
            file_size_bytes=entry.size_bytes,
            content_hash=entry.sha256,
        )
        for entry in sorted(metadata.values(), key=lambda entry: entry.filename)
    ]
    return items, sum(item.file_size_bytes for item in items)


class _UploadHttpClient:
    """HTTP client for uploading datasets to Zygo Cloud."""

    def __init__(self, api_key: str, concurrency: int = 4) -> None:
        super().__init__()
        if concurrency < 1:
            raise ValueError("concurrency must be at least 1")
        self.api_key = api_key
        self.concurrency = concurrency

    def upload(
        self,
        dataset_id: str,
        manifest: list[UploadDatasetManifestItem],
        total_size: int,
        *,
        local_paths: dict[str, Path],
    ) -> None:
        """Create a dataset version, upload its files, and confirm success."""
        progress = UploadProgressLogger(_logger, total_size)
        progress.start(dataset_id, len(manifest))

        session = self._create_session(dataset_id, manifest)
        pending_items, uploaded_bytes = _validate_session(session, manifest)
        progress.resume(len(manifest) - len(pending_items), uploaded_bytes)

        def chunks(item: UploadDatasetManifestItem) -> Iterator[bytes]:
            for chunk in _file_chunks(item, local_paths[item.filename]):
                yield chunk
                progress.advance(len(chunk))

        def upload_file(item: UploadDatasetManifestItem) -> None:
            # Content-Length avoids chunked transfer encoding, which S3 PUTs may reject.
            request = Request(  # noqa: S310 - Presigned URLs validated by _validate_session.
                session.presigned_urls[item.filename],
                data=chunks(item),
                headers={"Content-Length": str(item.file_size_bytes)},
                method="PUT",
            )
            with cast("IO[bytes]", urlopen(request, timeout=120)):  # noqa: S310 - Validated HTTPS or loopback HTTP.
                pass
            progress.file_uploaded(item.filename, item.file_size_bytes)

        with ThreadPoolExecutor(max_workers=self.concurrency) as executor:
            futures = {
                executor.submit(upload_file, item): item for item in pending_items
            }
            _wait_for_uploads(futures)

        progress.confirming(session.dataset_version_id)
        self._confirm_upload(session.dataset_version_id)
        progress.complete(session.dataset_version_id)

    def _create_session(
        self, dataset_id: str, manifest: list[UploadDatasetManifestItem]
    ) -> CreateDatasetUploadSessionResponse:
        session_request = CreateDatasetUploadSessionRequest(
            dataset_id=dataset_id, manifest=manifest
        )
        request = Request(  # noqa: S310 - URL uses the fixed Zygo Cloud API host.
            session_request.url(),
            data=json.dumps(asdict(session_request)).encode(),
            headers=build_headers(self.api_key),
            method=session_request.method(),
        )
        with cast("IO[bytes]", urlopen(request, timeout=30)) as response:  # noqa: S310 - Fixed API host.
            return TypeAdapter(CreateDatasetUploadSessionResponse).validate_json(
                response.read()
            )

    def _confirm_upload(self, dataset_version_id: str) -> None:
        confirmation = ConfirmUploadDatasetSuccessRequest(
            dataset_version_id=dataset_version_id
        )

        request = Request(  # noqa: S310 - URL uses the fixed Zygo Cloud API host.
            confirmation.url(),
            data=json.dumps(asdict(confirmation)).encode(),
            headers=build_headers(self.api_key),
            method=confirmation.method(),
        )
        try:
            with cast("IO[bytes]", urlopen(request, timeout=30)):  # noqa: S310 - Fixed API host.
                pass
        except OSError as error:
            raise RuntimeError(
                f"Files uploaded, but confirmation failed for dataset version {dataset_version_id}"
            ) from error


def _validate_session(
    session: CreateDatasetUploadSessionResponse,
    manifest: list[UploadDatasetManifestItem],
) -> tuple[list[UploadDatasetManifestItem], int]:
    existing = set(session.existing_files)
    pending_items = [item for item in manifest if item.filename not in existing]
    for item in pending_items:
        parsed = urlsplit(session.presigned_urls[item.filename])
        if parsed.scheme != "https" and not (
            parsed.scheme == "http"
            and parsed.hostname in {"localhost", "127.0.0.1", "::1"}
        ):
            raise ValueError("Presigned transfer URL must use HTTPS or loopback HTTP")
    uploaded_bytes = sum(
        item.file_size_bytes for item in manifest if item.filename in existing
    )
    return pending_items, uploaded_bytes


def _file_chunks(item: UploadDatasetManifestItem, path: Path) -> Iterator[bytes]:
    digest = hashlib.sha256()
    with path.open("rb") as file:
        if file.seek(0, 2) != item.file_size_bytes:
            raise ValueError(f"File size changed since manifest: {item.filename}")
        file.seek(0)
        remaining = item.file_size_bytes
        while remaining:
            chunk = file.read(min(_UPLOAD_CHUNK_SIZE, remaining))
            if not chunk:
                raise ValueError(f"File truncated during upload: {item.filename}")
            digest.update(chunk)
            yield chunk
            remaining -= len(chunk)
        if file.read(1) or digest.hexdigest() != item.content_hash:
            raise ValueError(f"File content changed since manifest: {item.filename}")


def _wait_for_uploads(
    futures: dict["Future[None]", UploadDatasetManifestItem],
) -> None:
    for future in as_completed(futures):
        item = futures[future]
        try:
            future.result()
        except Exception as error:
            for pending in futures:
                pending.cancel()
            raise RuntimeError(f"Upload failed for {item.filename}") from error
