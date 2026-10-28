"""Typed wire models for the source, image, and workflow APIs."""

from __future__ import annotations

from datetime import datetime  # noqa: TC003
from typing import Annotated, Literal
from urllib.parse import urlsplit

from pydantic import (
    BaseModel,
    ConfigDict,
    Field,
    StringConstraints,
    TypeAdapter,
    field_validator,
)

__all__ = [
    "CreateImageRequest",
    "DeployWorkflowRequest",
    "DeployWorkflowResponse",
    "ImageAwaitingUpload",
    "ImageInputUpload",
    "ImageResponse",
    "ImageStarted",
    "PythonVersion",
    "SourceAwaitingUpload",
    "SourceReady",
    "SourceRequest",
    "SourceResponse",
]

Id = Annotated[str, StringConstraints(min_length=1, max_length=255)]
Sha256 = Annotated[str, StringConstraints(pattern=r"^[a-f0-9]{64}$")]
WorkflowId = Annotated[str, StringConstraints(min_length=1, max_length=255)]
PythonVersion = Literal["3.11", "3.12", "3.13", "3.14", "3.15"]
PythonTarget = Annotated[
    str,
    StringConstraints(
        min_length=1,
        max_length=255,
        pattern=r"^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*(?::[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)?$",
    ),
]


class _ApiModel(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)


class SourceRequest(_ApiModel):
    source_hash: Sha256


class SourceAwaitingUpload(_ApiModel):
    status: Literal["awaiting_upload"]
    source_hash: Sha256
    upload_url: str
    expires_at: datetime

    @field_validator("upload_url")
    @classmethod
    def validate_upload_url(cls, value: str) -> str:
        parsed = urlsplit(value)
        if not parsed.scheme or not parsed.netloc:
            raise ValueError("upload_url must be an absolute URI")
        return value


class SourceReady(_ApiModel):
    status: Literal["ready"]
    source_id: Id
    source_hash: Sha256
    reused: bool


type SourceResponse = Annotated[
    SourceAwaitingUpload | SourceReady,
    Field(discriminator="status"),
]


class CreateImageRequest(_ApiModel):
    python_version: PythonVersion
    pyproject_hash: Sha256
    uv_lock_hash: Sha256


class ImageInputUpload(_ApiModel):
    file: Literal["pyproject", "uv_lock"]
    hash: Sha256
    upload_url: str
    expires_at: datetime

    @field_validator("upload_url")
    @classmethod
    def validate_upload_url(cls, value: str) -> str:
        parsed = urlsplit(value)
        if not parsed.scheme or not parsed.netloc:
            raise ValueError("upload_url must be an absolute URI")
        return value


class ImageAwaitingUpload(_ApiModel):
    status: Literal["awaiting_upload"]
    uploads: list[ImageInputUpload] = Field(min_length=1)


class ImageStarted(_ApiModel):
    status: Literal["started"]
    image_id: Id
    reused: bool


type ImageResponse = Annotated[
    ImageAwaitingUpload | ImageStarted,
    Field(discriminator="status"),
]


class DeployWorkflowRequest(_ApiModel):
    workflow_id: WorkflowId
    target: PythonTarget
    source_id: Id
    image_id: Id


class DeployWorkflowResponse(_ApiModel):
    id: Id
    workflow_id: WorkflowId
    version_id: Id


source_response_adapter: TypeAdapter[SourceResponse] = TypeAdapter(SourceResponse)
image_response_adapter: TypeAdapter[ImageResponse] = TypeAdapter(ImageResponse)
