"""Unit tests for IPC CLI argument parsing helpers."""

from __future__ import annotations

import argparse
from dataclasses import asdict
import json
from typing import TYPE_CHECKING, Self, cast
import urllib.error
import urllib.request

import pytest

if TYPE_CHECKING:
    from http.client import HTTPResponse
    from pathlib import Path
    from types import TracebackType

    from zygo.cli.v0.transport import IpcTransport
    from zygo.cli.v0.types import StoreConfig

import zygo.cli.v0.__main__ as cli_module
from zygo.cli.v0.__main__ import IpcArguments, build_parser
from zygo.cli.v0.arguments import parse_http_config, parse_job_args, parse_store_config
from zygo.cli.v0.transport import HttpTransport, StdioTransport
from zygo.cli.v0.types import (
    ChannelItemInserted,
    DataReferenceCreated,
    JobFailed,
    JobRunArgs,
    TagInserted,
    serialize_ipc_message,
)
from zygo.store import DataUri


class _SuccessfulResponse:
    status = 200

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        return None


def test_generated_protocol_models_keep_wire_format() -> None:
    args = JobRunArgs("job", "file:///input", "workflow-run", "job-run")
    assert asdict(args) == {
        "job_id": "job",
        "data_reference_uri": "file:///input",
        "workflow_run_id": "workflow-run",
        "job_run_id": "job-run",
    }

    messages = [
        (
            DataReferenceCreated("data_reference_created", "file:///output"),
            {"type": "data_reference_created", "data_reference": "file:///output"},
        ),
        (
            ChannelItemInserted("channel_item_inserted", "out", "file:///output"),
            {
                "type": "channel_item_inserted",
                "channel_id": "out",
                "data_reference": "file:///output",
            },
        ),
        (
            TagInserted("tag_inserted", "ready"),
            {"type": "tag_inserted", "value": "ready", "data_reference": None},
        ),
        (
            JobFailed("job_failed", "jr-1", "boom"),
            {"type": "job_failed", "job_run_id": "jr-1", "error": "boom"},
        ),
    ]
    for message, expected in messages:
        assert json.loads(serialize_ipc_message(message)) == expected


_JOB_ARGS = '{"job_id":"job","data_reference_uri":"file:///input","workflow_run_id":"wr-1","job_run_id":"jr-1"}'


def test_parse_store_config() -> None:
    raw = '{"root_uri":"file:///custom-results","kwargs":{"auto_mkdir":"true"}}'
    config = parse_store_config(raw)

    assert config.root_uri == "file:///custom-results"
    assert config.kwargs == {"auto_mkdir": "true"}

    assert parse_store_config('{"root_uri":"memory://results"}').kwargs == {}


