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
    from zygo.workflow.store import WorkflowStoreConfig

import zygo.cli.v0.__main__ as cli_module
from zygo.cli.v0.__main__ import build_parser
from zygo.cli.v0.arguments import parse_command
from zygo.cli.v0.transport import HttpTransport, StdioTransport
from zygo.cli.v0.types import (
    ChannelItemInserted,
    DataReferenceInserted,
    JobRunArgs,
    ModelTrainCommand,
    StoreConfig,
    TagInserted,
    WorkflowRunJobCommand,
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
            DataReferenceInserted("data_reference_inserted", "file:///output"),
            {"type": "data_reference_inserted", "data_reference": "file:///output"},
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
    ]
    for message, expected in messages:
        assert json.loads(serialize_ipc_message(message)) == expected


_JOB = {
    "job_id": "job",
    "data_reference_uri": "file:///input",
    "workflow_run_id": "wr-1",
    "job_run_id": "jr-1",
}


def _run_command(**fields: object) -> str:
    return json.dumps({
        "command": "workflow_run_job",
        "target": "pkg.mod:workflow",
        "args": _JOB,
        **fields,
    })


def _train_command(**fields: object) -> str:
    body: dict[str, object] = {
        "command": "model_train",
        "target": "pkg:model",
        "dataset_config": {"root_uri": "memory:///dataset"},
        "store_config": {"root_uri": "memory:///store"},
    }
    body.update(fields)
    return json.dumps(body)


def test_parse_store_config() -> None:
    parsed = parse_command(
        _train_command(
            store_config={
                "root_uri": "file:///custom-results",
                "kwargs": {"auto_mkdir": True},
            }
        )
    )
    assert isinstance(parsed, ModelTrainCommand)
    assert parsed.store_config.root_uri == "file:///custom-results"
    assert parsed.store_config.kwargs == {"auto_mkdir": True}

    defaults = parse_command(
        _train_command(store_config={"root_uri": "memory:///results"})
    )
    assert isinstance(defaults, ModelTrainCommand)
    assert defaults.store_config.kwargs == {}


@pytest.mark.parametrize("root_uri", ["results", "absolute"])
def test_parse_store_config_assumes_local_path(
    root_uri: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.chdir(tmp_path)
    path = str(tmp_path / root_uri) if root_uri == "absolute" else root_uri
    parsed = parse_command(_train_command(store_config={"root_uri": path}))
    assert isinstance(parsed, ModelTrainCommand)
    assert parsed.store_config.root_uri == f"file://{tmp_path / root_uri}"
    assert parsed.store_config.kwargs == {}


def test_parse_zygo_store_and_input_uris_without_backend_credentials() -> None:
    parsed = parse_command(
        json.dumps({
            "command": "workflow_run_job",
            "target": "pkg.mod:workflow",
            "args": {
                "job_id": "job",
                "data_reference_uri": "zygo://runs/input.json",
                "workflow_run_id": "wr-1",
                "job_run_id": "jr-1",
            },
            "job_store_config": {
                "root_uri": "zygo://runs",
                "kwargs": {
                    "api_host": "https://api.example.com",
                    "api_bearer_auth": "secret",
                },
            },
            "workflow_store_config": {"root_uri": "zygo://workflows"},
            "cache_store_config": {"root_uri": "zygo://cache"},
        })
    )
    assert isinstance(parsed, WorkflowRunJobCommand)
    config = parsed.job_store_config
    assert isinstance(config, StoreConfig)
    assert DataUri(config.root_uri).protocol == "zygo"
    assert config.kwargs == {
        "api_host": "https://api.example.com",
        "api_bearer_auth": "secret",
    }
    assert DataUri(parsed.args.data_reference_uri).path == "runs/input.json"


@pytest.mark.parametrize(
    ("raw", "error"),
    [
        ("{", "valid JSON"),
        ("[]", "JSON object"),
        (_train_command(store_config={"root_uri": ""}), "store_config.root_uri"),
        (
            _train_command(store_config={"root_uri": "unknown-protocol://results"}),
            "store_config.root_uri",
        ),
        (
            _train_command(store_config={"root_uri": "file:///tmp", "extra": 1}),
            "unknown fields",
        ),
        (
            _train_command(store_config={"root_uri": "file:///tmp", "kwargs": []}),
            "store_config.kwargs",
        ),
        (
            _train_command(
                store_config={"root_uri": "file:///tmp", "kwargs": {"token": None}}
            ).replace('"token": null', '"token": NaN'),
            "store_config.kwargs",
        ),
        (
            _train_command(store_config={"root_uri": "file:///tmp", "kwargs": None}),
            "store_config.kwargs",
        ),
    ],
)
def test_parse_store_config_rejects_invalid(raw: str, error: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError, match=error):
        parse_command(raw)


def _transport_from_cli(
    monkeypatch: pytest.MonkeyPatch, argv: list[str]
) -> IpcTransport:
    transports: list[IpcTransport] = []

    def capture_run(
        *,
        target: str,
        args: JobRunArgs,
        store_config: WorkflowStoreConfig | None,
        ipc_transport: IpcTransport,
    ) -> None:
        del target, args, store_config
        transports.append(ipc_transport)

    monkeypatch.setattr("zygo.cli.v0.workflow.run", capture_run)
    assert cli_module.main(argv) == 0
    assert len(transports) == 1
    return transports[0]


def test_build_transport_defaults_to_stdio(monkeypatch: pytest.MonkeyPatch) -> None:
    transport = _transport_from_cli(monkeypatch, ["--args", _run_command()])
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
        ["--args", _run_command(http_config={"url": "https://example.com/events"})],
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
        "headers": {
            " Authorization ": " Bearer secret ",
            "X-Zygo-Attempt-Handle": "opaque-attempt-handle",
        },
    })
    transport = _transport_from_cli(
        monkeypatch,
        ["--args", _run_command(http_config=json.loads(raw_config))],
    )
    assert isinstance(transport, HttpTransport)
    transport.emit(ChannelItemInserted("channel_item_inserted", "out", "file:///one"))
    assert requests[0].full_url == "https://example.com/api/events"
    assert requests[0].get_header("Authorization") == "Bearer secret"
    assert requests[0].get_header("X-zygo-attempt-handle") == "opaque-attempt-handle"
    assert requests[0].get_header("Content-type") == "application/json"
    assert timeouts == [4.0]


