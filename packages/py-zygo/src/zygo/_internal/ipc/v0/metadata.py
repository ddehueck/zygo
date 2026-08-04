from dataclasses import asdict
import json
import sys

from zygo._internal.ipc.importer import load_workflow
from zygo._internal.ipc.v0.types import (
    ChannelMetadata,
    EdgeMetadata,
    JobMetadata,
    WorkflowMetadata,
)
from zygo._internal.meta.jobs import build_job_definition
from zygo.workflow import Workflow


def build_workflow_metadata(workflow: Workflow) -> WorkflowMetadata:
    jobs = workflow.jobs.entries()
    edges: list[EdgeMetadata] = []

    for job in jobs:
        job_fn = workflow.jobs.get_by_id(job.id)
        if job_fn is None:
            raise RuntimeError(f"Job {job.id!r} disappeared from the registry")

        definition = build_job_definition(job_fn)
        edges.append(
            EdgeMetadata(
                job_id=job.id,
                channel_id=definition["input_channel_id"],
                kind="input",
            )
        )
        edges.extend(
            EdgeMetadata(job_id=job.id, channel_id=channel_id, kind="output")
            for channel_id in definition["output_channel_ids"]
        )

    if workflow.input_channel is None:
        raise ValueError("Input channel not found")

    return WorkflowMetadata(
        id=workflow.id,
        input_channel=workflow.input_channel.id,
        content_hash=workflow.content_hash,
        channels=[
            ChannelMetadata(id=channel.id) for channel in workflow.channels.values()
        ],
        jobs=[JobMetadata(id=job.id, content_hash=job.hash) for job in jobs],
        edges=edges,
    )


def inspect_workflow(target: str) -> None:
    metadata = build_workflow_metadata(load_workflow(target))
    json.dump(asdict(metadata), sys.stdout)
    sys.stdout.write("\n")
