from __future__ import annotations

from typing import TYPE_CHECKING, cast

from grpc import insecure_channel

from zygo._internal import grpc
from zygo._internal.meta.jobs import build_register_job_args
from zygo._internal.python.fsspec import FsspecUri
from zygo._internal.utils.id import UUID7
from zygo.store._internal.impl import ingest
from zygo.types import (
    JobName,
    RunEventContext,
)

if TYPE_CHECKING:
    from pathlib import Path
    from types import FunctionType

    from zygo.backends.protocol import Backend, Entrypoint
    from zygo.channel import Channel
    from zygo.workflow import Workflow


def start_workflow(
    *,
    workflow: Workflow,
    channel: Channel,
    uri: str,
    backend: Backend,
    module_path: Path,
) -> None:
    """Deploy the workflow, ingest input data, register with the orchestrator,
    and emit the initial channel event to kick off execution."""

    if not backend.allow_local_store and backend.store_options.root_uri.is_local():
        raise ValueError(
            "Backend does not allow local store. Please use a remote store. e.g. S3, GCS, etc."
        )

    entrypoints_by_job = backend.deploy(
        jobs=workflow.jobs.job_configs(),
        content_hash=workflow.content_hash,
        module_path=module_path,
    )

    # TODO: Use the backend's orchestrator URI? Need to seperate orchestrator URI for client vs for worker.
    with insecure_channel("localhost:50051") as grpc_channel:
        client = grpc.OrchestratorClient(grpc_channel)

        input_reference = ingest(
            data_uri=FsspecUri(uri), store_options=backend.store_options
        )

        response = _register_workflow(
            workflow=workflow,
            entrypoints_by_job=entrypoints_by_job,
            client=client,
        )

        workflow_run_id = str(UUID7.generate())
        client.emit(
            context=RunEventContext(
                workflow_version_id=response.workflow_version_id,
                workflow_run_id=workflow_run_id,
                source=grpc.InputSource(),
            ),
            event=grpc.ChannelItemInserted(
                channel_id=response.channel_ids_by_name[channel.name],
                data_reference=grpc.DataReference.from_store_ref(input_reference),
            ),
        )


def _register_workflow(
    *,
    workflow: Workflow,
    entrypoints_by_job: dict[JobName, Entrypoint],
    client: grpc.OrchestratorClient,
) -> grpc.RegisterWorkflowResponse:
    channel_schemas = [
        grpc.ChannelSchema(name=ch_name) for ch_name in workflow.channels
    ]

    job_schemas: list[grpc.JobSchema] = []
    for job_entry in workflow.jobs.entries():
        args = build_register_job_args(cast("FunctionType", job_entry.job_fn))
        entrypoint = entrypoints_by_job.get(JobName(job_entry.name))

        schema = grpc.JobSchema(
            name=str(job_entry.name),
            content_hash=str(job_entry.hash),
            input_channel_name=str(args["input_channel"]),
            output_channel_names=[str(ch) for ch in args["output_channels"]],
            entrypoint=entrypoint,
        )
        job_schemas.append(schema)

    registration = grpc.WorkflowRegistration(
        name=workflow.name,
        content_hash=workflow.content_hash,
        channels=channel_schemas,
        jobs=job_schemas,
    )

    return client.register_workflow(registration)
