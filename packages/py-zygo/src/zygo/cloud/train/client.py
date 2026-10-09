"""HTTP client that sends the training API contract."""

from __future__ import annotations

import json
import time
from typing import TYPE_CHECKING, cast
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import Request, urlopen

from pydantic import TypeAdapter, ValidationError

from zygo.cloud.api import build_headers
from zygo.cloud.errors import CloudApiError, CloudConnectionError
from zygo.cloud.train.api import (
    DeclareSourceRequest,
    ModelTrainingRun,
    RunLogLine,
    SourceDeclaration,
    SourceReady,
    run_build_logs_url,
    run_logs_url,
    run_status_url,
)

if TYPE_CHECKING:
    from collections.abc import Iterator
    from typing import IO

    from zygo.cloud.train.api import SourceAwaitingUpload, StartModelTrainingRunRequest

_SERVER_ERROR = 500
_MAX_SERVER_RETRIES = 3
_RETRY_SECONDS = 2.0
_LOG_TIMEOUT_SECONDS = 300.0
_IMAGE_BUILD_ERROR = "Unable to build the image for this run."
_SOURCE = TypeAdapter(SourceDeclaration)
_RUN = TypeAdapter(ModelTrainingRun)
_LOG_LINE = TypeAdapter(RunLogLine)


class TrainingClient:
    """Post declared training requests and parse their responses."""

    def __init__(self, api_key: str) -> None:
        super().__init__()
        self._api_key = api_key

    def ensure_source(self, archive: bytes, source_hash: str) -> str:
        """Upload the archive unless this hash is already stored."""
        last_error: OSError | None = None
        for _attempt in range(_MAX_SERVER_RETRIES):
            try:
                return self._upload_if_needed(archive, source_hash)
            except OSError as error:
                last_error = error
                time.sleep(_RETRY_SECONDS)
        raise RuntimeError("Source archive upload failed") from last_error

    def start_run(self, request: StartModelTrainingRunRequest) -> ModelTrainingRun:
        """Send one runtime request and parse the run it created."""
        return self._send(request, _RUN, fatal_message=_IMAGE_BUILD_ERROR)

    def training_status(self, run_id: str) -> ModelTrainingRun:
        """Pull the current status of a model training run."""
        return _parse(_RUN, self._get_retrying(run_status_url(run_id)))

    def iter_build_logs(self, run_id: str) -> Iterator[RunLogLine]:
        """Yield image build log lines until the server closes the stream.

        Each event is one JSON log line. Lines that start with a colon are
        keepalives and are skipped. A stream that is quiet for several
        minutes raises TimeoutError so the caller can reconnect.
        """
        return self._iter_log_lines(run_build_logs_url(run_id))

    def iter_logs(self, run_id: str) -> Iterator[RunLogLine]:
        """Yield training log lines until the server closes the stream.

        Each event is one JSON log line. Lines that start with a colon are
        keepalives and are skipped. A stream that is quiet for several
        minutes raises TimeoutError so the caller can reconnect.
        """
        return self._iter_log_lines(run_logs_url(run_id))

    def _iter_log_lines(self, url: str) -> Iterator[RunLogLine]:
        response = self._open_logs(url)
        try:
            for payload in _sse_payloads(_text_lines(response)):
                try:
                    yield _LOG_LINE.validate_json(payload)
                except ValidationError as error:
                    raise ValueError(
                        "Zygo Cloud API response did not match the expected shape"
                    ) from error
        finally:
            response.close()

    def _upload_if_needed(self, archive: bytes, source_hash: str) -> str:
        declared = self._declare_source(source_hash)
        if isinstance(declared, SourceReady):
            return declared.source_id
        _put_zip(declared.upload_url, archive)
        confirmed = self._declare_source(source_hash)
        if isinstance(confirmed, SourceReady):
            return confirmed.source_id
        raise OSError("Uploaded source archive is still awaiting upload")

    def _declare_source(self, source_hash: str) -> SourceAwaitingUpload | SourceReady:
        return self._send(DeclareSourceRequest(source_hash=source_hash), _SOURCE)

    def _send[T](
        self,
        request: DeclareSourceRequest | StartModelTrainingRunRequest,
        response: TypeAdapter[T],
        *,
        fatal_message: str | None = None,
    ) -> T:
        payload = self._request_json(
            request.url(),
            method="POST",
            body=cast("dict[str, object]", request.model_dump(mode="json")),
            fatal_message=fatal_message,
        )
        return _parse(response, payload)

    def _get_retrying(self, url: str) -> dict[str, object]:
        return self._request_json(url, method="GET")

    def _request_json(
        self,
        url: str,
        *,
        method: str,
        body: dict[str, object] | None = None,
        fatal_message: str | None = None,
    ) -> dict[str, object]:
        for attempt in range(_MAX_SERVER_RETRIES):
            try:
                return self._read_json(url, method=method, body=body)
            except CloudApiError as error:
                retry = _should_retry(error, fatal_message=fatal_message)
                if attempt == _MAX_SERVER_RETRIES - 1 or not retry:
                    raise
                time.sleep(_RETRY_SECONDS)
        raise RuntimeError("Zygo Cloud API request was not sent")

    def _read_json(
        self,
        url: str,
        *,
        method: str,
        body: dict[str, object] | None = None,
    ) -> dict[str, object]:
        data = None if body is None else json.dumps(body).encode()
        request = Request(  # ruff: ignore[suspicious-url-open-usage]
            url,
            data=data,
            headers=build_headers(self._api_key),
            method=method,
        )
        try:
            with _open(request, timeout=30) as response:
                return _json_object(response.read())
        except HTTPError as error:
            raise CloudApiError(error.code, _error_message(error)) from error
        except URLError as error:
            raise _connection_error(url, error) from error

    def _open_logs(self, url: str) -> IO[bytes]:
        headers = build_headers(self._api_key)
        headers["Accept"] = "text/event-stream"
        headers.pop("Content-Type", None)
        request = Request(url, headers=headers, method="GET")  # ruff: ignore[suspicious-url-open-usage]
        for attempt in range(_MAX_SERVER_RETRIES):
            try:
                return _open(request, timeout=_LOG_TIMEOUT_SECONDS)
            except HTTPError as error:
                failure = CloudApiError(error.code, _error_message(error))
                retry = _should_retry(failure, fatal_message=None)
                if attempt == _MAX_SERVER_RETRIES - 1 or not retry:
                    raise failure from error
                time.sleep(_RETRY_SECONDS)
            except URLError as error:
                raise _connection_error(url, error) from error
        raise RuntimeError("Zygo Cloud API request was not sent")


