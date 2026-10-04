from zygo.cli.v0.transport import build_transport
from zygo.cli.v0.types import (
    WorkflowGetMetadataCommand,
    WorkflowRunJobCommand,
)
from zygo.cli.v0.workflow_metadata import inspect_workflow
from zygo.cli.v0.workflow_run import run
from zygo.workflow.store import WorkflowStoreConfig


def execute(command: WorkflowRunJobCommand | WorkflowGetMetadataCommand) -> None:
    match command:
        case WorkflowRunJobCommand():
            store_config = None
            if (
                command.job_store_config is not None
                and command.workflow_store_config is not None
                and command.cache_store_config is not None
            ):
                store_config = WorkflowStoreConfig(
                    job=command.job_store_config,
                    workflow=command.workflow_store_config,
                    cache=command.cache_store_config,
                )
            run(
                target=command.target,
                args=command.args,
                store_config=store_config,
                ipc_transport=build_transport(command.http_config),
            )
        case WorkflowGetMetadataCommand():
            inspect_workflow(command.target)
