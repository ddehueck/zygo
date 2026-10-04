from __future__ import annotations

from typing import TYPE_CHECKING, cast

from zygo._internal.meta.injection import build_injected_job_fn
from zygo._internal.meta.job_context import JobContextImpl
from zygo.cli.importer import Importer
from zygo.cli.v0.types import ChannelItemInserted, StoreConfig, WorkflowStoreConfig
from zygo.cli.v0.workflow_config import local_store_options
from zygo.store import DataUri
from zygo.workflow import Workflow
from zygo.workflow.store import WorkflowStore
from zygo.workflow.types import JobId, JobRunContext, JobRunId, WorkflowRunId

if TYPE_CHECKING:
    from collections.abc import Callable

    from zygo.cli.v0.transport import IpcTransport
    from zygo.cli.v0.types import JobRunArgs


def run(
    *,
    target: str,
    args: JobRunArgs,
    store_config: WorkflowStoreConfig | None,
    ipc_transport: IpcTransport,
) -> None:
    try:
        _execute_job(
            target=target,
            args=args,
            store_config=store_config,
            ipc_transport=ipc_transport,
        )
    except Exception as e:
        error_message = f"Failed to run job {args.job_id}: {e}"
        raise RuntimeError(error_message) from e


def _execute_job(
    *,
    target: str,
    args: JobRunArgs,
    store_config: WorkflowStoreConfig | None,
    ipc_transport: IpcTransport,
) -> None:
    importer = Importer.from_target(target)
    workflow = importer.load(Workflow)

    run_context = JobRunContext(
        workflow_run_id=WorkflowRunId(args.workflow_run_id),
        job_run_id=JobRunId(args.job_run_id),
        input=DataUri(args.data_reference_uri),
    )

    job_entry = workflow.jobs.get_by_id(JobId(args.job_id))
    if job_entry is None:
        raise ValueError(f"Could not find job {args.job_id}")

    if store_config is None:
        # Use local store options + defaults when no configuration is provided
        base = local_store_options(importer.module).root_uri.uri.rstrip("/")
        store_config = WorkflowStoreConfig(
            job=StoreConfig(root_uri=f"{base}/jobs/"),
            workflow=StoreConfig(root_uri=f"{base}/workflows/"),
            cache=StoreConfig(root_uri=f"{base}/cache/"),
        )
    store = WorkflowStore(
        context=run_context,
        config=store_config,
        ipc_transport=ipc_transport,
    )

    decoded_input = cast(
        "object", job_entry.input_channel.codec.decode(store.get(run_context.input))
    )

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
        extension = (
            f"{output_format.extension.with_leading_dot()}"
            if output_format.extension
            else ""
        )
        uri = store.put(
            f"{job_entry.output_channel.id}_{index}{extension}",
            job_entry.output_channel.codec.encode(item),
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
