"""Unit tests for workflow metadata IPC output."""

from dataclasses import asdict
import json
import sys
from types import ModuleType

import pytest

from zygo import Channel, Workflow
from zygo.cli.v0 import workflow_metadata as metadata_module
from zygo.cli.v0.types import STDOUT_IPC_PREFIX
from zygo.cli.v0.workflow_metadata import build_workflow_metadata
from zygo.workflow.codecs import Integer, String


def _workflow() -> Workflow:
    source = Channel(id="source", codec=Integer)
    processed = Channel(id="processed", codec=String)
    workflow = Workflow(id="example", input=source, output=processed)

    @workflow.job(input=source, output=processed)
    def transform(value: int) -> str:
        return str(value)

    del transform
    return workflow


def test_build_workflow_metadata() -> None:
    workflow = _workflow()

    result = asdict(build_workflow_metadata(workflow))

    assert result == {
        "id": "example",
        "content_hash": workflow.content_hash,
        "input_channel_id": "source",
        "output_channel_id": "processed",
        "jobs": [
            {
                "id": "transform",
                "content_hash": next(iter(workflow.jobs)).hash,
                "input_channel_id": "source",
                "output_channel_id": "processed",
            }
        ],
        "channels": [
            {"id": "source", "accepted_file_extensions": ["txt"]},
            {"id": "processed", "accepted_file_extensions": ["txt"]},
        ],
    }


def test_inspect_workflow_prints_json(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    workflow = _workflow()

    module = ModuleType("module")
    vars(module)["workflow"] = workflow
    monkeypatch.setitem(sys.modules, "module", module)

    metadata_module.inspect_workflow("module:workflow")

    stdout = capsys.readouterr().out
    assert stdout.startswith(STDOUT_IPC_PREFIX)
    assert json.loads(stdout.removeprefix(STDOUT_IPC_PREFIX)) == asdict(
        build_workflow_metadata(workflow)
    )
