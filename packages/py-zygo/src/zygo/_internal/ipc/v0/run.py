from __future__ import annotations

from typing import TYPE_CHECKING, cast

from zygo._internal.fsspec import FsspecUri
from zygo._internal.ipc.importer import load_workflow
from zygo._internal.ipc.v0.types import (
    ChannelItemInserted,
    DataReference,
    write_stdout_ipc_message,
)
from zygo._internal.meta.injection import build_injected_job_fn
from zygo._internal.meta.job_context import JobContextImpl
from zygo.store import Reference, StoreOptions
from zygo.store._internal.impl import StoreImpl
from zygo.types import JobId, JobRunContext, JobRunId, WorkflowRunId

if TYPE_CHECKING:
    from collections.abc import Callable

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
            etag=args.data_reference_version,
        ),
    )

    try:
        job_entry = workflow.jobs.get_by_id(JobId(args.job_id))
        if job_entry is None:
            raise ValueError(f"Could not find job {args.job_id}")

        store = StoreImpl(
            context=run_context,
            options=StoreOptions(
                # TODO: Where to pipe these options through
                root_uri=FsspecUri(uri="file://./todoreplaceme")
            ),
        )

        input_bytes = store.get(run_context.data_ref)
        decoded_input = cast(
            "object", job_entry.input_channel.codec.decode(input_bytes)
        )

        callable_w_deps = build_injected_job_fn(
            cast("Callable[..., object]", job_entry.job_fn),
            input_data=decoded_input,
            ctx=JobContextImpl(store=store),
        )
        result = callable_w_deps()

        if result is None:
            return

        # Save to store to get a data reference.
        # Then send data reference to output channel via stdout ipc
        output_format = job_entry.output_channel.codec.format
        output_bytes = job_entry.output_channel.codec.encode(result)
        extension = (
            f"{output_format.extension.with_leading_dot()}"
            if output_format.extension
            else ""
        )
        reference = store.put(
            f"{job_entry.output_channel.id}{extension}",
            output_bytes,
        )
        write_stdout_ipc_message(
            ChannelItemInserted(
                channel_id=job_entry.output_channel.id,
                data_reference=DataReference.from_reference(reference),
            )
        )

    except Exception as e:
        raise RuntimeError(f"Failed to run job {args.job_id}: {e}") from e
