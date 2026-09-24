from __future__ import annotations

from pathlib import Path
import tomllib
from typing import TYPE_CHECKING, cast

from zygo.store import DataUri, StoreOptions

if TYPE_CHECKING:
    from types import ModuleType

_DEFAULT_DATA_DIR = "zygo"


def _table(value: object) -> dict[str, object] | None:
    return cast("dict[str, object]", value) if isinstance(value, dict) else None


def project_search_paths(module: ModuleType) -> list[Path]:
    """List module-to-working-directory paths without searching above the caller."""
    cwd = Path.cwd().resolve()
    if module.__file__ is None:
        raise ValueError(f"Workflow module {module.__name__!r} has no file")

    module_dir = Path(module.__file__).resolve().parent
    if not module_dir.is_relative_to(cwd):
        return [cwd]

    paths = [module_dir]
    while paths[-1] != cwd:
        paths.append(paths[-1].parent)
    return paths


def local_store_options(
    module: ModuleType, store_root_uri: str | None = None
) -> StoreOptions:
    """Use a supplied store URI, or load config within the calling project."""
    if store_root_uri is not None:
        return StoreOptions(root_uri=DataUri(store_root_uri))

    search_paths = project_search_paths(module)
    path: Path | None = None
    for directory in search_paths:
        pyproject = directory / "pyproject.toml"
        if not pyproject.is_file():
            continue

        with pyproject.open("rb") as source:
            config = cast("dict[str, object]", tomllib.load(source))

        tool = _table(config.get("tool"))
        zygo = _table(tool.get("zygo")) if tool is not None else None
        local = _table(zygo.get("local")) if zygo is not None else None

        if local is not None and "data_dir" in local:
            data_dir = local["data_dir"]
            if not isinstance(data_dir, str) or not data_dir.strip():
                raise ValueError(
                    f"{pyproject}: tool.zygo.local.data_dir must be a non-empty path"
                )
            path = directory / data_dir
        break

    if path is None:
        path = search_paths[0] / _DEFAULT_DATA_DIR

    return StoreOptions(root_uri=DataUri(f"file://{path.resolve()}"))
