"""Local preparation shared by the training client and entrypoint."""

from __future__ import annotations

import hashlib
from typing import TYPE_CHECKING, cast

from zygo.cloud.api_key import load_api_key
from zygo.cloud.train.api import ImageDefinition
from zygo.cloud.train.paths import file_in_source, resolve_image_files
from zygo.cloud.train.source import SourcePackage

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path

    from zygo.ml.hyperparams import HyperParams
    from zygo.ml.model import Model

_MAX_ID_LENGTH = 255


def request_params(params: HyperParams | None) -> dict[str, object]:
    """Serialize validated hyperparameters for the runtime request."""
    if params is None:
        return {}
    return cast("dict[str, object]", params.model_dump(mode="json", warnings="error"))


def require_model_id(model: Model) -> str:
    """Return the workspace model id."""
    model_id = model.name.strip()
    if not model_id or len(model_id) > _MAX_ID_LENGTH:
        raise ValueError("Model name must be 1-255 characters after trimming")
    return model_id


def require_dataset_id(dataset_id: str) -> str:
    """Return a trimmed dataset version id."""
    version_id = dataset_id.strip()
    if not version_id or len(version_id) > _MAX_ID_LENGTH:
        raise ValueError("dataset_id must be 1-255 characters after trimming")
    return version_id


def require_api_key(api_key: str | None) -> str:
    """Use the explicit key, or the ZYGO_CLOUD_API_KEY environment variable."""
    if api_key is not None:
        return api_key
    loaded = load_api_key()
    if loaded is None:
        raise ValueError(
            "API key must be provided via 'api_key' or the ZYGO_CLOUD_API_KEY environment variable"
        )
    return loaded


def build_archive(
    source: Path | None = None,
    *,
    exclude: str | Sequence[str] = (),
) -> tuple[bytes, ImageDefinition, str, tuple[str, ...]]:
    """Zip the source and name the image files inside it.

    Returns the archive bytes, the image definition, the archive hash, and
    the paths left out of the archive. The image definition is the paths of
    the Dockerfile, pyproject.toml, and uv.lock. The archive is the Docker
    build context.
    """
    root, dockerfile, pyproject, uv_lock = resolve_image_files(source)
    dockerfile = file_in_source(root, dockerfile, label="Dockerfile")
    pyproject = file_in_source(root, pyproject, label="pyproject.toml")
    uv_lock = file_in_source(root, uv_lock, label="uv.lock")
    package = SourcePackage(root, excludes=_exclude_patterns(exclude))
    included = {path.relative_to(root).as_posix() for path in package.files()}
    for label, relative in (
        ("Dockerfile", dockerfile),
        ("pyproject.toml", pyproject),
        ("uv.lock", uv_lock),
    ):
        if relative not in included:
            raise ValueError(
                f"{label} is not included in the source archive: {relative}"
            )
    image = ImageDefinition(dockerfile=dockerfile, pyproject=pyproject, uv_lock=uv_lock)
    archive = package.archive()
    return (
        archive,
        image,
        hashlib.sha256(archive).hexdigest(),
        tuple(package.excluded()),
    )


def _exclude_patterns(exclude: str | Sequence[str]) -> tuple[str, ...]:
    """Return gitignore patterns from one pattern or a sequence of them."""
    if isinstance(exclude, str):
        return (exclude,) if exclude else ()
    return tuple(pattern for pattern in exclude if pattern)
