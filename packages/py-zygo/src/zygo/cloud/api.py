"""Cloud API configuration constants."""

from dataclasses import dataclass
import os

ZYGO_CLOUD_HOST = os.environ.get("ZYGO_CLOUD_HOST", "https://zygocloud.com").rstrip("/")


@dataclass
class UploadDatasetManifestItem:
    """Represents a single file entry in the dataset upload manifest."""

    filename: str  # Dataset-relative POSIX path
    file_size_bytes: int
    content_hash: str  # SHA-256 of the complete serialized file


@dataclass(frozen=True)
class CreateDatasetUploadSessionRequest:
    """Request body for creating a dataset upload session."""

    dataset_id: str  # A user-defined identifier e.g. "my-dataset-xyz"
    manifest: list[UploadDatasetManifestItem]

    @staticmethod
    def url() -> str:
        return f"{ZYGO_CLOUD_HOST}/api/v1/datasets/upload"

    @staticmethod
    def method() -> str:
        return "POST"


@dataclass()
class CreateDatasetUploadSessionResponse:
    """
    Response body for creating a dataset upload session.

    This may return fewer presigned URLs than requested if some files already exist.
    """

    dataset_version_id: str  # Unique identifier for this upload session
    presigned_urls: dict[
        str, str
    ]  # filename -> presigned URL, does not include existing files
    existing_files: list[str]  # Filenames that already exist and don't need uploading
    expires_at: str  # ISO 8601 timestamp indicating when the upload session expires


@dataclass
class ConfirmUploadDatasetSuccessRequest:
    """Request body for confirming successful dataset upload."""

    dataset_version_id: str

    @staticmethod
    def url() -> str:
        return f"{ZYGO_CLOUD_HOST}/api/v1/datasets/confirm-upload"

    @staticmethod
    def method() -> str:
        return "POST"


def build_headers(token: str) -> dict[str, str]:
    """Build HTTP headers for cloud API requests."""
    return {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}",
    }
