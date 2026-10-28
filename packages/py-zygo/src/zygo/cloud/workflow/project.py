"""Resolve and package a local workflow deployment project."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
import re
import stat
import tomllib
from typing import TYPE_CHECKING, cast
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

from pathspec import GitIgnoreSpec
from pydantic import TypeAdapter, ValidationError

from zygo.cloud.api import PythonTarget, PythonVersion, WorkflowId

if TYPE_CHECKING:
    from zygo.workflow import Workflow

_MAX_SOURCE_ARCHIVE_BYTES = 50_000_000
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
_PYTHON_TARGET_ADAPTER: TypeAdapter[PythonTarget] = TypeAdapter(PythonTarget)
_PYTHON_VERSION_ADAPTER: TypeAdapter[PythonVersion] = TypeAdapter(PythonVersion)
_WORKFLOW_ID_ADAPTER: TypeAdapter[WorkflowId] = TypeAdapter(WorkflowId)

type DeploymentModuleContext = tuple[
    str | None,
    str | None,
    Mapping[str, object],
]


@dataclass(frozen=True)
class DeploymentProject:
    """Validated inputs needed to create a cloud deployment."""

    python_version: PythonVersion
    workflow_id: str
    target: str
    pyproject_toml: bytes
    uv_lock: bytes
    source_archive: bytes


def infer_workflow_target(
    project_dir: str | Path,
    workflow: Workflow,
    module_contexts: tuple[DeploymentModuleContext, ...],
) -> tuple[str, Path]:
    """Infer a Python target from the module binding of a workflow instance."""

    root = Path(project_dir).expanduser().resolve()
    if not root.is_dir():
        raise ValueError(f"project_dir must be an existing directory: {root}")
    for module_name, module_file, namespace in module_contexts:
        if module_file is None:
            continue
        source_file = Path(module_file).expanduser().resolve()
        try:
            source_file.relative_to(root)
        except ValueError:
            continue
        if not source_file.is_file():
            continue

        bindings = sorted(
            name
            for name, value in namespace.items()
            if value is workflow and name.isidentifier()
        )
        if not bindings:
            continue

        resolved_module = _module_name_for_file(root, source_file, module_name)
        if resolved_module is None:
            continue
        conventional_names = [
            name for name in workflow.conventional_names if name in bindings
        ]
        attribute = conventional_names[0] if conventional_names else bindings[0]
        target = _validate_target(f"{resolved_module}:{attribute}")
        target_file = _resolve_target_file(root, target, expected_file=source_file)
        return target, target_file

    raise ValueError(
        "Could not infer an import target for this workflow. Define it as a "
        "module-level variable in a Python file under project_dir, then pass that "
        "Workflow instance to deploy_workflow."
    )


def resolve_deployment_project(
    project_dir: str | Path,
    workflow: Workflow,
    target: str,
    target_file: Path,
) -> DeploymentProject:
    """Validate project files and package the source for deployment."""

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
    workflow_id = _validate_workflow_id(str(workflow.id))
    source_archive, archived_paths = _build_source_archive(root)
    target_relative_path = target_file.relative_to(root).as_posix()
    if target_relative_path not in archived_paths:
        raise ValueError(
            f"Workflow target {target!r} is excluded from the source archive by an ignore rule"
        )

    return DeploymentProject(
        python_version=python_version,
        workflow_id=workflow_id,
        target=target,
        pyproject_toml=pyproject_path.read_bytes(),
        uv_lock=uv_lock_path.read_bytes(),
        source_archive=source_archive,
    )


def _module_name_for_file(
    project_dir: Path, source_file: Path, declared_module: str | None
) -> str | None:
    if declared_module and declared_module != "__main__":
        try:
            resolved_file = _resolve_target_file(
                project_dir, declared_module, expected_file=source_file
            )
        except (FileNotFoundError, ValueError):
            resolved_file = None
        if resolved_file == source_file:
            return declared_module

    return _module_name_from_path(project_dir, source_file)


def _module_name_from_path(project_dir: Path, source_file: Path) -> str | None:
    parts = list(source_file.relative_to(project_dir).parts)
    if parts and parts[0] == "src":
        parts.pop(0)
    if not parts:
        return None
    if parts[-1] == "__init__.py":
        parts.pop()
    else:
        module_file = Path(parts[-1])
        if module_file.suffix != ".py":
            return None
        parts[-1] = module_file.stem
    if not parts:
        return None

    module_name = ".".join(parts)
    try:
        resolved_file = _resolve_target_file(
            project_dir, module_name, expected_file=source_file
        )
    except (FileNotFoundError, ValueError):
        return None
    return module_name if resolved_file == source_file else None


def _resolve_target_file(
    project_dir: Path,
    target: str,
    *,
    expected_file: Path | None = None,
) -> Path:
    validated_target = _validate_target(target)
    module_name = validated_target.partition(":")[0]
    module_path = Path(*module_name.split("."))
    candidates: set[Path] = set()
    for import_root in (project_dir, project_dir / "src"):
        source_file = import_root / module_path.with_suffix(".py")
        package_file = import_root / module_path / "__init__.py"
        if source_file.is_file() and not source_file.is_symlink():
            candidates.add(source_file.resolve())
        if package_file.is_file() and not package_file.is_symlink():
            candidates.add(package_file.resolve())

    if expected_file is not None and expected_file in candidates:
        return expected_file
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


def _validate_target(target: str) -> str:
    try:
        return _PYTHON_TARGET_ADAPTER.validate_python(target)
    except ValidationError as error:
        raise ValueError(
            "The inferred workflow target must be a Python module target, "
            "such as 'workflow:workflow'"
        ) from error


def _validate_workflow_id(workflow_id: str) -> str:
    try:
        return _WORKFLOW_ID_ADAPTER.validate_python(workflow_id)
    except ValidationError as error:
        raise ValueError(
            "workflow.id must be a non-empty string up to 255 characters"
        ) from error


def _read_toml(path: Path) -> dict[str, object]:
    try:
        payload = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ValueError(f"Could not read {path}: {error}") from error
    return cast("dict[str, object]", payload)


def _cloud_python_version(
    config: dict[str, object], pyproject_path: Path
) -> PythonVersion:
    tool = _table(config.get("tool"))
    zygo = _table(tool.get("zygo")) if tool is not None else None
    cloud = _table(zygo.get("cloud")) if zygo is not None else None
    project_version = (
        _parse_python_version(
            cloud["python_version"],
            f"{pyproject_path}: [tool.zygo.cloud].python_version",
        )
        if cloud is not None and "python_version" in cloud
        else None
    )
    python_version_path = pyproject_path.parent / ".python-version"
    pinned_version = _parse_python_version_file(python_version_path)

    if (
        project_version is not None
        and pinned_version is not None
        and project_version != pinned_version
    ):
        raise ValueError(
            f"Python version mismatch: {pyproject_path} configures "
            f"{project_version}, but {python_version_path} resolves to "
            f"{pinned_version}. Keep these settings aligned."
        )
    if project_version is not None:
        return project_version
    if pinned_version is not None:
        return pinned_version
    raise ValueError(
        f"{pyproject_path}: set [tool.zygo.cloud].python_version or add a "
        f"supported Python version to {python_version_path}"
    )


def _parse_python_version(value: object, source: str) -> PythonVersion:
    try:
        return _PYTHON_VERSION_ADAPTER.validate_python(value)
    except ValidationError as error:
        raise ValueError(
            f"{source} must resolve to a supported Python version "
            "(3.11, 3.12, 3.13, 3.14, or 3.15)"
        ) from error


def _parse_python_version_file(path: Path) -> PythonVersion | None:
    if path.is_symlink():
        raise ValueError(f"{path} must be a regular project file, not a symlink")
    if not path.exists():
        return None
    if not path.is_file():
        raise ValueError(f"{path} must be a regular project file")

    try:
        value = path.read_text(encoding="utf-8").strip()
    except OSError as error:
        raise ValueError(f"Could not read {path}: {error}") from error

    match = re.fullmatch(r"(3\.\d+)(?:\.\d+)?", value)
    if match is None:
        raise ValueError(
            f"{path} must contain one Python version, such as '3.12' or '3.12.4'"
        )
    return _parse_python_version(match.group(1), str(path))


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
