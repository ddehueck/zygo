"""Errors raised by shared Zygo Cloud API clients."""

from __future__ import annotations

import json

__all__ = ["CloudApiError", "error_message"]


class CloudApiError(RuntimeError):
    """An error returned by a Zygo Cloud API or presigned upload."""

    def __init__(self, status_code: int | None, message: str) -> None:
        if status_code is None:
            super().__init__(message)
        else:
            super().__init__(
                f"Zygo Cloud request failed (HTTP {status_code}): {message}"
            )
        self.status_code = status_code
        self.message = message


def error_message(body: bytes) -> str:
    """Extract an API ``error`` field or return the response body."""

    try:
        payload = json.loads(body)
    except json.JSONDecodeError:
        return body.decode(errors="replace") or "Unknown error"
    if isinstance(payload, dict) and isinstance(payload.get("error"), str):
        return payload["error"]
    return body.decode(errors="replace") or "Unknown error"
