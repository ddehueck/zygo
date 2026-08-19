# pyright: reportAny=false, reportExplicitAny=false
# ruff: file-ignore[any-type]

import importlib
from types import ModuleType
from typing import Any

from zygo.workflow import Workflow

CONVENTIONAL_NAMES = ("workflow", "wf")


def import_module(module_name: str) -> ModuleType:
    try:
        return importlib.import_module(module_name)
    except Exception as exc:
        raise RuntimeError(f"Could not import module {module_name!r}: {exc}") from exc


def resolve_attribute(
    module: ModuleType,
    attribute_path: str,
) -> Any:
    value: Any = module

    for part in attribute_path.split("."):
        try:
            value = getattr(value, part)
        except AttributeError as exc:
            raise RuntimeError(
                f"{module.__name__!r} has no attribute {attribute_path!r}"
            ) from exc

    return value


def discover_workflow(module: ModuleType) -> Workflow:
    namespace = vars(module)

    # Conventional names get priority.
    for preferred_name in CONVENTIONAL_NAMES:
        value = namespace.get(preferred_name)

        if isinstance(value, Workflow):
            return value

    # Fallback to the first Workflow instance found in the module.
    matches = [
        (name, value)
        for name, value in namespace.items()
        if isinstance(value, Workflow)
    ]

    if not matches:
        raise RuntimeError(f"No Workflow instance found in {module.__name__!r}")

    if len(matches) > 1:
        names = ", ".join(name for name, _ in matches)

        raise RuntimeError(
            f"Multiple Workflow instances found in {module.__name__!r}: {names}. Select one with {module.__name__}:<name>."
        )

    return matches[0][1]


def load_workflow(target: str) -> Workflow:
    module_name, separator, attribute_path = target.partition(":")

    if not module_name:
        raise RuntimeError("A Python module is required")

    module = import_module(module_name)

    if not separator:
        return discover_workflow(module)

    if not attribute_path:
        raise RuntimeError(f"Expected {module_name}:<workflow-name>")

    value = resolve_attribute(module, attribute_path)

    if not isinstance(value, Workflow):
        raise RuntimeError(
            f"{target!r} resolved to {type(value).__name__}, not a zygo.Workflow"
        )

    return value
