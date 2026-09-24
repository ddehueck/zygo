import argparse
from dataclasses import dataclass
import json
import math
from typing import cast

from zygo._internal.fsspec import FsspecUri
from zygo.cli.v0.types import JobRunArgs

_DEFAULT_HTTP_TIMEOUT_SECONDS = 30.0
_DEFAULT_HTTP_MAX_RETRY_COUNT = 3
_DEFAULT_HTTP_RETRY_INTERVAL_SECONDS = 5.0


@dataclass(frozen=True)
class ValidatedHttpConfig:
    url: str
    headers: dict[str, str]
    timeout: float
    max_retries: int
    retry_interval: float


def _parse_json_object(raw: str, option: str, fields: set[str]) -> dict[str, object]:
    try:
        data = cast("object", json.loads(raw))
    except json.JSONDecodeError as error:
        raise argparse.ArgumentTypeError(f"{option} must be valid JSON") from error
    if not isinstance(data, dict):
        raise argparse.ArgumentTypeError(f"{option} must be a JSON object")

    result = cast("dict[str, object]", data)

    unknown = set(result.keys()) - fields
    if unknown:
        raise argparse.ArgumentTypeError(
            f"{option} has unknown fields: {', '.join(sorted(unknown))}"
        )
    return result


def _parse_dict_value_as_string(
    data: dict[str, object], field: str, option: str
) -> str:
    value = data.get(field)
    if not isinstance(value, str):
        raise argparse.ArgumentTypeError(f"{option}.{field} must be a string")
    return value


def _positive_number(value: object, field: str) -> float:
    error = argparse.ArgumentTypeError(
        f"--http-config.{field} must be a positive finite number"
    )
    if not isinstance(value, (int, float)):
        raise error
    if isinstance(value, bool):
        raise error

    is_finite = math.isfinite(value)
    if not is_finite:
        raise error

    is_positive = value > 0
    if not is_positive:
        raise error

    return float(value)


def parse_job_args(raw: str) -> JobRunArgs:
    fields = {
        "job_id",
        "data_reference_uri",
        "workflow_run_id",
        "job_run_id",
        "store_root_uri",
    }
    data = _parse_json_object(raw, "--args", fields)
    store_root_uri: str | None = None
    if "store_root_uri" in data:
        value = data["store_root_uri"]
        if not isinstance(value, str) or "://" not in value:
            raise argparse.ArgumentTypeError(
                "--args.store_root_uri must be a non-empty fsspec URI"
            )
        try:
            FsspecUri(value)
        except ValueError as error:
            raise argparse.ArgumentTypeError(
                f"--args.store_root_uri is invalid: {error}"
            ) from error
        store_root_uri = value
    return JobRunArgs(
        job_id=_parse_dict_value_as_string(data, "job_id", "--args"),
        data_reference_uri=_parse_dict_value_as_string(
            data, "data_reference_uri", "--args"
        ),
        workflow_run_id=_parse_dict_value_as_string(data, "workflow_run_id", "--args"),
        job_run_id=_parse_dict_value_as_string(data, "job_run_id", "--args"),
        store_root_uri=store_root_uri,
    )


def parse_http_config(raw: str) -> ValidatedHttpConfig:
    data = _parse_json_object(
        raw,
        "--http-config",
        {"url", "headers", "timeout", "max_retries", "retry_interval"},
    )
    url = _parse_dict_value_as_string(data, "url", "--http-config")
    if not url.startswith(("http://", "https://")):
        raise argparse.ArgumentTypeError(
            "--http-config.url must be a full http:// or https:// URL"
        )

    headers_data = data.get("headers", {})
    if not isinstance(headers_data, dict):
        raise argparse.ArgumentTypeError(
            "--http-config.headers must map nonempty names to strings"
        )
    headers: dict[str, str] = {}
    for name, value in cast("dict[object, object]", headers_data).items():
        if not isinstance(name, str) or not name.strip() or not isinstance(value, str):
            raise argparse.ArgumentTypeError(
                "--http-config.headers must map nonempty names to strings"
            )
        headers[name.strip()] = value.strip()

    max_retries = data.get("max_retries", _DEFAULT_HTTP_MAX_RETRY_COUNT)
    if type(max_retries) is not int or max_retries < 0:
        raise argparse.ArgumentTypeError(
            "--http-config.max_retries must be a nonnegative integer"
        )
    return ValidatedHttpConfig(
        url=url,
        headers=headers,
        timeout=_positive_number(
            data.get("timeout", _DEFAULT_HTTP_TIMEOUT_SECONDS), "timeout"
        ),
        max_retries=max_retries,
        retry_interval=_positive_number(
            data.get("retry_interval", _DEFAULT_HTTP_RETRY_INTERVAL_SECONDS),
            "retry_interval",
        ),
    )
