"""Pydantic models for the data repository HTTP API."""

from __future__ import annotations

from typing import Annotated, Literal
from urllib.parse import urlsplit

from pydantic import BaseModel, ConfigDict, Field, StringConstraints, field_validator

__all__ = [
    "ApiAddFileOperation",
    "ApiBlob",
    "ApiOperation",
    "ApiRemoveDirOperation",
    "ApiRemoveFileOperation",
    "CommitRequest",
    "CommitResponse",
    "RequiredUpload",
    "UploadRequest",
    "UploadResponse",
]

Blake3 = Annotated[str, StringConstraints(pattern=r"^[a-f0-9]{64}$")]
CommitId = Annotated[str, StringConstraints(pattern=r"^drc_[0-9A-HJKMNP-TV-Z]{26}$")]
RepositoryPath = Annotated[
    str,
    StringConstraints(min_length=1, max_length=1024, pattern=r"^(?!.*\/$).+$"),
]


class _ApiModel(BaseModel):
    model_config = ConfigDict(strict=True, extra="forbid", frozen=True)


class ApiBlob(_ApiModel):
    blake3: Blake3
    size_bytes: int = Field(ge=0)


class UploadRequest(_ApiModel):
    blobs: list[ApiBlob]


class RequiredUpload(_ApiModel):
    blake3: Blake3
    upload_url: str

    @field_validator("upload_url")
    @classmethod
    def validate_upload_url(cls, value: str) -> str:
        parsed = urlsplit(value)
        if not parsed.scheme or not parsed.netloc:
            raise ValueError("upload_url must be an absolute URI")
        return value


class UploadResponse(_ApiModel):
    head_commit_id: CommitId | None
    uploads: list[RequiredUpload]


class ApiAddFileOperation(_ApiModel):
    type: Literal["add_file"]
    path: RepositoryPath
    blake3: Blake3
    size_bytes: int = Field(ge=0)


class ApiRemoveFileOperation(_ApiModel):
    type: Literal["remove_file"]
    path: RepositoryPath


class ApiRemoveDirOperation(_ApiModel):
    type: Literal["remove_dir"]
    path: RepositoryPath


type ApiOperation = Annotated[
    ApiAddFileOperation | ApiRemoveFileOperation | ApiRemoveDirOperation,
    Field(discriminator="type"),
]


class CommitRequest(_ApiModel):
    expected_head_commit_id: CommitId | None
    operations: list[ApiOperation] = Field(min_length=1)


class CommitResponse(_ApiModel):
    commit_id: CommitId
