import argparse
import json
import sys
from typing import Any, cast

from zygo.types import (
    JobId,
    RunJobArgs,
    WorkflowRunId,
)


def parse_run_job_args(argv: list[str] | None = None) -> RunJobArgs | None:
    """Parse CLI-injected job arguments, or return ``None`` during inspection."""
    runtime_argv = sys.argv[1:] if argv is None else argv
    job_args_raw, job_id_raw = _extract_runtime_cli_args(runtime_argv)

    if job_args_raw is None and job_id_raw is None:
        return None
    if job_args_raw is None:
        raise ValueError("--job-args is required for job execution")
    if job_id_raw is None:
        raise ValueError("--job-id is required for job execution")

    return _parse_run_job_args(
        job_args_raw=job_args_raw,
        job_id=JobId(job_id_raw),
    )


def _parse_run_job_args(*, job_args_raw: str, job_id: JobId) -> RunJobArgs:
    try:
        raw: Any = json.loads(job_args_raw)  # pyright: ignore[reportExplicitAny, reportAny]
    except json.JSONDecodeError:
        raise ValueError("Invalid JSON") from None

    if not isinstance(raw, dict):
        raise ValueError("Invalid JSON format")

    data = cast("dict[str, object]", raw)
    return RunJobArgs(
        run_id=WorkflowRunId(_require_string_field(data, "run_id")),
        job_id=job_id,
        data_reference_uri=_require_string_field(data, "data_reference_uri"),
        data_reference_version=_require_string_field(data, "data_reference_version"),
        job_run_id=_require_string_field(data, "job_run_id"),
    )


def _require_string_field(data: dict[str, object], field: str) -> str:
    value = data.get(field)
    if not isinstance(value, str):
        raise ValueError(f"{field} must be a string, got {type(value).__name__}")
    return value


def _extract_runtime_cli_args(argv: list[str]) -> tuple[str | None, str | None]:
    parser = argparse.ArgumentParser()
    parser.add_argument("--job-args", default=None)
    parser.add_argument("--job-id", default=None)
    args, _ = parser.parse_known_args(argv)
    job_args: str | None = args.job_args  # pyright: ignore[reportAny]
    job_id: str | None = args.job_id  # pyright: ignore[reportAny]
    return job_args, job_id
