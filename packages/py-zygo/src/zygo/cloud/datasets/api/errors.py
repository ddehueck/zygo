"""Errors raised by the data repository API client."""

from __future__ import annotations

from zygo.cloud.api.errors import error_message

__all__ = ["DataRepositoryApiError", "error_message"]


class DataRepositoryApiError(RuntimeError):
    """An HTTP error returned by the data repository API or blob upload."""

    def __init__(self, status_code: int, message: str) -> None:
        super().__init__(
            f"Data repository API request failed (HTTP {status_code}): {message}"
        )
        self.status_code = status_code
        self.message = message
