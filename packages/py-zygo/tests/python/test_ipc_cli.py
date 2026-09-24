"""Unit tests for IPC CLI argument parsing helpers."""

from __future__ import annotations

import argparse
from contextlib import nullcontext
from dataclasses import asdict
import json
from types import SimpleNamespace
import urllib.error
import urllib.request

import pytest

from zygo.cli.v0.__main__ import _build_transport, build_parser
from zygo.cli.v0.arguments import parse_http_config, parse_job_args
from zygo.cli.v0.transport import HttpTransport, StdioTransport
from zygo.cli.v0.types import (
    ChannelItemInserted,
    DataReferenceCreated,
    JobRunArgs,
    TagInserted,
    serialize_ipc_message,
)


def test_generated_protocol_models_keep_wire_format() -> None:
    args = JobRunArgs("job", "file:///input", "workflow-run", "job-run")
    assert asdict(args) == {
        "job_id": "job",
        "data_reference_uri": "file:///input",
        "workflow_run_id": "workflow-run",
        "job_run_id": "job-run",
        "store_root_uri": None,
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
    ]
    for message, expected in messages:
        assert json.loads(serialize_ipc_message(message)) == expected


_JOB_ARGS = '{"job_id":"job","data_reference_uri":"file:///input","workflow_run_id":"wr-1","job_run_id":"jr-1"}'


def test_parse_job_args_store_root_uri() -> None:
    payload = {
        "job_id": "job",
        "data_reference_uri": "file:///input",
        "workflow_run_id": "wr-1",
        "job_run_id": "jr-1",
        "store_root_uri": "file:///custom-results",
    }

    assert (
        parse_job_args(json.dumps(payload)).store_root_uri == "file:///custom-results"
    )
    assert parse_job_args(_JOB_ARGS).store_root_uri is None

    with pytest.raises(argparse.ArgumentTypeError, match="store_root_uri"):
        parse_job_args(json.dumps({**payload, "store_root_uri": None}))


def test_build_transport_defaults_to_stdio() -> None:
    assert isinstance(_build_transport(None, parse_job_args(_JOB_ARGS)), StdioTransport)


def test_build_transport_http_defaults() -> None:
    config = parse_http_config('{"url":"https://example.com/events"}')
    transport = _build_transport(config, parse_job_args(_JOB_ARGS))
    assert isinstance(transport, HttpTransport)
    assert transport._url == "https://example.com/events"
    assert transport._max_retries == 3
    assert transport._retry_interval == 5.0
    assert transport._timeout == 30.0
    assert transport._headers == {"Content-Type": "application/json"}


def test_build_transport_http_success() -> None:
    config = parse_http_config(
        json.dumps({
            "url": "https://example.com/api/events",
            "max_retries": 2,
            "retry_interval": 0.5,
            "timeout": 4,
            "headers": {" Authorization ": " Bearer secret "},
        })
    )
    transport = _build_transport(config, parse_job_args(_JOB_ARGS))
    assert isinstance(transport, HttpTransport)
    assert transport._url == "https://example.com/api/events"
    assert transport._max_retries == 2
    assert transport._retry_interval == 0.5
    assert transport._timeout == 4.0
    assert transport._workflow_run_id == "wr-1"
    assert transport._job_run_id == "jr-1"
    assert transport._headers["Authorization"] == "Bearer secret"
    assert transport._headers["Content-Type"] == "application/json"


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
    )
    assert args.command == "run"
    assert args.target == "pkg.mod:workflow"
    assert args.args == parse_job_args(_JOB_ARGS)
    assert args.http_config is None


def test_parser_run_http_config() -> None:
    raw = '{"url":"http://myservice.com/api/events","timeout":5.5}'
    args = build_parser().parse_args(
        ["run", "pkg.mod:workflow", "--args", _JOB_ARGS, "--http-config", raw],
    )
    assert args.http_config == parse_http_config(raw)


@pytest.mark.parametrize(
    "failure", [urllib.error.URLError("response lost"), TimeoutError("timed out")]
)
def test_http_transport_retries_same_envelope_and_uses_timeout(
    monkeypatch, failure: Exception
) -> None:
    bodies: list[bytes] = []
    timeouts: list[float] = []
    sleeps: list[float] = []

    def fake_urlopen(request: urllib.request.Request, *, timeout: float):
        assert request.data is not None
        bodies.append(request.data)
        timeouts.append(timeout)
        assert request.get_method() == "POST"
        assert request.get_header("Content-type") == "application/json"
        if len(bodies) <= 2:
            raise failure
        return nullcontext(SimpleNamespace(status=200))

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
    first = json.loads(bodies[0])
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
    assert json.loads(bodies[3])["id"] != first["id"]
