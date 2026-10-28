"""Reusable source archive API operations for Zygo Cloud."""

from __future__ import annotations

from hashlib import sha256
from typing import TYPE_CHECKING

from pydantic import ValidationError

from zygo.cloud.api import (
    CloudApiClient,
    CloudApiError,
    SourceAwaitingUpload,
    SourceReady,
    SourceRequest,
    SourceResponse,
)
from zygo.cloud.api.types import source_response_adapter

if TYPE_CHECKING:
    from zygo.cloud.api import SourceResponse

__all__ = ["SourceApiClient"]

_SOURCE_PATH = "/api/v1/source"
_MAX_UPLOAD_ROUNDS = 3


class SourceApiClient:
    """Resolve and upload reusable ZIP source archives."""

    def __init__(self, client: CloudApiClient | None = None) -> None:
        self._client = client if client is not None else CloudApiClient()

    def resolve(self, source_hash: str) -> SourceResponse:
        """Resolve a source by its lowercase SHA-256 digest."""

        request = SourceRequest(source_hash=source_hash)
        response = self._client.post_json(_SOURCE_PATH, request.model_dump(mode="json"))
        try:
            return source_response_adapter.validate_python(response)
        except ValidationError as error:
            raise CloudApiError(
                None, f"Invalid source API response: {error}"
            ) from error

    def ensure_source(self, archive: bytes) -> SourceReady:
        """Resolve a source archive, uploading it when the server requests it."""

        source_hash = sha256(archive).hexdigest()
        for attempt in range(_MAX_UPLOAD_ROUNDS + 1):
            response = self.resolve(source_hash)
            if isinstance(response, SourceReady):
                if response.source_hash != source_hash:
                    raise CloudApiError(
                        None,
                        "Source API returned a source hash that does not match the archive",
                    )
                return response

            if not isinstance(response, SourceAwaitingUpload):
                raise CloudApiError(None, "Source API returned an unsupported status")
            if response.source_hash != source_hash:
                raise CloudApiError(
                    None,
                    "Source API requested an upload for a different source hash",
                )
            if attempt == _MAX_UPLOAD_ROUNDS:
                break
            self._client.put_presigned(
                response.upload_url,
                archive,
                content_type="application/zip",
            )

        raise CloudApiError(
            None,
            "Source upload completed, but the source API did not return a ready source",
        )
