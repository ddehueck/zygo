from __future__ import annotations

from typing import TYPE_CHECKING

from zygo._internal.fsspec import FsspecUri
from zygo._internal.ipc.importer import load_workflow
from zygo._internal.meta.container import RunContainer
from zygo._internal.meta.injection import build_injected_call
from zygo.store import Reference, StoreOptions
from zygo.types import JobId, JobRunContext, JobRunId, WorkflowRunId

if TYPE_CHECKING:
    from zygo._internal.ipc.v0.types import JobRunArgs


def run(
    *,
    target: str,
    args: JobRunArgs,
) -> None:
    workflow = load_workflow(target)

    run_context = JobRunContext(
        workflow_run_id=WorkflowRunId(args.workflow_run_id),
        job_run_id=JobRunId(args.job_run_id),
        data_ref=Reference(
            key=args.data_reference_uri,
            scope="job",
            uri=FsspecUri(args.data_reference_uri),
            etag=args.data_reference_etag,
        ),
    )

    try:
        job_func = workflow.jobs.get_by_id(JobId(args.job_id))
        if job_func is None:
            raise ValueError(f"Could not find job {args.job_id}")

        container = RunContainer(
            context=run_context,
            # TODO: Where to pipe these options through
            store_options=StoreOptions(
                root_uri=FsspecUri(uri="file://./todoreplaceme")
            ),
        )
        callable_w_deps = build_injected_call(job_func, container=container)
        callable_w_deps()
    except Exception as e:
        raise RuntimeError(f"Failed to run job {args.job_id}: {e}") from e
