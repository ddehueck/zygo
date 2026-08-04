# zygo/_cli_bridge.py

import importlib
from typing import Any

from zygo import Workflow


class ImportStringError(RuntimeError):
    pass


def load_import_string(import_string: str) -> Any:
    module_name, separator, attribute_path = import_string.partition(":")

    if not separator or not module_name or not attribute_path:
        raise ImportStringError(
            f'Expected "<module>:<attribute>", got {import_string!r}'
        )

    try:
        value: Any = importlib.import_module(module_name)
    except ImportError as exc:
        raise ImportStringError(
            f"Could not import module {module_name!r}"
        ) from exc

    for attribute in attribute_path.split("."):
        try:
            value = getattr(value, attribute)
        except AttributeError as exc:
            raise ImportStringError(
                f"{import_string!r} has no attribute {attribute!r}"
            ) from exc

    return value


def load_workflow(import_string: str) -> Workflow:
    value = load_import_string(import_string)

    if not isinstance(value, Workflow):
        raise ImportStringError(
            f"{import_string!r} resolves to {type(value).__name__}, "
            "not a zygo.Workflow"
        )

    return value
