"""Workflow command registration and dispatch."""

from __future__ import annotations

from typing import TYPE_CHECKING

from zygo.cli.v0.arguments import (
    add_publication_options,
    parse_job_args,
    parse_workflow_store_config,
)
from zygo.cli.v0.transport import build_transport
from zygo.cli.v0.types import JobRunArgs, WorkflowStoreConfig
from zygo.cli.v0.workflow_metadata import inspect_workflow
from zygo.cli.v0.workflow_run import run

if TYPE_CHECKING:
    import argparse

    from zygo.cli.v0.arguments import IpcArguments


def configure_parser(parser: argparse.ArgumentParser) -> None:
    commands = parser.add_subparsers(dest="command", required=True)
    run_parser = commands.add_parser("run", help="Run a workflow job")
    run_parser.add_argument(
        "target",
        help="Workflow import target, for example myproject.main:workflow",
    )
    run_parser.add_argument(
        "--args",
        required=True,
        type=parse_job_args,
        metavar="JSON",
        help="Orchestrator-provided job arguments as JSON",
    )
    run_parser.add_argument(
        "--store-config",
        type=parse_workflow_store_config,
        metavar="JSON",
        help="Job, workflow, and cache store configurations as JSON",
    )
    add_publication_options(run_parser)

    metadata_parser = commands.add_parser("metadata", help="Print workflow metadata")
    metadata_parser.add_argument(
        "target",
        help="Workflow import target, for example myproject.main:workflow",
    )


def execute(args: IpcArguments) -> None:
    match args.command:
        case "run":
            job_args = args.args
            if not isinstance(job_args, JobRunArgs):
                raise TypeError("Workflow execution requires JobRunArgs")
            store_config = args.store_config
            if store_config is not None and not isinstance(
                store_config, WorkflowStoreConfig
            ):
                raise TypeError("Workflow execution requires WorkflowStoreConfig")
            run(
                target=args.target,
                args=job_args,
                store_config=store_config,
                ipc_transport=build_transport(args.http_config),
            )
        case "metadata":
            inspect_workflow(args.target)
        case _:
            raise ValueError(f"Unsupported workflow command: {args.command}")
