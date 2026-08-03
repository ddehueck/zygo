import argparse
from collections.abc import Sequence
import json
from typing import cast

from zygo._internal.ipc.v0.metadata import inspect_workflow
from zygo._internal.ipc.v0.run import run
from zygo._internal.ipc.v0.types import JobRunArgs


class IpcArguments(argparse.Namespace):
    command: str
    target: str
    args: str

    def __init__(self) -> None:
        super().__init__()
        self.command = ""
        self.target = ""
        self.args = ""


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m zygo._internal.ipc",
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
        metavar="JSON",
        help="Orchestrator-provided job arguments as JSON",
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


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv, namespace=IpcArguments())

    match args.command:
        case "run":
            job_args_data = cast("dict[str, str]", json.loads(args.args))
            job_args = JobRunArgs(**job_args_data)
            run(target=args.target, args=job_args)
        case "metadata":
            inspect_workflow(args.target)
        case _:
            parser.error(f"Unsupported command: {args.command}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
