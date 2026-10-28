"""Resolve and package a local workflow deployment project."""

from __future__ import annotations

import ast
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
import re
import stat
import tomllib
from typing import TYPE_CHECKING, cast
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

from pathspec import GitIgnoreSpec

if TYPE_CHECKING:
    from zygo.cloud.api import PythonVersion

_MAX_SOURCE_ARCHIVE_BYTES = 50_000_000
_MAX_WORKFLOW_ID_LENGTH = 255
_MAX_PYTHON_TARGET_LENGTH = 255
_PYTHON_TARGET_PATTERN = re.compile(
    r"^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*"
    r"(?::[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)?$"
)
_SUPPORTED_PYTHON_VERSIONS = {"3.11", "3.12", "3.13", "3.14", "3.15"}
_BUILTIN_IGNORES = (
    ".git/",
    "**/.git/",
    ".venv/",
    "**/.venv/",
    "__pycache__/",
    "**/__pycache__/**",
    "*.py[cod]",
    ".pytest_cache/",
    "**/.pytest_cache/",
    ".mypy_cache/",
    "**/.mypy_cache/",
    ".ruff_cache/",
    "**/.ruff_cache/",
    ".env",
    "**/.env",
    ".env.*",
    "**/.env.*",
    "!.env.example",
    "!**/.env.example",
)


@dataclass(frozen=True)
class DeploymentProject:
    """Validated inputs needed to create a cloud deployment."""

    python_version: PythonVersion
    workflow_id: str
    target: str
    pyproject_toml: bytes
    uv_lock: bytes
    source_archive: bytes


def resolve_deployment_project(
    project_dir: str | Path, target: str
) -> DeploymentProject:
    """Validate project inputs, resolve its workflow ID, and package its source."""

    root = Path(project_dir).expanduser().resolve()
    if not root.is_dir():
        raise ValueError(f"project_dir must be an existing directory: {root}")

    pyproject_path = root / "pyproject.toml"
    if not pyproject_path.is_file():
        raise FileNotFoundError(
            f"No pyproject.toml found in project_dir {root}. "
            "Pass the directory containing the workflow project."
        )
    uv_lock_path = root / "uv.lock"
    if not uv_lock_path.is_file():
        raise FileNotFoundError(
            f"No uv.lock found in project_dir {root}. Run `uv lock` first."
        )
    if pyproject_path.is_symlink() or uv_lock_path.is_symlink():
        raise ValueError("pyproject.toml and uv.lock must be regular project files")

    config = _read_toml(pyproject_path)
    python_version = _cloud_python_version(config, pyproject_path)
    target_file = _resolve_target_file(root, target)
    source_archive, archived_paths = _build_source_archive(root)
    target_relative_path = target_file.relative_to(root).as_posix()
    if target_relative_path not in archived_paths:
        raise ValueError(
            f"Workflow target {target!r} is excluded from the source archive by an ignore rule"
        )

    workflow_id = _workflow_id_from_source(
        target_file, target, _configured_workflow_id(config, pyproject_path)
    )
    return DeploymentProject(
        python_version=python_version,
        workflow_id=workflow_id,
        target=target,
        pyproject_toml=pyproject_path.read_bytes(),
        uv_lock=uv_lock_path.read_bytes(),
        source_archive=source_archive,
    )


def _read_toml(path: Path) -> dict[str, object]:
    try:
        payload = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ValueError(f"Could not read {path}: {error}") from error
    return cast("dict[str, object]", payload)


def _cloud_config(config: dict[str, object]) -> dict[str, object]:
    tool = _table(config.get("tool"))
    zygo = _table(tool.get("zygo")) if tool is not None else None
    cloud = _table(zygo.get("cloud")) if zygo is not None else None
    return cloud if cloud is not None else {}