def _open(request: Request, *, timeout: float) -> IO[bytes]:
    return cast("IO[bytes]", urlopen(request, timeout=timeout))  # ruff: ignore[suspicious-url-open-usage]


def _parse[T](response: TypeAdapter[T], payload: dict[str, object]) -> T:
    try:
        return response.validate_python(payload)
    except ValidationError as error:
        raise ValueError(
            "Zygo Cloud API response did not match the expected shape"
        ) from error


def _text_lines(response: IO[bytes]) -> Iterator[str]:
    while True:
        raw = response.readline()
        if not raw:
            return
        yield raw.decode("utf-8", errors="replace")


def _sse_payloads(lines: Iterator[str]) -> Iterator[str]:
    """Yield the data of each server-sent event.

    A blank line ends an event. Lines that start with a colon are
    keepalives. Only ``data`` fields are kept, and a trailing event is
    still yielded if the stream closes without a final blank line.
    """
    data: list[str] = []
    for raw in lines:
        line = raw.rstrip("\r\n")
        if not line:
            if data:
                yield "\n".join(data)
                data = []
            continue
        if line.startswith(":"):
            continue
        field, separator, value = line.partition(":")
        if not separator or field != "data":
            continue
        data.append(value[1:] if value.startswith(" ") else value)
    if data:
        yield "\n".join(data)


def _should_retry(error: CloudApiError, *, fatal_message: str | None) -> bool:
    if error.status != _SERVER_ERROR:
        return False
    return error.message != fatal_message


def _put_zip(url: str, archive: bytes) -> None:
    _require_transfer_url(url)
    request = Request(  # ruff: ignore[suspicious-url-open-usage]
        url,
        data=archive,
        headers={"Content-Type": "application/zip"},
        method="PUT",
    )
    try:
        with cast("IO[bytes]", urlopen(request, timeout=120)) as response:  # ruff: ignore[suspicious-url-open-usage]
            response.read()
    except HTTPError as error:
        error.read()
        raise


def _require_transfer_url(url: str) -> None:
    parsed = urlsplit(url)
    if parsed.scheme == "https":
        return
    if parsed.scheme == "http" and parsed.hostname in {"localhost", "127.0.0.1", "::1"}:
        return
    raise ValueError("Upload URL must use HTTPS or loopback HTTP")


def _json_object(raw: bytes) -> dict[str, object]:
    try:
        value: object = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ValueError(
            "Zygo Cloud API returned a response that is not JSON"
        ) from error
    if not isinstance(value, dict):
        raise ValueError("Zygo Cloud API returned a response that is not a JSON object")
    return cast("dict[str, object]", value)


def _connection_error(url: str, error: URLError) -> CloudConnectionError:
    parsed = urlsplit(url)
    host = f"{parsed.scheme}://{parsed.netloc}" if parsed.netloc else url
    reason = error.reason
    detail = str(reason) if reason is not None else str(error)
    return CloudConnectionError(host, detail)


def _error_message(error: HTTPError) -> str:
    raw = error.read()
    try:
        payload: object = json.loads(raw)
    except json.JSONDecodeError:
        return str(error.reason)
    if isinstance(payload, dict):
        message = cast("dict[str, object]", payload).get("error")
        if isinstance(message, str) and message:
            return message
    return str(error.reason)
