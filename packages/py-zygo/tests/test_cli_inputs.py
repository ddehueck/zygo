"""CLI input validation is separate from the generated wire dataclasses."""

from __future__ import annotations

import argparse
from dataclasses import asdict
import json

from pydantic import ValidationError
import pytest

from zygo.cli.v0.arguments import parse_command
from zygo.cli.v0.inputs import (
    HttpConfigInput,
    JobRunInput,
    StoreConfigInput,
    WorkflowRunJobCommandInput,
)
from zygo.cli.v0.types import (
    HttpConfig,
    JobRunArgs,
    ModelTrainCommand,
    StoreConfig,
    WorkflowRunJobCommand,
)

_JOB = {
    "job_id": "job",
    "data_reference_uri": "file:///input",
    "workflow_run_id": "workflow-run",
    "job_run_id": "job-run",
}


def test_job_input_maps_explicitly_to_protocol():
    model = JobRunInput.model_validate(_JOB)
    protocol = model.to_protocol()
    assert isinstance(protocol, JobRunArgs)
    assert asdict(protocol) == _JOB
    parsed = parse_command(
        json.dumps({
            "command": "workflow_run_job",
            "target": "pkg:workflow",
            "args": _JOB,
        })
    )
    assert isinstance(parsed, WorkflowRunJobCommand)
    assert parsed.args == protocol


def test_store_input_preserves_backend_json_types():
    kwargs = {"nested": [None, True, 1, 1.5, "value", {"token": "secret"}]}
    model = StoreConfigInput.model_validate({
        "root_uri": "memory:///dataset",
        "kwargs": kwargs,
    })
    protocol = model.to_protocol()
    assert isinstance(protocol, StoreConfig)
    assert protocol.kwargs == kwargs
    parsed = parse_command(
        json.dumps({
            "command": "model_train",
            "target": "pkg:model",
            "dataset_config": {"root_uri": "memory:///dataset", "kwargs": kwargs},
            "store_config": {"root_uri": "memory:///store"},
        })
    )
    assert isinstance(parsed, ModelTrainCommand)
    assert parsed.dataset_config == protocol
    assert StoreConfigInput(root_uri="memory:///dataset").kwargs == {}
    nested = model.kwargs["nested"]
    assert isinstance(nested, list)
    assert type(nested[1]) is bool
    assert type(nested[2]) is int


def test_workflow_store_inputs_map_nested_protocol_types():
    command = {
        "command": "workflow_run_job",
        "target": "pkg:workflow",
        "args": _JOB,
        "job_store_config": {
            "root_uri": "memory:///jobs",
            "kwargs": {"auto_mkdir": True},
        },
        "workflow_store_config": {"root_uri": "memory:///workflows"},
        "cache_store_config": {"root_uri": "memory:///cache"},
    }
    protocol = WorkflowRunJobCommandInput.model_validate(command).to_protocol()
    assert isinstance(protocol, WorkflowRunJobCommand)
    assert isinstance(protocol.job_store_config, StoreConfig)
    assert protocol.job_store_config.kwargs == {"auto_mkdir": True}
    assert protocol.workflow_store_config is not None
    assert protocol.workflow_store_config.kwargs == {}
    assert protocol.cache_store_config is not None
    assert protocol.cache_store_config.kwargs == {}
    assert parse_command(json.dumps(command)) == protocol


