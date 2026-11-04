"""Shared authenticated HTTP transport for Zygo Cloud APIs."""

from __future__ import annotations

import json
import math
from typing import TYPE_CHECKING, cast
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import Request, urlopen

from zygo.cloud.api.errors import CloudApiError, error_message
from zygo.cloud.api.util import is_local_http
from zygo.cloud.env import api_host as get_api_host, api_key as get_api_key

if TYPE_CHECKING:
    from collections.abc import Iterable, Iterator, Mapping
    from typing import IO


class CloudApiClient:
    """Send authenticated API requests and unauthenticated presigned uploads."""

    def __init__(
        self,
        api_host: str | None = None,
        api_bearer_auth: str | None = None,
        *,
        timeout: float = 120,
    ) -> None:
        resolved_host = api_host if api_host is not None else get_api_host()
        resolved_key = api_bearer_auth if api_bearer_auth is not None else get_api_key()
        if not resolved_host:
            raise ValueError("api_host must be provided")
        if not resolved_key:
            raise ValueError("Set ZYGO_CLOUD_API_KEY or provide api_bearer_auth")
        if not math.isfinite(timeout) or timeout <= 0:
            raise ValueError("timeout must be positive")
        parsed_host = urlsplit(resolved_host)
        if not parsed_host.netloc or (
            parsed_host.scheme != "https" and not is_local_http(resolved_host)
        ):
            raise ValueError("api_host must use HTTPS or loopback HTTP")

        self._api_host = resolved_host.rstrip("/")
        self._api_bearer_auth = resolved_key
        self._timeout = timeout

    def post_json(self, path: str, body: Mapping[str, object]) -> dict[str, object]:
        """POST a JSON body to an API path and return its JSON object response."""

        if not path.startswith("/") or path.startswith("//"):
            raise ValueError("API path must be an absolute path on api_host")
        request = Request(  # ruff: ignore[suspicious-url-open-usage]
            f"{self._api_host}{path}",
            data=json.dumps(body).encode(),
            headers={
                "Authorization": f"Bearer {self._api_bearer_auth}",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        try:
            with cast("IO[bytes]", urlopen(request, timeout=self._timeout)) as response:  # ruff: ignore[suspicious-url-open-usage]
                payload = response.read()
        except HTTPError as error:
            raise CloudApiError(error.code, error_message(error.read())) from error

        try:
            result = json.loads(payload)
        except json.JSONDecodeError as error:
            raise CloudApiError(
                None, f"Zygo Cloud API POST {path} returned a non-JSON response"
            ) from error
        if not isinstance(result, dict):
            raise CloudApiError(
                None, f"Zygo Cloud API POST {path} returned a non-object response"
            )
        return cast("dict[str, object]", result)

    def stream_events(self, path: str) -> Iterator[str]:
        """GET an authenticated SSE stream and yield each event's data."""

        if not path.startswith("/") or path.startswith("//"):
            raise ValueError("API path must be an absolute path on api_host")
        request = Request(  # ruff: ignore[suspicious-url-open-usage]
            f"{self._api_host}{path}",
            headers={
                "Authorization": f"Bearer {self._api_bearer_auth}",
                "Accept": "text/event-stream",
            },
            method="GET",
        )
        try:
            with cast("IO[bytes]", urlopen(request, timeout=self._timeout)) as response:  # ruff: ignore[suspicious-url-open-usage]
                yield from _event_data(response)
        except HTTPError as error:
            raise CloudApiError(error.code, error_message(error.read())) from error

    def put_presigned(
        self,
        url: str,
        content: bytes,
        *,
        content_type: str | None = None,
    ) -> None:
        """PUT bytes to a presigned URL without sending Zygo API credentials."""

        parsed = urlsplit(url)
        if not parsed.netloc or (parsed.scheme != "https" and not is_local_http(url)):
            raise ValueError("Presigned upload URL must use HTTPS or loopback HTTP")
        headers = {
            "Content-Length": str(len(content)),
            "Content-Type": content_type or "application/octet-stream",
        }
        request = Request(  # ruff: ignore[suspicious-url-open-usage]
            url,
            data=content,
            headers=headers,
            method="PUT",
        )
        try:
            with cast("IO[bytes]", urlopen(request, timeout=self._timeout)) as response:  # ruff: ignore[suspicious-url-open-usage]
                response.read()
        except HTTPError as error:
            raise CloudApiError(error.code, error_message(error.read())) from error


def _event_data(lines: Iterable[bytes]) -> Iterator[str]:
    data: list[str] = []
    for raw_line in lines:
        line = raw_line.decode("utf-8").rstrip("\r\n")
        if not line:
            if data:
                yield "\n".join(data)
                data.clear()
            continue
        field, separator, value = line.partition(":")
        if field == "data":
            data.append(value.removeprefix(" ") if separator else "")
