from __future__ import annotations

import argparse
from typing import TYPE_CHECKING

from pydantic import TypeAdapter, ValidationError

from zygo.cli.v0.inputs import CliCommandInput

if TYPE_CHECKING:
    from zygo.cli.v0.types import CliCommand


_command_adapter = TypeAdapter(CliCommandInput)


def parse_command(raw: str) -> CliCommand:
    try:
        return _command_adapter.validate_json(raw).to_protocol()
    except ValidationError as error:
        # Do not echo raw inputs, which may contain credentials or headers.
        messages: list[str] = []
        for detail in error.errors(include_url=False, include_input=False):
            path = ".".join(["--args", *(str(part) for part in detail["loc"])])
            match detail["type"]:
                case "json_invalid":
                    message = "must be valid JSON"
                case "model_type" | "model_attributes_type" | "dict_type":
                    message = "must be a JSON object"
                case "union_tag_not_found":
                    message = "requires a command field"
                case "union_tag_invalid":
                    message = (
                        "command must be workflow_run, workflow_metadata, or model_train"
                    )
                case "extra_forbidden":
                    message = "has unknown fields"
                case _:
                    message = detail["msg"]
            messages.append(f"{path} {message}")
        raise argparse.ArgumentTypeError("\n".join(messages)) from error