def test_store_input_normalizes_local_paths(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    protocol = StoreConfigInput(root_uri="results").to_protocol()
    assert protocol.root_uri == f"file://{tmp_path / 'results'}"
    parsed = parse_command(
        json.dumps({
            "command": "model_train",
            "target": "pkg:model",
            "dataset_config": {"root_uri": "memory:///dataset"},
            "store_config": {"root_uri": "results"},
        })
    )
    assert isinstance(parsed, ModelTrainCommand)
    assert parsed.store_config == protocol


def test_http_input_maps_normalized_defaults_to_protocol():
    protocol = HttpConfigInput.model_validate({
        "url": "https://example.com/events",
        "headers": {" Authorization ": " Bearer secret "},
        "timeout": 4,
    }).to_protocol()
    assert isinstance(protocol, HttpConfig)
    assert asdict(protocol) == {
        "url": "https://example.com/events",
        "headers": {"Authorization": "Bearer secret"},
        "timeout": 4.0,
        "max_retries": 3,
        "retry_interval": 5.0,
    }
    parsed = parse_command(
        json.dumps({
            "command": "workflow_run_job",
            "target": "pkg:workflow",
            "args": _JOB,
            "http_config": {"url": "https://example.com/events"},
        })
    )
    assert isinstance(parsed, WorkflowRunJobCommand)
    assert parsed.http_config == HttpConfig(url="https://example.com/events")


@pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
def test_backend_options_reject_nested_nonfinite_values(value):
    data: dict[str, object] = {"root_uri": "memory:///store"}
    data["kwargs"] = {"nested": [{"value": value}]}
    with pytest.raises(ValidationError):
        StoreConfigInput.model_validate(data)
    with pytest.raises(ValidationError):
        StoreConfigInput.model_validate_json(json.dumps(data))


@pytest.mark.parametrize(
    "kwargs", [[], None, {"nested": {1: "value"}}, {"value": object()}]
)
def test_backend_options_reject_non_json_python_values(kwargs):
    with pytest.raises(ValidationError):
        StoreConfigInput.model_validate({
            "root_uri": "memory:///dataset",
            "kwargs": kwargs,
        })


@pytest.mark.parametrize("uri", ["", None, True, 1])
def test_dataset_root_uri_is_a_nonempty_strict_string(uri):
    with pytest.raises(argparse.ArgumentTypeError, match=r"dataset_config\.root_uri"):
        parse_command(
            json.dumps({
                "command": "model_train",
                "target": "pkg:model",
                "dataset_config": {"root_uri": uri},
                "store_config": {"root_uri": "memory:///store"},
            })
        )


def test_nested_unknown_fields_report_the_option_path_without_input():
    data = {
        "command": "workflow_run_job",
        "target": "pkg:workflow",
        "args": _JOB,
        "job_store_config": {
            "root_uri": "memory:///jobs",
            "token": "sensitive-value",
        },
        "workflow_store_config": {"root_uri": "memory:///workflows"},
        "cache_store_config": {"root_uri": "memory:///cache"},
    }
    with pytest.raises(argparse.ArgumentTypeError) as caught:
        parse_command(json.dumps(data))
    message = str(caught.value)
    assert (
        "--args.workflow_run_job.job_store_config.token has unknown fields" in message
    )
    assert "sensitive-value" not in message


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("timeout", True),
        ("timeout", "4"),
        ("timeout", 0),
        ("timeout", None),
        ("timeout", float("inf")),
        ("retry_interval", True),
        ("retry_interval", -1),
        ("retry_interval", float("nan")),
        ("max_retries", True),
        ("max_retries", 1.5),
        ("max_retries", "3"),
        ("max_retries", -1),
        ("headers", {"  ": "value"}),
        ("headers", None),
        ("url", "ftp://example.com"),
    ],
)
def test_http_fields_are_strict(field, value):
    data = {
        "command": "workflow_run_job",
        "target": "pkg:workflow",
        "args": _JOB,
        "http_config": {"url": "https://example.com", field: value},
    }
    with pytest.raises(argparse.ArgumentTypeError, match=field):
        parse_command(json.dumps(data))


@pytest.mark.parametrize(
    ("raw", "message"), [("{", "valid JSON"), ("[]", "JSON object")]
)
def test_command_json_reports_argparse_errors(raw, message):
    with pytest.raises(argparse.ArgumentTypeError, match=message):
        parse_command(raw)


def test_invalid_header_value_is_not_exposed_in_diagnostics():
    with pytest.raises(argparse.ArgumentTypeError) as caught:
        parse_command(
            json.dumps({
                "command": "workflow_run_job",
                "target": "pkg:workflow",
                "args": _JOB,
                "http_config": {
                    "url": "https://example.com",
                    "headers": {"Authorization": {"secret": "sensitive-value"}},
                },
            })
        )
    assert "--args.workflow_run_job.http_config.headers.Authorization" in str(
        caught.value
    )
    assert "sensitive-value" not in str(caught.value)


def test_input_defaults_are_not_shared():
    first = StoreConfigInput(root_uri="memory:///dataset")
    second = StoreConfigInput(root_uri="memory:///dataset")
    first.kwargs["auto_mkdir"] = True
    assert second.kwargs == {}
