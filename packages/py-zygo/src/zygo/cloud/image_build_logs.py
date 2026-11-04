"""Streaming image build logs from Zygo Cloud."""

from __future__ import annotations

from typing import TYPE_CHECKING
from urllib.parse import quote

from pydantic import ValidationError

from zygo.cloud.api import CloudApiClient, CloudApiError, ImageBuildLogLine

if TYPE_CHECKING:
    from collections.abc import Iterator

__all__ = ["ImageBuildLogsApiClient"]


class ImageBuildLogsApiClient:
    """Watch an image build's log events until the server closes the stream."""

    def __init__(self, client: CloudApiClient | None = None) -> None:
        self._client = client if client is not None else CloudApiClient()

    def watch(self, image_id: str) -> Iterator[ImageBuildLogLine]:
        """Yield available log events while the image build runs."""

        path = f"/api/v1/images/{quote(image_id, safe='')}/build-logs"
        for data in self._client.stream_events(path):
            try:
                yield ImageBuildLogLine.model_validate_json(data)
            except ValidationError as error:
                raise CloudApiError(
                    None, f"Invalid image build log event: {error}"
                ) from error
