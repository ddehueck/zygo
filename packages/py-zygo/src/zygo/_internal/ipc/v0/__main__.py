import argparse
from collections.abc import Sequence
import json
from typing import cast

from zygo._internal.ipc.v0.metadata import inspect_workflow
from zygo._internal.ipc.v0.run import run
from zygo._internal.ipc.v0.transport import HttpTransport, IpcTransport, StdioTransport
from zygo._internal.ipc.v0.types import JobRunArgs

_DEFAULT_HTTP_MAX_RETRIES = 3
_DEFAULT_HTTP_RETRY_INTERVAL = 1.0


class IpcArguments(argparse.Namespace):
    command: str
    target: str
    args: str
    use_http: bool
    http_host: str | None
    http_max_retries: int
    http_retry_interval: float
    http_header: list[str]

    def __init__(self) -> None:
        super().__init__()
        self.command = ""
        self.target = ""
        self.args = ""
        self.use_http = False
        self.http_host = None
        self.http_max_retries = _DEFAULT_HTTP_MAX_RETRIES
        self.http_retry_interval = _DEFAULT_HTTP_RETRY_INTERVAL
        self.http_header = []


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
    run_parser.add_argument(
        "--use-http",
        action="store_true",
        help="Send IPC events over HTTP instead of stdout",
    )
    run_parser.add_argument(
        "--http-host",
        default=None,
        metavar="URL",
        help="HTTP endpoint for IPC events, e.g. http://myservice.com/api/events",
    )
    run_parser.add_argument(
        "--http-max-retries",
        type=int,
        default=_DEFAULT_HTTP_MAX_RETRIES,
        metavar="N",
        help="Max retry attempts after the first HTTP emit failure (default: 3)",
    )
    run_parser.add_argument(
        "--http-retry-interval",
        type=float,
        default=_DEFAULT_HTTP_RETRY_INTERVAL,
        metavar="SECONDS",
        help="Base interval for linear HTTP retry backoff in seconds (default: 1.0)",
    )
    run_parser.add_argument(
        "--http-header",
        action="append",
        default=[],
        metavar="NAME:VALUE",
        help=(
            "Extra HTTP header to send with each IPC request "
            '(repeatable), e.g. --http-header "Authorization: Bearer TOKEN"'
        ),
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


def _parse_http_headers(raw_headers: list[str]) -> dict[str, str]:
    headers: dict[str, str] = {}
    for raw in raw_headers:
        if ":" not in raw:
            raise ValueError(
                f"Invalid --http-header {raw!r}; expected NAME:VALUE"
            )
        name, value = raw.split(":", maxsplit=1)
        name = name.strip()
        value = value.strip()
        if not name:
            raise ValueError(
                f"Invalid --http-header {raw!r}; header name must not be empty"
            )
        headers[name] = value
    return headers


def _build_transport(args: IpcArguments) -> IpcTransport:
    if not args.use_http:
        return StdioTransport()

    if args.http_host is None:
        raise ValueError("--http-host is required when --use-http is set")
    if args.http_max_retries < 0:
        raise ValueError("--http-max-retries must be >= 0")
    if args.http_retry_interval < 0:
        raise ValueError("--http-retry-interval must be >= 0")

    return HttpTransport(
        url=args.http_host,
        max_retries=args.http_max_retries,
        retry_interval=args.http_retry_interval,
        headers=_parse_http_headers(args.http_header),
    )


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv, namespace=IpcArguments())

    match args.command:
        case "run":
            try:
                ipc_transport = _build_transport(args)
            except ValueError as error:
                parser.error(str(error))

            job_args_data = cast("dict[str, str]", json.loads(args.args))
            job_args = JobRunArgs(**job_args_data)
            run(target=args.target, args=job_args, ipc_transport=ipc_transport)
        case "metadata":
            inspect_workflow(args.target)
        case _:
            parser.error(f"Unsupported command: {args.command}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
