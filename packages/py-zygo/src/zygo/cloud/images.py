"""Reusable image build API operations for Zygo Cloud."""

from __future__ import annotations

from hashlib import sha256
from typing import TYPE_CHECKING

from pydantic import ValidationError

from zygo.cloud.api import (
    CloudApiClient,
    CloudApiError,
    CreateImageRequest,
    ImageAwaitingUpload,
    ImageStarted,
)
from zygo.cloud.api.types import image_response_adapter

if TYPE_CHECKING:
    from zygo.cloud.api import PythonVersion

__all__ = ["ImageApiClient"]

_IMAGES_PATH = "/api/v1/images"


class ImageApiClient:
    """Start image builds from Python and dependency lock inputs."""

    def __init__(self, client: CloudApiClient | None = None) -> None:
        self._client = client if client is not None else CloudApiClient()

    def create(
        self,
        *,
        python_version: PythonVersion,
        pyproject_toml: bytes,
        uv_lock: bytes,
    ) -> ImageStarted:
        """Upload missing image inputs and return the started image build."""

        pyproject_hash = sha256(pyproject_toml).hexdigest()
        uv_lock_hash = sha256(uv_lock).hexdigest()
        request = CreateImageRequest(
            python_version=python_version,
            pyproject_hash=pyproject_hash,
            uv_lock_hash=uv_lock_hash,
        )
        file_inputs = {
            "pyproject": (pyproject_toml, pyproject_hash),
            "uv_lock": (uv_lock, uv_lock_hash),
        }

        response = self._create_image(request)
        if isinstance(response, ImageStarted):
            return response
        if not isinstance(response, ImageAwaitingUpload):
            raise CloudApiError(None, "Image API returned an unsupported status")

        for upload in response.uploads:
            content, content_hash = file_inputs[upload.file]
            if upload.hash != content_hash:
                raise CloudApiError(
                    None,
                    f"Image API requested an unexpected hash for {upload.file}",
                )
            self._client.put_presigned(upload.upload_url, content)

        response = self._create_image(request)
        if not isinstance(response, ImageStarted):
            raise CloudApiError(
                None,
                "Image inputs uploaded, but the image API did not return a started build",
            )
        return response

    def _create_image(
        self, request: CreateImageRequest
    ) -> ImageStarted | ImageAwaitingUpload:
        response_payload = self._client.post_json(
            _IMAGES_PATH, request.model_dump(mode="json")
        )
        try:
            return image_response_adapter.validate_python(response_payload)
        except ValidationError as error:
            raise CloudApiError(None, f"Invalid image API response: {error}") from error
