"""Unit tests for IPC CLI argument parsing helpers."""

from __future__ import annotations

import pytest

from zygo._internal.ipc.v0.__main__ import (
    IpcArguments,
    _build_transport,
    _parse_http_headers,
    build_parser,
)
from zygo._internal.ipc.v0.transport import HttpTransport, StdioTransport


def test_parse_http_headers_empty() -> None:
    assert _parse_http_headers([]) == {}


def test_parse_http_headers_single_and_repeatable() -> None:
    assert _parse_http_headers(["Authorization: Bearer tok"]) == {
        "Authorization": "Bearer tok",
    }
    assert _parse_http_headers(
        ["Authorization: Bearer tok", "X-Custom: a:b:c"]
    ) == {
        "Authorization": "Bearer tok",
        "X-Custom": "a:b:c",
    }


def test_parse_http_headers_strips_whitespace() -> None:
    assert _parse_http_headers(["  X-Foo  :  bar  "]) == {"X-Foo": "bar"}


def test_parse_http_headers_rejects_invalid() -> None:
    with pytest.raises(ValueError, match="expected NAME:VALUE"):
        _parse_http_headers(["nocolon"])
    with pytest.raises(ValueError, match="header name must not be empty"):
        _parse_http_headers([": value"])


def test_build_transport_defaults_to_stdio() -> None:
    assert isinstance(_build_transport(IpcArguments()), StdioTransport)


def test_build_transport_http_requires_host() -> None:
    args = IpcArguments()
    args.use_http = True
    with pytest.raises(ValueError, match="--http-host is required"):
        _build_transport(args)


def test_build_transport_http_success() -> None:
    args = IpcArguments()
    args.use_http = True
    args.http_host = "https://example.com/api/events"
    args.http_max_retries = 2
    args.http_retry_interval = 0.5
    args.http_header = ["Authorization: Bearer secret"]

    transport = _build_transport(args)

    assert isinstance(transport, HttpTransport)
    assert transport._url == "https://example.com/api/events"
    assert transport._max_retries == 2
    assert transport._retry_interval == 0.5
    assert transport._headers["Authorization"] == "Bearer secret"
    assert transport._headers["Content-Type"] == "application/json"


def test_build_transport_rejects_negative_retry_settings() -> None:
    args = IpcArguments()
    args.use_http = True
    args.http_host = "https://example.com/e"

    args.http_max_retries = -1
    with pytest.raises(ValueError, match="--http-max-retries"):
        _build_transport(args)

    args.http_max_retries = 0
    args.http_retry_interval = -0.1
    with pytest.raises(ValueError, match="--http-retry-interval"):
        _build_transport(args)


def test_parser_run_stdio_defaults() -> None:
    args = build_parser().parse_args(
        ["run", "pkg.mod:workflow", "--args", "{}"],
        namespace=IpcArguments(),
    )
    assert args.command == "run"
    assert args.target == "pkg.mod:workflow"
    assert args.args == "{}"
    assert args.use_http is False
    assert args.http_host is None
    assert args.http_max_retries == 3
    assert args.http_retry_interval == 1.0
    assert args.http_header == []


def test_parser_run_http_flags() -> None:
    args = build_parser().parse_args(
        [
            "run",
            "pkg.mod:workflow",
            "--args",
            "{}",
            "--use-http",
            "--http-host",
            "http://myservice.com/api/events",
            "--http-max-retries",
            "5",
            "--http-retry-interval",
            "2.5",
            "--http-header",
            "Authorization: Bearer tok",
            "--http-header",
            "X-Request-Id: 1",
        ],
        namespace=IpcArguments(),
    )
    assert args.use_http is True
    assert args.http_host == "http://myservice.com/api/events"
    assert args.http_max_retries == 5
    assert args.http_retry_interval == 2.5
    assert args.http_header == [
        "Authorization: Bearer tok",
        "X-Request-Id: 1",
    ]
