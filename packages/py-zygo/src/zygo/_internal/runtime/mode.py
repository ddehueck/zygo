import argparse
from dataclasses import dataclass
import json
import sys
from typing import (
    TYPE_CHECKING,
    Any,
    cast,
)

from zygo.types import (
    JobFnName,
    JobId,
    RunId,
    RunJobArgs,
    WorkflowVersionId,
)

if TYPE_CHECKING:
    from zygo.types import ChannelId, ChannelName


@dataclass(frozen=True)
class RunJobMode:
    run_job_args: RunJobArgs


@dataclass(frozen=True)
class StartMode:
    pass


type WorkflowEntrypointMode = RunJobMode | StartMode


def parse_workflow_entrypoint_mode(
    argv: list[str] | None = None,
) -> WorkflowEntrypointMode:
    runtime_argv = sys.argv[1:] if argv is None else argv
    if not runtime_argv:
        return StartMode()

    job_args_raw, job_fn_name_raw = _extract_runtime_cli_args(runtime_argv)
    if job_args_raw is None:
        return StartMode()
    if job_fn_name_raw is None:
        raise ValueError("--job-fn-name is required for job execution")

    return RunJobMode(
        run_job_args=_parse_run_job_args(
            job_args_raw=job_args_raw,
            job_fn_name=JobFnName(job_fn_name_raw),
        )
    )


def is_run_job_command(argv: list[str] | None = None) -> RunJobArgs | None:
    match parse_workflow_entrypoint_mode(argv):
        case RunJobMode(run_job_args=args):
            return args
        case StartMode():
            return None


def _parse_run_job_args(*, job_args_raw: str, job_fn_name: JobFnName) -> RunJobArgs:
    try:
        raw: Any = json.loads(job_args_raw)  # pyright: ignore[reportExplicitAny, reportAny]
    except json.JSONDecodeError:
        raise ValueError("Invalid JSON") from None

    if not isinstance(raw, dict):
        raise ValueError("Invalid JSON format")

    data = cast("dict[str, object]", raw)

    run_id = _require_string_field(data, "run_id")
    workflow_version_id = _require_string_field(data, "workflow_version_id")
    job_id = _require_string_field(data, "job_id")
    data_reference_uri = _require_string_field(data, "data_reference_uri")
    data_reference_etag = _require_string_field(data, "data_reference_etag")
    job_run_id = _require_string_field(data, "job_run_id")

    channel_ids_by_name = data.get("channel_ids_by_name")
    if not isinstance(channel_ids_by_name, dict):
        raise ValueError("channel_ids_by_name must be a dictionary")

    channel_ids_dict = cast("dict[object, object]", channel_ids_by_name)
    if not all(
        isinstance(key, str) and isinstance(value, str)
        for key, value in channel_ids_dict.items()
    ):
        raise ValueError("channel_ids_by_name must be a dictionary of str -> str")

    return RunJobArgs(
        run_id=RunId(run_id),
        workflow_version_id=WorkflowVersionId(workflow_version_id),
        job_fn_name=job_fn_name,
        job_id=JobId(job_id),
        data_reference_uri=data_reference_uri,
        data_reference_etag=data_reference_etag,
        channel_ids_by_name=cast("dict[ChannelName, ChannelId]", channel_ids_by_name),
        job_run_id=job_run_id,
    )


def _require_string_field(data: dict[str, object], field: str) -> str:
    value = data.get(field)
    if not isinstance(value, str):
        raise ValueError(f"{field} must be a string, got {type(value).__name__}")
    return value


def _extract_runtime_cli_args(argv: list[str]) -> tuple[str | None, str | None]:
    parser = argparse.ArgumentParser()
    parser.add_argument("--job-args", default=None)
    parser.add_argument("--job-fn-name", default=None)
    args, _ = parser.parse_known_args(argv)
    job_args: str | None = args.job_args  # pyright: ignore[reportAny]
    job_fn_name: str | None = args.job_fn_name  # pyright: ignore[reportAny]
    return job_args, job_fn_name
