"""Human-readable, thread-safe logging for dataset uploads."""

import logging
from math import ceil
from threading import Lock
from time import monotonic

_BYTES_PER_SIZE_UNIT = 1000
_BYTES_PER_MB = 1_000_000
_SECONDS_PER_MINUTE = 60
_SECONDS_PER_HOUR = 3600


def _format_size(size_bytes: int) -> str:
    if size_bytes < _BYTES_PER_SIZE_UNIT:
        return f"{size_bytes} B"
    size = size_bytes / _BYTES_PER_SIZE_UNIT
    for unit in ("KB", "MB", "GB"):
        if size < _BYTES_PER_SIZE_UNIT:
            return f"{size:.1f} {unit}"
        size /= _BYTES_PER_SIZE_UNIT
    return f"{size:.1f} TB"


def _format_upload_progress(uploaded_bytes: int, total_size: int) -> str:
    percentage = uploaded_bytes / total_size * 100 if total_size else 100.0
    return (
        f"{percentage:.1f}% "
        f"({_format_size(uploaded_bytes)} / {_format_size(total_size)})"
    )


def _format_duration(seconds: float) -> str:
    hours, remainder = divmod(ceil(seconds), _SECONDS_PER_HOUR)
    minutes, seconds_left = divmod(remainder, _SECONDS_PER_MINUTE)
    if hours:
        return f"{hours}h {minutes}m {seconds_left}s"
    if minutes:
        return f"{minutes}m {seconds_left}s"
    return f"{seconds_left}s"


class UploadProgressLogger:
    """Track aggregate transfer progress, average speed, and transfer-only ETA."""

    def __init__(self, logger: logging.Logger, total_size: int) -> None:
        super().__init__()
        self._logger = logger
        self._total_size = total_size
        self._uploaded_bytes = 0
        self._existing_bytes = 0
        self._started_at: float | None = None
        self._lock = Lock()

    def start(self, dataset_id: str, file_count: int) -> None:
        self._logger.info(
            "Uploading dataset %s: %d file%s (%s total)",
            dataset_id,
            file_count,
            "" if file_count == 1 else "s",
            _format_size(self._total_size),
        )

    def resume(self, existing_count: int, existing_bytes: int) -> None:
        """Initialize transfer timing after the upload session has been created."""
        with self._lock:
            self._existing_bytes = existing_bytes
            self._uploaded_bytes = existing_bytes
            self._started_at = monotonic()
            self._logger.info(
                "Already uploaded: %d file%s, %s",
                existing_count,
                "" if existing_count == 1 else "s",
                _format_upload_progress(self._uploaded_bytes, self._total_size),
            )

    def advance(self, size_bytes: int) -> None:
        """Record bytes sent by any upload worker, excluding reused bytes from speed."""
        with self._lock:
            self._uploaded_bytes += size_bytes
            elapsed = (
                monotonic() - self._started_at if self._started_at is not None else 0
            )
            transferred_bytes = self._uploaded_bytes - self._existing_bytes
            speed = transferred_bytes / elapsed if elapsed > 0 else 0
            remaining_bytes = max(0, self._total_size - self._uploaded_bytes)
            speed_text = f"{speed / _BYTES_PER_MB:.1f} MB/s" if speed else "calculating"
            eta = _format_duration(remaining_bytes / speed) if speed else "calculating"
            if not remaining_bytes:
                eta = "0s"
            self._logger.info(
                "Upload progress: %s | %s | ETA %s",
                _format_upload_progress(self._uploaded_bytes, self._total_size),
                speed_text,
                eta,
            )

    def file_uploaded(self, filename: str, size_bytes: int) -> None:
        self._logger.info("Uploaded %s (%s)", filename, _format_size(size_bytes))

    def confirming(self, dataset_version_id: str) -> None:
        self._logger.info(
            "Confirming upload for dataset version %s", dataset_version_id
        )

    def complete(self, dataset_version_id: str) -> None:
        self._logger.info(
            "Upload complete: %s (dataset version %s)",
            _format_upload_progress(self._uploaded_bytes, self._total_size),
            dataset_version_id,
        )
