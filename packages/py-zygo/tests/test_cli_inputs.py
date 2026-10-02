"""CLI input validation is separate from the generated wire dataclasses."""

from __future__ import annotations

import argparse
from dataclasses import asdict
import json

from pydantic import ValidationError
import pytest

from zygo.cli.v0.arguments import (
    parse_dataset_config,
    parse_http_config,
    parse_job_args,
    parse_store_config,
    parse_workflow_store_config,
)
from zygo.cli.v0.inputs import (
    DatasetConfigInput,
    HttpConfigInput,
    JobRunInput,
    StoreConfigInput,
    WorkflowStoreConfigInput,
)
from zygo.cli.v0.types import (
    DatasetConfig,
    HttpConfig,
    JobRunArgs,
    StoreConfig,
    WorkflowStoreConfig,
)


def test_job_input_maps_explicitly_to_protocol():
    data = {
        "job_id": "job",
        "data_reference_uri": "file:///input",
        "workflow_run_id": "workflow-run",
        "job_run_id": "job-run",
    }
    model = JobRunInput.model_validate(data)
    protocol = model.to_protocol()
    assert isinstance(protocol, JobRunArgs)
    assert asdict(protocol) == data
    assert parse_job_args(json.dumps(data)) == protocol


def test_dataset_input_preserves_backend_json_types():
    kwargs = {"nested": [None, True, 1, 1.5, "value", {"token": "secret"}]}
    model = DatasetConfigInput.model_validate({"uri": "dataset", "kwargs": kwargs})
    protocol = model.to_protocol()
    assert isinstance(protocol, DatasetConfig)
    assert protocol.kwargs == kwargs
    assert (
        parse_dataset_config(json.dumps({"uri": "dataset", "kwargs": kwargs}))
        == protocol
    )
    assert DatasetConfigInput(uri="dataset").kwargs == {}
    assert type(model.kwargs["nested"][1]) is bool
    assert type(model.kwargs["nested"][2]) is int


def test_store_inputs_map_nested_protocol_types():
    data = {
        "job": {"root_uri": "memory:///jobs", "kwargs": {"auto_mkdir": True}},
        "workflow": {"root_uri": "memory:///workflows"},
        "cache": {"root_uri": "memory:///cache"},
    }
    protocol = WorkflowStoreConfigInput.model_validate(data).to_protocol()
    assert isinstance(protocol, WorkflowStoreConfig)
    assert isinstance(protocol.job, StoreConfig)
    assert asdict(protocol) == {
        **data,
        "workflow": {**data["workflow"], "kwargs": {}},
        "cache": {**data["cache"], "kwargs": {}},
    }
    assert parse_workflow_store_config(json.dumps(data)) == protocol
    assert StoreConfigInput(
        root_uri="memory://jobs"
    ).to_protocol() == parse_store_config('{"root_uri":"memory://jobs"}')


def test_store_input_normalizes_local_paths(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    protocol = StoreConfigInput(root_uri="results").to_protocol()
    assert protocol.root_uri == f"file://{tmp_path / 'results'}"
    assert parse_store_config('{"root_uri":"results"}') == protocol


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
    assert parse_http_config('{"url":"https://example.com/events"}') == HttpConfig(
        url="https://example.com/events"
    )


@pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
@pytest.mark.parametrize("model", [DatasetConfigInput, StoreConfigInput])
def test_backend_options_reject_nested_nonfinite_values(model, value):
    data = (
        {"uri": "dataset"}
        if model is DatasetConfigInput
        else {"root_uri": "memory://store"}
    )
    data["kwargs"] = {"nested": [{"value": value}]}
    with pytest.raises(ValidationError):
        model.model_validate(data)
    with pytest.raises(ValidationError):
        model.model_validate_json(json.dumps(data))


@pytest.mark.parametrize(
    "kwargs", [[], None, {"nested": {1: "value"}}, {"value": object()}]
)
def test_backend_options_reject_non_json_python_values(kwargs):
    with pytest.raises(ValidationError):
        DatasetConfigInput.model_validate({"uri": "dataset", "kwargs": kwargs})


@pytest.mark.parametrize("uri", ["", None, True, 1])
def test_dataset_uri_is_a_nonempty_strict_string(uri):
    with pytest.raises(argparse.ArgumentTypeError, match=r"--dataset-config\.uri"):
        parse_dataset_config(json.dumps({"uri": uri}))


def test_nested_unknown_fields_report_the_option_path_without_input():
    data = {
        "job": {"root_uri": "memory://jobs", "token": "sensitive-value"},
        "workflow": {"root_uri": "memory://workflows"},
        "cache": {"root_uri": "memory://cache"},
    }
    with pytest.raises(argparse.ArgumentTypeError) as caught:
        parse_workflow_store_config(json.dumps(data))
    message = str(caught.value)
    assert "--store-config.job.token has unknown fields" in message
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
    data = {"url": "https://example.com", field: value}
    with pytest.raises(argparse.ArgumentTypeError, match=field):
        parse_http_config(json.dumps(data))


@pytest.mark.parametrize(
    "parse",
    [
        parse_job_args,
        parse_dataset_config,
        parse_store_config,
        parse_workflow_store_config,
        parse_http_config,
    ],
)
@pytest.mark.parametrize(
    ("raw", "message"), [("{", "valid JSON"), ("[]", "JSON object")]
)
def test_json_options_share_argparse_error_handling(parse, raw, message):
    with pytest.raises(argparse.ArgumentTypeError, match=message):
        parse(raw)


def test_invalid_header_value_is_not_exposed_in_diagnostics():
    with pytest.raises(argparse.ArgumentTypeError) as caught:
        parse_http_config(
            json.dumps({
                "url": "https://example.com",
                "headers": {"Authorization": {"secret": "sensitive-value"}},
            })
        )
    assert "--http-config.headers.Authorization" in str(caught.value)
    assert "sensitive-value" not in str(caught.value)


def test_input_defaults_are_not_shared():
    first = DatasetConfigInput(uri="dataset")
    second = DatasetConfigInput(uri="dataset")
    first.kwargs["auto_mkdir"] = True
    assert second.kwargs == {}
