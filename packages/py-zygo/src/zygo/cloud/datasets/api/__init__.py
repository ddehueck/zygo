"""Data repository API client, wire types, and errors."""

from zygo.cloud.datasets.api.client import DataRepositoryApiClient
from zygo.cloud.datasets.api.errors import DataRepositoryApiError
from zygo.cloud.datasets.api.types import (
    ApiAddFileOperation,
    ApiBlob,
    ApiOperation,
    ApiRemoveDirOperation,
    ApiRemoveFileOperation,
    CommitRequest,
    CommitResponse,
    DirectoryEntry,
    EntriesResponse,
    Entry,
    FileEntry,
    RequiredUpload,
    ResolveResponse,
    UploadRequest,
    UploadResponse,
)

__all__ = [
    "ApiAddFileOperation",
    "ApiBlob",
    "ApiOperation",
    "ApiRemoveDirOperation",
    "ApiRemoveFileOperation",
    "CommitRequest",
    "CommitResponse",
    "DataRepositoryApiClient",
    "DataRepositoryApiError",
    "DirectoryEntry",
    "EntriesResponse",
    "Entry",
    "FileEntry",
    "RequiredUpload",
    "ResolveResponse",
    "UploadRequest",
    "UploadResponse",
]