def _cloud_python_version(
    config: dict[str, object], pyproject_path: Path
) -> PythonVersion:
    value = _cloud_config(config).get("python_version")
    if not isinstance(value, str) or value not in _SUPPORTED_PYTHON_VERSIONS:
        supported = ", ".join(sorted(_SUPPORTED_PYTHON_VERSIONS))
        raise ValueError(
            f"{pyproject_path}: set [tool.zygo.cloud].python_version to one of: {supported}"
        )
    return cast("PythonVersion", value)


def _configured_workflow_id(
    config: dict[str, object], pyproject_path: Path
) -> str | None:
    value = _cloud_config(config).get("workflow_id")
    if value is None:
        return None
    if not isinstance(value, str) or not value or len(value) > _MAX_WORKFLOW_ID_LENGTH:
        raise ValueError(
            f"{pyproject_path}: tool.zygo.cloud.workflow_id must be a non-empty "
            "string up to 255 characters"
        )
    return value


def _resolve_target_file(project_dir: Path, target: str) -> Path:
    if len(target) > _MAX_PYTHON_TARGET_LENGTH or not _PYTHON_TARGET_PATTERN.fullmatch(
        target
    ):
        raise ValueError(
            "target must be a Python module target such as 'workflow' or "
            "'my_package.workflow:my_workflow'"
        )

    module_name = target.partition(":")[0]
    module_path = Path(*module_name.split("."))
    candidates: set[Path] = set()
    for import_root in (project_dir, project_dir / "src"):
        source_file = import_root / module_path.with_suffix(".py")
        package_file = import_root / module_path / "__init__.py"
        if source_file.is_file() and not source_file.is_symlink():
            candidates.add(source_file.resolve())
        if package_file.is_file() and not package_file.is_symlink():
            candidates.add(package_file.resolve())

    if not candidates:
        module_file = module_path.with_suffix(".py").as_posix()
        raise FileNotFoundError(
            f"Could not find target module {module_name!r} under {project_dir}. "
            f"Expected {module_file} or the corresponding package __init__.py."
        )
    if len(candidates) > 1:
        paths = ", ".join(str(path) for path in sorted(candidates))
        raise ValueError(
            f"Target module {module_name!r} is ambiguous under {project_dir}: {paths}"
        )
    return next(iter(candidates))


def _workflow_id_from_source(
    source_file: Path, target: str, configured_id: str | None
) -> str:
    try:
        module = ast.parse(
            source_file.read_text(encoding="utf-8"), filename=str(source_file)
        )
    except (OSError, SyntaxError, UnicodeDecodeError) as error:
        return _require_configured_workflow_id(configured_id, source_file, error)

    candidates = _workflow_constructor_assignments(module)
    attribute_path = target.partition(":")[2]
    if attribute_path:
        target_name = attribute_path.split(".")[0]
        candidates = [
            (name, workflow_id)
            for name, workflow_id in candidates
            if name == target_name
        ]
    else:
        conventional = [
            candidate for candidate in candidates if candidate[0] in {"workflow", "wf"}
        ]
        if conventional:
            candidates = conventional

    workflow_ids = {workflow_id for _, workflow_id in candidates if workflow_id}
    if len(candidates) == 1 and len(workflow_ids) == 1:
        source_id = next(iter(workflow_ids))
        if configured_id is not None and configured_id != source_id:
            raise ValueError(
                f"{source_file}: tool.zygo.cloud.workflow_id {configured_id!r} "
                f"does not match the workflow's id {source_id!r}"
            )
        return source_id
    return _require_configured_workflow_id(configured_id, source_file)


