from dataclasses import asdict
import json
from typing import Annotated

import pytest

from zygo import Input, Output, Publisher, Reference, Workflow
from zygo._internal.ipc.v0 import metadata as metadata_module
from zygo._internal.ipc.v0.metadata import build_workflow_metadata
from zygo._internal.ipc.v0.types import STDOUT_IPC_PREFIX


def _workflow() -> Workflow:
    workflow = Workflow(id="example")
    source = workflow.channel(id="source")
    processed = workflow.channel(id="processed")

    @workflow.job(id="transform")
    def transform(
        input_ref: Annotated[Reference, Input(source)],
        publisher: Annotated[Publisher, Output(processed)],
    ) -> None:
        del input_ref, publisher

    @workflow.job
    def consume(
        input_ref: Annotated[Reference, Input(processed)],
    ) -> None:
        del input_ref

    del transform, consume
    return workflow


def test_build_workflow_metadata() -> None:
    workflow = _workflow()

    result = asdict(build_workflow_metadata(workflow))

    assert result == {
        "id": "example",
        "content_hash": workflow.content_hash,
        "channels": [{"id": "source"}, {"id": "processed"}],
        "jobs": [
            {
                "id": entry.id,
                "content_hash": entry.hash,
            }
            for entry in workflow.jobs.entries()
        ],
        "edges": [
            {"job_id": "transform", "channel_id": "source", "kind": "input"},
            {
                "job_id": "transform",
                "channel_id": "processed",
                "kind": "output",
            },
            {"job_id": "consume", "channel_id": "processed", "kind": "input"},
        ],
    }


def test_inspect_workflow_prints_json(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    workflow = _workflow()

    def load_workflow(target: str) -> Workflow:
        assert target == "module:workflow"
        return workflow

    monkeypatch.setattr(metadata_module, "load_workflow", load_workflow)

    metadata_module.inspect_workflow("module:workflow")

    stdout = capsys.readouterr().out
    assert stdout.startswith(STDOUT_IPC_PREFIX)
    assert json.loads(stdout.removeprefix(STDOUT_IPC_PREFIX)) == asdict(
        build_workflow_metadata(workflow)
    )