@pytest.mark.parametrize("root_uri", ["results", "absolute"])
def test_parse_store_config_assumes_local_path(
    root_uri: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.chdir(tmp_path)
    path = str(tmp_path / root_uri) if root_uri == "absolute" else root_uri
    config = parse_store_config(json.dumps({"root_uri": path}))

    assert config.root_uri == f"file://{tmp_path / root_uri}"
    assert config.kwargs == {}


def test_parse_zygo_store_and_input_uris_without_backend_credentials() -> None:
    config = parse_store_config(
        '{"root_uri":"zygo://runs","kwargs":{"api_host":"https://api.example.com","api_bearer_auth":"secret"}}'
    )
    args = parse_job_args(
        '{"job_id":"job","data_reference_uri":"zygo://runs/input.json","workflow_run_id":"wr-1","job_run_id":"jr-1"}'
    )

    assert DataUri(config.root_uri).protocol == "zygo"
    assert config.kwargs == {
        "api_host": "https://api.example.com",
        "api_bearer_auth": "secret",
    }
    assert DataUri(args.data_reference_uri).path == "runs/input.json"


@pytest.mark.parametrize(
    ("raw", "error"),
    [
        ("{", "valid JSON"),
        ("[]", "JSON object"),
        ("{}", "root_uri"),
        ('{"root_uri":""}', "root_uri"),
        ('{"root_uri":"unknown-protocol://results"}', "root_uri"),
        ('{"root_uri":"file:///tmp","extra":1}', "unknown fields"),
        ('{"root_uri":"file:///tmp","kwargs":[]}', "kwargs"),
        ('{"root_uri":"file:///tmp","kwargs":{"token":1}}', "kwargs"),
        ('{"root_uri":"file:///tmp","kwargs":null}', "kwargs"),
    ],
)
def test_parse_store_config_rejects_invalid(raw: str, error: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError, match=error):
        parse_store_config(raw)


def _transport_from_cli(
    monkeypatch: pytest.MonkeyPatch, argv: list[str]
) -> IpcTransport:
    transports: list[IpcTransport] = []

    def capture_run(
        *,
        target: str,
        args: JobRunArgs,
        store_config: StoreConfig | None,
        ipc_transport: IpcTransport,
    ) -> None:
        del target, args, store_config
        transports.append(ipc_transport)

    monkeypatch.setattr(cli_module, "run", capture_run)
    assert cli_module.main(argv) == 0
    assert len(transports) == 1
    return transports[0]


def test_build_transport_defaults_to_stdio(monkeypatch: pytest.MonkeyPatch) -> None:
    transport = _transport_from_cli(
        monkeypatch, ["run", "pkg.mod:workflow", "--args", _JOB_ARGS]
    )
    assert isinstance(transport, StdioTransport)


def test_build_transport_http_defaults(monkeypatch: pytest.MonkeyPatch) -> None:
    requests: list[urllib.request.Request] = []
    timeouts: list[float] = []

    def fake_urlopen(
        request: urllib.request.Request, *, timeout: float
    ) -> HTTPResponse:
        requests.append(request)
        timeouts.append(timeout)
        return cast("HTTPResponse", _SuccessfulResponse())

    monkeypatch.setattr(urllib.request, "urlopen", fake_urlopen)
    transport = _transport_from_cli(
        monkeypatch,
        [
            "run",
            "pkg.mod:workflow",
            "--args",
            _JOB_ARGS,
            "--http-config",
            '{"url":"https://example.com/events"}',
        ],
    )
    assert isinstance(transport, HttpTransport)
    transport.emit(ChannelItemInserted("channel_item_inserted", "out", "file:///one"))
    assert requests[0].full_url == "https://example.com/events"
    assert requests[0].get_header("Content-type") == "application/json"
    assert timeouts == [30.0]


def test_build_transport_http_configures_request(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    requests: list[urllib.request.Request] = []
    timeouts: list[float] = []

    def fake_urlopen(
        request: urllib.request.Request, *, timeout: float
    ) -> HTTPResponse:
        requests.append(request)
        timeouts.append(timeout)
        return cast("HTTPResponse", _SuccessfulResponse())

    monkeypatch.setattr(urllib.request, "urlopen", fake_urlopen)
    raw_config = json.dumps({
        "url": "https://example.com/api/events",
        "max_retries": 2,
        "retry_interval": 0.5,
        "timeout": 4,
        "headers": {" Authorization ": " Bearer secret "},
    })
    transport = _transport_from_cli(
        monkeypatch,
        [
            "run",
            "pkg.mod:workflow",
            "--args",
            _JOB_ARGS,
            "--http-config",
            raw_config,
        ],
    )
    assert isinstance(transport, HttpTransport)
    transport.emit(ChannelItemInserted("channel_item_inserted", "out", "file:///one"))
    assert requests[0].full_url == "https://example.com/api/events"
    assert requests[0].get_header("Authorization") == "Bearer secret"
    assert requests[0].get_header("Content-type") == "application/json"
    assert timeouts == [4.0]


@pytest.mark.parametrize(
    ("raw", "error"),
    [
        ("{", "valid JSON"),
        ("[]", "JSON object"),
        ("{}", "url"),
        ('{"url":"ftp://example.com"}', "url"),
        ('{"url":"https://example.com","extra":1}', "unknown fields"),
        ('{"url":"https://example.com","headers":[]}', "headers"),
        ('{"url":"https://example.com","headers":{"  ":"value"}}', "headers"),
        ('{"url":"https://example.com","headers":{"X-Test":1}}', "headers"),
        ('{"url":"https://example.com","timeout":0}', "timeout"),
        ('{"url":"https://example.com","timeout":null}', "timeout"),
        ('{"url":"https://example.com","timeout":true}', "timeout"),
        ('{"url":"https://example.com","max_retries":-1}', "max_retries"),
        ('{"url":"https://example.com","max_retries":1.5}', "max_retries"),
        ('{"url":"https://example.com","max_retries":true}', "max_retries"),
        ('{"url":"https://example.com","retry_interval":-1}', "retry_interval"),
        ('{"url":"https://example.com","retry_interval":0}', "retry_interval"),
        ('{"url":"https://example.com","retry_interval":null}', "retry_interval"),
        ('{"url":"https://example.com","retry_interval":true}', "retry_interval"),
    ],
)
def test_parse_http_config_rejects_invalid(raw: str, error: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError, match=error):
        parse_http_config(raw)


@pytest.mark.parametrize("field", ["timeout", "retry_interval"])
def test_parse_http_config_rejects_nonfinite(field: str) -> None:
    for value in (float("inf"), float("nan")):
        with pytest.raises(argparse.ArgumentTypeError, match=field):
            parse_http_config(json.dumps({"url": "https://example.com", field: value}))


@pytest.mark.parametrize(
    ("raw", "error"),
    [
        ("{", "valid JSON"),
        ("[]", "JSON object"),
        ("{}", "job_id"),
        ('{"job_id":1}', "job_id"),
        ('{"job_id":"job","unexpected":1}', "unknown fields"),
        ('{"job_id":"job","store_root_uri":"file:///tmp"}', "unknown fields"),
    ],
)
def test_parse_job_args_rejects_invalid(raw: str, error: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError, match=error):
        parse_job_args(raw)


@pytest.mark.parametrize(
    ("option", "raw", "error"),
    [
        ("--args", "{", "--args must be valid JSON"),
        ("--args", "{}", "--args.job_id"),
        ("--store-config", "{", "--store-config must be valid JSON"),
        ("--store-config", "{}", "--store-config.root_uri"),
        ("--http-config", "{", "--http-config must be valid JSON"),
        ("--http-config", "{}", "--http-config.url"),
    ],
)
def test_parser_reports_invalid_json_arguments(
    option: str, raw: str, error: str, capsys: pytest.CaptureFixture[str]
) -> None:
    argv = ["run", "pkg.mod:workflow", "--args", _JOB_ARGS]
    if option == "--args":
        argv[-1] = raw
    else:
        argv.extend([option, raw])
    with pytest.raises(SystemExit) as caught:
        build_parser().parse_args(argv)
    exit_error = caught.value
    assert isinstance(exit_error, SystemExit)
    assert exit_error.code == 2
    assert error in capsys.readouterr().err


def test_parser_run_stdio_defaults() -> None:
    args = build_parser().parse_args(
        ["run", "pkg.mod:workflow", "--args", _JOB_ARGS],
        namespace=IpcArguments(),
    )
    assert args.command == "run"
    assert args.target == "pkg.mod:workflow"
    assert args.args == parse_job_args(_JOB_ARGS)
    assert args.http_config is None
    assert args.store_config is None


def test_parser_run_store_config() -> None:
    raw = '{"root_uri":"memory://results","kwargs":{"token":"secret"}}'
    args = build_parser().parse_args(
        ["run", "pkg.mod:workflow", "--args", _JOB_ARGS, "--store-config", raw],
        namespace=IpcArguments(),
    )
    assert args.store_config == parse_store_config(raw)


def test_parser_run_http_config() -> None:
    raw = '{"url":"http://myservice.com/api/events","timeout":5.5}'
    args = build_parser().parse_args(
        ["run", "pkg.mod:workflow", "--args", _JOB_ARGS, "--http-config", raw],
        namespace=IpcArguments(),
    )
    assert args.http_config == parse_http_config(raw)


@pytest.mark.parametrize(
    "failure", [urllib.error.URLError("response lost"), TimeoutError("timed out")]
)
def test_http_transport_retries_same_envelope_and_uses_timeout(
    monkeypatch: pytest.MonkeyPatch, failure: Exception
) -> None:
    bodies: list[bytes] = []
    timeouts: list[float] = []
    sleeps: list[float] = []

    def fake_urlopen(request: urllib.request.Request, *, timeout: float):
        assert isinstance(request.data, bytes)
        bodies.append(request.data)
        timeouts.append(timeout)
        assert request.get_method() == "POST"
        assert request.get_header("Content-type") == "application/json"
        if len(bodies) <= 2:
            raise failure
        return cast("HTTPResponse", _SuccessfulResponse())

    monkeypatch.setattr(urllib.request, "urlopen", fake_urlopen)
    monkeypatch.setattr("zygo.cli.v0.transport.time.sleep", sleeps.append)
    transport = HttpTransport(
        url="https://example.com/events",
        max_retries=2,
        retry_interval=1.0,
        timeout=2.5,
        workflow_run_id="wr-1",
        job_run_id="jr-1",
    )
    transport.emit(ChannelItemInserted("channel_item_inserted", "out", "file:///one"))
    assert len(bodies) == 3
    assert bodies[0] == bodies[1] == bodies[2]
    assert timeouts == [2.5, 2.5, 2.5]
    assert sleeps == [1.0, 1.0]
    first = cast("dict[str, object]", json.loads(bodies[0]))
    assert first == {
        "id": first["id"],
        "workflow_run_id": "wr-1",
        "job_run_id": "jr-1",
        "message": {
            "type": "channel_item_inserted",
            "channel_id": "out",
            "data_reference": "file:///one",
        },
    }
    assert isinstance(first["id"], str) and first["id"]

    transport.emit(ChannelItemInserted("channel_item_inserted", "out", "file:///one"))
    subsequent = cast("dict[str, object]", json.loads(bodies[3]))
    assert subsequent["id"] != first["id"]