def _workflow_constructor_assignments(
    module: ast.Module,
) -> list[tuple[str, str | None]]:
    assignments: dict[int, set[str]] = {}
    for node in ast.walk(module):
        if isinstance(node, ast.Assign):
            value = node.value
            names = {
                name for target in node.targets for name in _assigned_names(target)
            }
        elif isinstance(node, ast.AnnAssign):
            value = node.value
            names = set(_assigned_names(node.target))
        else:
            continue
        if isinstance(value, ast.Call) and _is_workflow_constructor(value.func):
            assignments.setdefault(id(value), set()).update(names)

    calls = {
        id(node): node
        for node in ast.walk(module)
        if isinstance(node, ast.Call) and _is_workflow_constructor(node.func)
    }
    result: list[tuple[str, str | None]] = []
    for call_id, names in assignments.items():
        call = calls[call_id]
        workflow_id = _literal_workflow_id(call)
        result.extend((name, workflow_id) for name in names)
    return result


def _assigned_names(target: ast.expr) -> set[str]:
    if isinstance(target, ast.Name):
        return {target.id}
    if isinstance(target, (ast.Tuple, ast.List)):
        return {name for item in target.elts for name in _assigned_names(item)}
    return set()


def _is_workflow_constructor(function: ast.expr) -> bool:
    if isinstance(function, ast.Name):
        return function.id == "Workflow"
    return isinstance(function, ast.Attribute) and function.attr == "Workflow"


def _literal_workflow_id(call: ast.Call) -> str | None:
    keyword_values = [keyword.value for keyword in call.keywords if keyword.arg == "id"]
    if len(keyword_values) != 1 or not isinstance(keyword_values[0], ast.Constant):
        return None
    value = keyword_values[0].value
    if isinstance(value, str) and value and len(value) <= _MAX_WORKFLOW_ID_LENGTH:
        return value
    return None


def _require_configured_workflow_id(
    configured_id: str | None,
    source_file: Path,
    cause: BaseException | None = None,
) -> str:
    if configured_id is not None:
        return configured_id
    error = ValueError(
        f"Could not resolve a literal Workflow(id=...) in {source_file}. "
        "Set [tool.zygo.cloud].workflow_id in pyproject.toml."
    )
    if cause is not None:
        raise error from cause
    raise error


def _build_source_archive(project_dir: Path) -> tuple[bytes, set[str]]:
    ignore_lines = list(_BUILTIN_IGNORES)
    gitignore = project_dir / ".gitignore"
    if gitignore.is_file():
        ignore_lines.extend(gitignore.read_text(encoding="utf-8").splitlines())
    ignore_spec = GitIgnoreSpec.from_lines(ignore_lines)

    files: list[tuple[str, Path]] = []
    for directory, child_dirs, filenames in project_dir.walk(top_down=True):
        relative_dir = directory.relative_to(project_dir)
        child_dirs[:] = [
            name
            for name in child_dirs
            if not ignore_spec.match_file(
                (relative_dir / name).as_posix().rstrip("/") + "/"
            )
        ]
        for filename in filenames:
            path = directory / filename
            relative_path = path.relative_to(project_dir).as_posix()
            if (
                ignore_spec.match_file(relative_path)
                or path.is_symlink()
                or not path.is_file()
            ):
                continue
            files.append((relative_path, path))

    for required_name in ("pyproject.toml", "uv.lock"):
        required_path = project_dir / required_name
        if not any(relative_path == required_name for relative_path, _ in files):
            files.append((required_name, required_path))

    archive_buffer = BytesIO()
    with ZipFile(archive_buffer, mode="w", compression=ZIP_DEFLATED) as archive:
        for relative_path, path in sorted(files):
            info = ZipInfo(relative_path, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = ZIP_DEFLATED
            info.external_attr = stat.S_IFREG | (0o644 << 16)
            archive.writestr(info, path.read_bytes())

    source_archive = archive_buffer.getvalue()
    if len(source_archive) > _MAX_SOURCE_ARCHIVE_BYTES:
        raise ValueError(
            "Workflow source archive exceeds the 50,000,000 byte Zygo Cloud limit"
        )
    return source_archive, {relative_path for relative_path, _ in files}


def _table(value: object) -> dict[str, object] | None:
    return cast("dict[str, object]", value) if isinstance(value, dict) else None
