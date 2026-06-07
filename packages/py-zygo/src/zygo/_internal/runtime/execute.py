from __future__ import annotations

from typing import TYPE_CHECKING

from grpc import insecure_channel

from zygo._internal import grpc
from zygo._internal.meta.container import RunContainer
from zygo._internal.meta.injection import build_injected_call
from zygo._internal.python.fsspec import FsspecUri
from zygo.store import Reference
from zygo.types import (
    JobRunContext,
    RunEventContext,
)

if TYPE_CHECKING:
    from zygo.backends.protocol import Backend
    from zygo.types import RunJobArgs
    from zygo.workflow import Workflow


def execute_job(
    *,
    workflow: Workflow,
    run_job_args: RunJobArgs,
    backend: Backend,
) -> None:
    """Resolve and run a single job inside its own gRPC client scope."""

    with insecure_channel(backend.orchestrator_uri) as grpc_channel:
        client = grpc.OrchestratorClient(grpc_channel)
        _run(workflow=workflow, job_args=run_job_args, backend=backend, client=client)


def _run(
    *,
    workflow: Workflow,
    job_args: RunJobArgs,
    backend: Backend,
    client: grpc.OrchestratorClient,
) -> None:
    job_run_id = job_args.job_run_id

    event_ctx = RunEventContext(
        workflow_version_id=str(job_args.workflow_version_id),
        workflow_run_id=str(job_args.run_id),
        source=grpc.JobRunSource(
            job_id=str(job_args.job_id),
            job_run_id=job_run_id,
        ),
    )

    run_context = JobRunContext(
        workflow_run_id=str(job_args.run_id),
        job_run_id=job_run_id,
        data_ref=Reference(
            key=job_args.data_reference_uri,
            scope="job",
            uri=FsspecUri(job_args.data_reference_uri),
            etag=job_args.data_reference_etag,
        ),
    )

    client.emit(context=event_ctx, event=grpc.JobStarted())

    try:
        job_func = workflow.jobs.get_by_name(job_args.job_fn_name)
        if job_func is None:
            raise ValueError(f"Could not find job {job_args.job_fn_name}")

        container = RunContainer(
            context=run_context,
            store_options=backend.store_options,
            client=client,
            event_context=event_ctx,
            channel_ids_by_name=job_args.channel_ids_by_name,
        )
        callable_w_deps = build_injected_call(job_func, container=container)
        callable_w_deps()

        client.emit(context=event_ctx, event=grpc.JobSucceeded())
    except Exception as e:
        client.emit(
            context=event_ctx,
            event=grpc.JobFailed(error_message=str(e)),
        )
        raise RuntimeError(f"Failed to run job {job_args.job_fn_name}: {e}") from e
