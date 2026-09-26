"""IPC message transport: stdio (default) or HTTP override."""

from __future__ import annotations

import os
from pathlib import Path
import sys
import time
from typing import TYPE_CHECKING, Protocol, TextIO, cast, overload
import urllib.error
import urllib.request
from uuid import uuid4

from zygo.cli.v0.types import (
    STDOUT_IPC_PREFIX,
    HttpIPCMessage,
    serialize_http_ipc_message,
    serialize_ipc_message,
)

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence
    from http.client import HTTPResponse

    from zygo.cli.v0.types import IpcMessage


def _as_batch(messages: IpcMessage | Sequence[IpcMessage]) -> list[IpcMessage]:
    if isinstance(messages, (list, tuple)):
        return list(messages)
    return [cast("IpcMessage", messages)]


class IpcTransport(Protocol):
    @overload
    def emit(self, messages: IpcMessage) -> None: ...
    @overload
    def emit(self, messages: Sequence[IpcMessage]) -> None: ...
    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None: ...


class StdioTransport:
    """Write one flushed, parseable IPC message line per message to stdout.

    A closed IPC reader leaves Python's stdout buffer pointing at a broken
    pipe. Replace it with ``os.devnull`` after handling that condition so
    interpreter shutdown does not report a second flush error.
    """

    @overload
    def emit(self, messages: IpcMessage) -> None: ...
    @overload
    def emit(self, messages: Sequence[IpcMessage]) -> None: ...
    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None:  # ruff: ignore[no-self-use]
        batch = _as_batch(messages)
        if not batch:
            return
        stdout: TextIO = sys.stdout
        try:
            for message in batch:
                serialized = f"{STDOUT_IPC_PREFIX}{serialize_ipc_message(message)}"
                stdout.write(f"{serialized}\n")
            stdout.flush()
        except BrokenPipeError:
            # Keep stdout open for interpreter shutdown, but detach it from the pipe.
            sys.stdout = Path(os.devnull).open("w", encoding="utf-8")  # ruff: ignore[open-file-with-context-handler]


class HttpTransport:
    """POST a batch of IPC messages as JSON to a configured endpoint with fixed retry delays."""

    def __init__(  # ruff: ignore[too-many-arguments]
        self,
        *,
        url: str,
        max_retries: int,
        retry_interval: float,
        timeout: float,
        workflow_run_id: str,
        job_run_id: str,
        headers: Mapping[str, str] | None = None,
    ) -> None:
        super().__init__()
        if not url.startswith(("http://", "https://")):
            raise ValueError(
                f"HTTP IPC URL must start with http:// or https://, got: {url!r}"
            )
        self._url = url
        self._max_retries = max_retries
        self._retry_interval = retry_interval
        self._timeout = timeout
        self._workflow_run_id = workflow_run_id
        self._job_run_id = job_run_id
        self._headers = {
            "Content-Type": "application/json",
            **(dict(headers) if headers is not None else {}),
        }

    @overload
    def emit(self, messages: IpcMessage) -> None: ...
    @overload
    def emit(self, messages: Sequence[IpcMessage]) -> None: ...
    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None:
        batch = _as_batch(messages)
        if not batch:
            return
        body = serialize_http_ipc_message(
            HttpIPCMessage(
                id=str(uuid4()),
                workflow_run_id=self._workflow_run_id,
                job_run_id=self._job_run_id,
                messages=batch,
            )
        ).encode("utf-8")
        last_error: Exception | None = None

        for attempt in range(self._max_retries + 1):
            last_error = self._post(body)
            if last_error is None:
                return
            if attempt < self._max_retries:
                time.sleep(self._retry_interval)

        raise RuntimeError(
            f"HTTP IPC emit failed after {self._max_retries + 1} attempt(s)"
        ) from last_error

    def _post(self, body: bytes) -> Exception | None:
        request = urllib.request.Request(  # ruff: ignore[suspicious-url-open-usage]
            self._url,
            data=body,
            headers=self._headers,
            method="POST",
        )
        try:
            response = cast(
                "HTTPResponse",
                urllib.request.urlopen(request, timeout=self._timeout),  # ruff: ignore[suspicious-url-open-usage]
            )
            with response:
                if response.status // 100 == 2:  # ruff: ignore[magic-value-comparison]
                    return None
                return RuntimeError(
                    f"HTTP IPC emit failed with status {response.status}"
                )
        except urllib.error.HTTPError as error:
            return error
        except (urllib.error.URLError, TimeoutError) as error:
            return error