@pytest.mark.parametrize(
    ("raw", "error"),
    [
        ("{", "valid JSON"),
        ("[]", "JSON object"),
        (_run_command(http_config={}), "http_config.url"),
        (_run_command(http_config={"url": "ftp://example.com"}), "http_config.url"),
        (
            _run_command(http_config={"url": "https://example.com", "extra": 1}),
            "unknown fields",
        ),
        (
            _run_command(http_config={"url": "https://example.com", "headers": []}),
            "http_config.headers",
        ),
        (
            _run_command(
                http_config={"url": "https://example.com", "headers": {"  ": "value"}}
            ),
            "http_config.headers",
        ),
        (
            _run_command(
                http_config={"url": "https://example.com", "headers": {"X-Test": 1}}
            ),
            "http_config.headers",
        ),
        (
            _run_command(http_config={"url": "https://example.com", "timeout": 0}),
            "http_config.timeout",
        ),
        (
            _run_command(http_config={"url": "https://example.com", "timeout": None}),
            "http_config.timeout",
        ),
        (
            _run_command(http_config={"url": "https://example.com", "timeout": True}),
            "http_config.timeout",
        ),
        (
            _run_command(http_config={"url": "https://example.com", "max_retries": -1}),
            "http_config.max_retries",
        ),
        (
            _run_command(
                http_config={"url": "https://example.com", "max_retries": 1.5}
            ),
            "http_config.max_retries",
        ),
        (
            _run_command(
                http_config={"url": "https://example.com", "max_retries": True}
            ),
            "http_config.max_retries",
        ),
        (
            _run_command(
                http_config={"url": "https://example.com", "retry_interval": -1}
            ),
            "http_config.retry_interval",
        ),
        (
            _run_command(
                http_config={"url": "https://example.com", "retry_interval": 0}
            ),
            "http_config.retry_interval",
        ),
        (
            _run_command(
                http_config={"url": "https://example.com", "retry_interval": None}
            ),
            "http_config.retry_interval",
        ),
        (
            _run_command(
                http_config={"url": "https://example.com", "retry_interval": True}
            ),
            "http_config.retry_interval",
        ),
    ],
)
def test_parse_http_config_rejects_invalid(raw: str, error: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError, match=error):
        parse_command(raw)


