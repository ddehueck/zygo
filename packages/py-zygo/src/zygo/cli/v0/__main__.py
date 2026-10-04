import argparse
from collections.abc import Sequence

from zygo.cli.v0 import ml, workflow
from zygo.cli.v0.arguments import parse_command
from zygo.cli.v0.types import (
    ModelTrainCommand,
    WorkflowGetMetadataCommand,
    WorkflowRunJobCommand,
)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m zygo.cli.v0",
        description="Execute workflow and model commands for the Zygo runtime.",
        allow_abbrev=False,
    )
    parser.add_argument(
        "--args",
        required=True,
        type=parse_command,
        metavar="JSON",
        help="One JSON command object containing command, target, and command inputs",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    command = parser.parse_args(argv).args

    match command:
        case WorkflowRunJobCommand() | WorkflowGetMetadataCommand():
            workflow.execute(command)
        case ModelTrainCommand():
            ml.execute(command)
        case _:
            parser.error("Unsupported command")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
