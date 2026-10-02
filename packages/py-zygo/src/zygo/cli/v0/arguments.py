from __future__ import annotations

import argparse
from typing import TYPE_CHECKING

from pydantic import ValidationError

from zygo.cli.v0.inputs import (
    CliInput,
    DatasetConfigInput,
    HttpConfigInput,
    JobRunInput,
    StoreConfigInput,
    WorkflowStoreConfigInput,
)

if TYPE_CHECKING:
    from zygo.cli.v0.types import (
        DatasetConfig,
        HttpConfig,
        JobRunArgs,
        StoreConfig,
        WorkflowStoreConfig,
    )


class IpcArguments(argparse.Namespace):
    domain: str
    command: str
    target: str
    args: JobRunArgs | None
    http_config: HttpConfig | None
    dataset_config: DatasetConfig | None
    store_config: StoreConfig | WorkflowStoreConfig | None

    def __init__(self) -> None:
        super().__init__()
        self.domain = ""
        self.command = ""
        self.target = ""
        self.args = None
        self.http_config = None
        self.dataset_config = None
        self.store_config = None


def add_publication_options(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--http-config",
        type=parse_http_config,
        metavar="JSON",
        help="HTTP IPC settings as a JSON object. Omit to publish via stdout.",
    )


def _parse_input[Model: CliInput](raw: str, option: str, model: type[Model]) -> Model:
    try:
        return model.model_validate_json(raw)
    except ValidationError as error:
        # Keep diagnostics option-qualified and omit raw inputs, which may
        # contain backend credentials or authorization headers.
        messages: list[str] = []
        for detail in error.errors(include_url=False, include_input=False):
            path = ".".join([option, *(str(part) for part in detail["loc"])])
            match detail["type"]:
                case "json_invalid":
                    message = "must be valid JSON"
                case "model_type":
                    message = "must be a JSON object"
                case "extra_forbidden":
                    message = "has unknown fields"
                case _:
                    message = detail["msg"]
            messages.append(f"{path} {message}")
        raise argparse.ArgumentTypeError("\n".join(messages)) from error


def parse_job_args(raw: str) -> JobRunArgs:
    return _parse_input(raw, "--args", JobRunInput).to_protocol()


def parse_dataset_config(raw: str) -> DatasetConfig:
    return _parse_input(raw, "--dataset-config", DatasetConfigInput).to_protocol()


def parse_store_config(raw: str) -> StoreConfig:
    return _parse_input(raw, "--store-config", StoreConfigInput).to_protocol()


def parse_workflow_store_config(raw: str) -> WorkflowStoreConfig:
    return _parse_input(raw, "--store-config", WorkflowStoreConfigInput).to_protocol()


def parse_http_config(raw: str) -> HttpConfig:
    return _parse_input(raw, "--http-config", HttpConfigInput).to_protocol()
