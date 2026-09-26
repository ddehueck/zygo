from __future__ import annotations

from typing import TYPE_CHECKING, cast

from zygo._internal.meta.injection import build_injected_job_fn
from zygo._internal.meta.job_context import JobContextImpl
from zygo.cli.importer import load_workflow_with_module
from zygo.cli.v0.config import local_store_options
from zygo.cli.v0.types import (
    ChannelItemInserted,
    JobFailed,
    JobStarted,
    JobSucceeded,
)
from zygo.store import DataUri
from zygo.store._internal.impl import StoreImpl
from zygo.types import JobId, JobRunContext, JobRunId, WorkflowRunId

if TYPE_CHECKING:
    from collections.abc import Callable

    from zygo.cli.v0.transport import IpcTransport
    from zygo.cli.v0.types import JobRunArgs, StoreConfig


def run(
    *,
    target: str,
    args: JobRunArgs,
    store_config: StoreConfig | None,
    ipc_transport: IpcTransport,
) -> None:
    ipc_transport.emit(JobStarted(type="job_started", job_run_id=args.job_run_id))

    try:
        _execute_job(
            target=target,
            args=args,
            store_config=store_config,
            ipc_transport=ipc_transport,
        )
        ipc_transport.emit(
            JobSucceeded(type="job_succeeded", job_run_id=args.job_run_id)
        )
    except Exception as e:
        error_message = f"Failed to run job {args.job_id}: {e}"
        ipc_transport.emit(
            JobFailed(
                type="job_failed",
                job_run_id=args.job_run_id,
                error=error_message,
            )
        )
        raise RuntimeError(error_message) from e


def _execute_job(
    *,
    target: str,
    args: JobRunArgs,
    store_config: StoreConfig | None,
    ipc_transport: IpcTransport,
) -> None:
    workflow, module = load_workflow_with_module(target)

    run_context = JobRunContext(
        workflow_run_id=WorkflowRunId(args.workflow_run_id),
        job_run_id=JobRunId(args.job_run_id),
        input=DataUri(args.data_reference_uri),
    )

    job_entry = workflow.jobs.get_by_id(JobId(args.job_id))
    if job_entry is None:
        raise ValueError(f"Could not find job {args.job_id}")

    store = StoreImpl(
        context=run_context,
        root=local_store_options(
            module, store_config.root_uri if store_config is not None else None
        ).root_uri,
        kwargs=cast(
            "dict[str, str | int | float | bool | None] | None",
            store_config.kwargs if store_config is not None else None,
        ),
        ipc_transport=ipc_transport,
    )

    input_bytes = store.get(run_context.input)
    decoded_input = cast("object", job_entry.input_channel.codec.decode(input_bytes))

    callable_w_deps = build_injected_job_fn(
        cast("Callable[..., object]", job_entry.job_fn),
        input_data=decoded_input,
        ctx=JobContextImpl(store=store, ipc_transport=ipc_transport),
    )
    # Run the user-defined job function with injected dependencies.
    result = callable_w_deps()
    if result is None:
        return

    output_format = job_entry.output_channel.codec.format
    # Save output to store in multiple files or in a single file depending on the
    # return value and channel type.
    #
    # e.g. if the return value is a list and the channel type is a scalar,
    #      save each item in a separate file and publish a reference to each file.
    result_as_batch: list[object]
    if isinstance(result, list) and job_entry.output_channel.is_scalar:
        result_as_batch = cast("list[object]", result)
    else:
        result_as_batch = [cast("object", result)]

    # Save output to the store and collect its URI.
    data_uris: list[DataUri] = []
    for index, item in enumerate(result_as_batch):
        output_bytes = job_entry.output_channel.codec.encode(item)

        extension = (
            f"{output_format.extension.with_leading_dot()}"
            if output_format.extension
            else ""
        )
        uri = store.put(
            f"{job_entry.output_channel.id}_{index}{extension}",
            output_bytes,
        )
        data_uris.append(uri)

    # Then send output URIs to the output channel via IPC.
    ipc_transport.emit([
        ChannelItemInserted(
            type="channel_item_inserted",
            channel_id=job_entry.output_channel.id,
            data_reference=str(uri),
        )
        for uri in data_uris
    ])
