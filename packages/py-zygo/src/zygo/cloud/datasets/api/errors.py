"""Errors raised by the data repository API client."""

from __future__ import annotations

import json

__all__ = ["DataRepositoryApiError"]


class DataRepositoryApiError(RuntimeError):
    """An HTTP error returned by the data repository API or blob upload."""

    def __init__(self, status_code: int, message: str) -> None:
        super().__init__(
            f"Data repository API request failed (HTTP {status_code}): {message}"
        )
        self.status_code = status_code
        self.message = message


def error_message(body: bytes) -> str:
    """Extract the API error field, falling back to the response body."""

    try:
        payload = json.loads(body)
    except json.JSONDecodeError:
        return body.decode(errors="replace") or "Unknown error"
    if isinstance(payload, dict) and isinstance(payload.get("error"), str):
        return payload["error"]
    return body.decode(errors="replace") or "Unknown error"
