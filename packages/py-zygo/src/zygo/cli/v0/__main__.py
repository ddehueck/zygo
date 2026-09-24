import argparse
from collections.abc import Sequence

from zygo.cli.v0.arguments import (
    ValidatedHttpConfig,
    parse_http_config,
    parse_job_args,
    parse_store_config,
)
from zygo.cli.v0.metadata import inspect_workflow
from zygo.cli.v0.run import run
from zygo.cli.v0.transport import HttpTransport, IpcTransport, StdioTransport
from zygo.cli.v0.types import JobRunArgs, StoreConfig


class IpcArguments(argparse.Namespace):
    command: str
    target: str
    args: JobRunArgs | None
    http_config: ValidatedHttpConfig | None
    store_config: StoreConfig | None

    def __init__(self) -> None:
        super().__init__()
        self.command = ""
        self.target = ""
        self.args = None
        self.http_config = None
        self.store_config = None


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m zygo.cli.v0",
        description="Inspect workflows and run jobs for the Zygo runtime.",
    )
    commands = parser.add_subparsers(dest="command", required=True)

    # Run a workflow job
    run_parser = commands.add_parser(
        "run",
        help="Run a workflow job",
    )
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
        type=parse_store_config,
        metavar="JSON",
        help="Store root URI and optional fsspec keyword arguments as JSON",
    )
    run_parser.add_argument(
        "--http-config",
        type=parse_http_config,
        metavar="JSON",
        help="HTTP IPC settings as a JSON object; omit to publish via stdout",
    )

    # Get workflow metadata
    schema_parser = commands.add_parser(
        "metadata",
        help="Print a workflow schema",
    )
    schema_parser.add_argument(
        "target",
        help="Workflow import target, for example myproject.main:workflow",
    )

    return parser


def _build_transport(
    config: ValidatedHttpConfig | None, job_args: JobRunArgs
) -> IpcTransport:
    if config is None:
        return StdioTransport()
    return HttpTransport(
        url=config.url,
        max_retries=config.max_retries,
        retry_interval=config.retry_interval,
        timeout=config.timeout,
        workflow_run_id=job_args.workflow_run_id,
        job_run_id=job_args.job_run_id,
        headers=config.headers,
    )


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv, namespace=IpcArguments())

    match args.command:
        case "run":
            job_args = args.args
            if job_args is None:
                parser.error("--args is required")
                return 2
            ipc_transport = _build_transport(args.http_config, job_args)
            run(
                target=args.target,
                args=job_args,
                store_config=args.store_config,
                ipc_transport=ipc_transport,
            )
        case "metadata":
            inspect_workflow(args.target)
        case _:
            parser.error(f"Unsupported command: {args.command}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
