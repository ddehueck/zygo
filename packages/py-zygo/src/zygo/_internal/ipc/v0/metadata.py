from dataclasses import asdict
import json
import sys
from typing import TypeVar

from zygo._internal.ipc.importer import load_workflow
from zygo._internal.ipc.v0.types import (
    STDOUT_IPC_PREFIX,
    ChannelMetadata,
    JobMetadata,
    WorkflowMetadata,
)
from zygo.channel import Channel
from zygo.workflow import Workflow

C = TypeVar("C")


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
        channels=_collect_channel_metadata(workflow),
    )


def inspect_workflow(target: str) -> None:
    metadata = build_workflow_metadata(load_workflow(target))
    payload = json.dumps(asdict(metadata))
    sys.stdout.write(f"{STDOUT_IPC_PREFIX}{payload}\n")
    sys.stdout.flush()


def _collect_channel_metadata(workflow: Workflow) -> list[ChannelMetadata]:
    channels: dict[str, ChannelMetadata] = {}

    def add(channel: Channel[C]) -> None:
        channels.setdefault(
            channel.id,
            ChannelMetadata(
                id=channel.id,
                accepted_file_extensions=[
                    str(channel.codec.format.extension),
                ],
            ),
        )

    add(workflow.input_channel)
    add(workflow.output_channel)

    for job in workflow.jobs:
        add(job.input_channel)
        add(job.output_channel)

    return list(channels.values())
