"""Resolve the source directory and the cloud runtime target."""

from __future__ import annotations

import inspect
from pathlib import Path
from typing import TYPE_CHECKING

from zygo.cloud.config import from_toml

if TYPE_CHECKING:
    from zygo.cloud.config import ZygoCloudProjectConfig

_MAX_ID_LENGTH = 255


def require_target(target: str) -> str:
    """Return a trimmed Python module path for the cloud runtime."""
    module_path = target.strip()
    if not module_path or len(module_path) > _MAX_ID_LENGTH:
        raise ValueError("target must be 1-255 characters after trimming")
    return module_path


def resolve_source_path(source: str | Path | None) -> Path | None:
    """Resolve the source directory passed to ``train``.

    ``train`` calls this directly. A relative path is resolved from the
    file that contains that call, not from the process working directory.
    An absolute path is used as given. ``None`` keeps the project-config
    or working-directory default.
    """
    if source is None:
        return None
    path = Path(source)
    if path.is_absolute():
        return path.resolve()
    return (_caller_directory() / path).resolve()


def source_directory(source: Path | None = None) -> Path:
    """Return the directory that will be packed and uploaded."""
    root, _dockerfile, _pyproject, _uv_lock = resolve_image_files(source)
    return root


def resolve_image_files(source: Path | None = None) -> tuple[Path, str, str, str]:
    """Return the source directory and the image files inside it.

    The paths are the Dockerfile, ``pyproject.toml``, and ``uv.lock``
    relative to that directory. Those paths are the image definition, and
    the server reads the files from the ZIP. An explicit source directory
    uses the default names. Otherwise the project config can name the
    Dockerfile and lockfile.
    """
    if source is not None:
        return source, "Dockerfile", "pyproject.toml", "uv.lock"
    config = _project_config()
    if config is None:
        return Path.cwd().resolve(), "Dockerfile", "pyproject.toml", "uv.lock"
    return (
        config.src_dir.resolve(),
        config.dockerfile or "Dockerfile",
        "pyproject.toml",
        config.uv_lockfile,
    )


def file_in_source(root: Path, relative_path: str, *, label: str) -> str:
    """Return a relative path when that file sits inside the source directory."""
    relative = Path(relative_path)
    if (
        relative.is_absolute()
        or "\\" in relative_path
        or any(part in {"", ".", ".."} for part in relative.parts)
    ):
        raise ValueError(f"{label} must be a relative path inside the source directory")
    if not (root / relative).is_file():
        raise ValueError(
            f"{label} was not found in the source directory: {relative_path}"
        )
    return relative.as_posix()


def _caller_directory() -> Path:
    """Return the directory of the file that called ``train``.

    ``prepare_source`` calls this, and ``train`` calls ``prepare_source``,
    so the user's file is three frames up.
    """
    frame = inspect.currentframe()
    caller = None
    try:
        current = frame
        # Walk past _caller_directory, prepare_source, and train.
        # This will have to changed if we add more frames by adding function calls.
        for _ in range(3):
            current = None if current is None else current.f_back
        caller = current
        if caller is None:
            filename = ""
        else:
            written = caller.f_globals.get("__file__")
            filename = (
                written if isinstance(written, str) else caller.f_code.co_filename
            )
    finally:
        del caller, frame
    if not filename or filename.startswith("<"):
        raise ValueError("A relative source path must be passed from a Python file")
    return Path(filename).resolve().parent


def _project_config() -> ZygoCloudProjectConfig | None:
    path = Path.cwd() / "pyproject.toml"
    if not path.is_file():
        return None
    try:
        return from_toml(str(path))
    except KeyError:
        return None
