import argparse
from collections.abc import Sequence

from zygo.cli.v0 import ml, workflow
from zygo.cli.v0.arguments import IpcArguments


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m zygo.cli.v0",
        description="Execute workflow and model commands for the Zygo runtime.",
    )
    domains = parser.add_subparsers(dest="domain", required=True)
    workflow.configure_parser(
        domains.add_parser("workflow", help="Inspect workflows and run jobs")
    )
    ml.configure_parser(domains.add_parser("ml", help="Model commands"))
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv, namespace=IpcArguments())

    match args.domain:
        case "workflow":
            workflow.execute(args)
        case "ml":
            ml.execute(args)
        case _:
            parser.error(f"Unsupported command group: {args.domain}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