@pytest.mark.parametrize("field", ["timeout", "retry_interval"])
def test_parse_http_config_rejects_nonfinite(field: str) -> None:
    for value in (float("inf"), float("nan")):
        with pytest.raises(argparse.ArgumentTypeError, match=field):
            parse_command(
                _run_command(http_config={"url": "https://example.com", field: value})
            )


@pytest.mark.parametrize(
    ("raw", "error"),
    [
        ("{", "valid JSON"),
        ("[]", "JSON object"),
        (_run_command(args={}), "args.job_id"),
        (_run_command(args={"job_id": 1}), "args.job_id"),
        (
            _run_command(args={"job_id": "job", "unexpected": 1}),
            "unknown fields",
        ),
        (
            _run_command(args={"job_id": "job", "store_root_uri": "file:///tmp"}),
            "unknown fields",
        ),
    ],
)
def test_parse_job_args_rejects_invalid(raw: str, error: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError, match=error):
        parse_command(raw)


@pytest.mark.parametrize(
    ("raw", "error"),
    [
        ("{", "--args must be valid JSON"),
        ("{}", "--args requires a command field"),
        (
            json.dumps({
                "command": "workflow_run_job",
                "target": "pkg.mod:workflow",
            }),
            "--args.workflow_run_job.args",
        ),
        (_run_command(http_config={}), "--args.workflow_run_job.http_config.url"),
    ],
)
def test_parser_reports_invalid_json_arguments(
    raw: str, error: str, capsys: pytest.CaptureFixture[str]
) -> None:
    with pytest.raises(SystemExit) as caught:
        build_parser().parse_args(["--args", raw])
    exit_error = caught.value
    assert isinstance(exit_error, SystemExit)
    assert exit_error.code == 2
    assert error in capsys.readouterr().err


def test_parser_run_stdio_defaults() -> None:
    parsed = build_parser().parse_args(["--args", _run_command()])
    command = parsed.args
    assert isinstance(command, WorkflowRunJobCommand)
    assert command.command == "workflow_run_job"
    assert command.target == "pkg.mod:workflow"
    assert command.args == JobRunArgs(**_JOB)
    assert command.http_config is None
    assert command.job_store_config is None


def test_parser_run_store_config() -> None:
    parsed = build_parser().parse_args([
        "--args",
        _run_command(
            job_store_config={
                "root_uri": "memory:///jobs",
                "kwargs": {"token": "secret"},
            },
            workflow_store_config={"root_uri": "memory:///workflows"},
            cache_store_config={"root_uri": "memory:///cache"},
        ),
    ])
    command = parsed.args
    assert isinstance(command, WorkflowRunJobCommand)
    assert command.job_store_config == StoreConfig(
        root_uri="memory:///jobs", kwargs={"token": "secret"}
    )


def test_parser_run_http_config() -> None:
    parsed = build_parser().parse_args([
        "--args",
        _run_command(
            http_config={"url": "http://myservice.com/api/events", "timeout": 5.5}
        ),
    ])
    command = parsed.args
    assert isinstance(command, WorkflowRunJobCommand)
    assert command.http_config is not None
    assert command.http_config.url == "http://myservice.com/api/events"
    assert command.http_config.timeout == pytest.approx(5.5)


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
    )
    transport.emit(ChannelItemInserted("channel_item_inserted", "out", "file:///one"))
    assert len(bodies) == 3
    assert bodies[0] == bodies[1] == bodies[2]
    assert timeouts == [2.5, 2.5, 2.5]
    assert sleeps == [1.0, 1.0]
    first = cast("dict[str, object]", json.loads(bodies[0]))
    assert first == {
        "id": first["id"],
        "messages": [
            {
                "type": "channel_item_inserted",
                "channel_id": "out",
                "data_reference": "file:///one",
            }
        ],
    }
    assert isinstance(first["id"], str) and first["id"]

    transport.emit(ChannelItemInserted("channel_item_inserted", "out", "file:///one"))
    subsequent = cast("dict[str, object]", json.loads(bodies[3]))
    assert subsequent["id"] != first["id"]
