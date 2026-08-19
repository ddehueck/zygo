from dataclasses import asdict
import json
import sys

from zygo._internal.ipc.importer import load_workflow
from zygo._internal.ipc.v0.types import (
    JobMetadata,
    WorkflowMetadata,
)
from zygo.workflow import Workflow


def build_workflow_metadata(workflow: Workflow) -> WorkflowMetadata:
    return WorkflowMetadata(
        id=workflow.id,
        input_channel_id=workflow.input_channel.id,
        output_channel_id=workflow.output_channel.id,
        content_hash=workflow.content_hash,
        jobs=[
            JobMetadata(
                id=job.id,
                content_hash=job.hash,
                input_channel_id=job.input_channel.id,
                output_channel_id=job.output_channel.id,
            )
            for job in workflow.jobs
        ],
    )


def inspect_workflow(target: str) -> None:
    metadata = build_workflow_metadata(load_workflow(target))
    json.dump(asdict(metadata), sys.stdout)
    sys.stdout.write("\n")
