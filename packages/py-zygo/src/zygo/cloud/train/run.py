"""Upload source code and follow a cloud training run."""

from __future__ import annotations

from typing import TYPE_CHECKING

from zygo.cloud.train.api import StartModelTrainingRunRequest
from zygo.cloud.train.client import TrainingClient
from zygo.cloud.train.display import (
    follow_run,
    format_archive_size,
    format_source_dir,
    make_console,
    progress,
    show_excluded,
)
from zygo.cloud.train.paths import require_target, resolve_source_path, source_directory
from zygo.cloud.train.utils import (
    build_archive,
    request_params,
    require_api_key,
    require_dataset_id,
    require_model_id,
)

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence
    from pathlib import Path

    from zygo.cloud.train.api import ModelTrainingRun
    from zygo.ml.hyperparams import HyperParams
    from zygo.ml.model import Model


def train(  # ruff: ignore[too-many-arguments]
    model: Model,
    dataset_id: str,
    params: HyperParams | Mapping[str, object] | None = None,
    *,
    source: str | Path | None = None,
    exclude: str | Sequence[str] = (),
    target: str,
    api_key: str | None = None,
) -> ModelTrainingRun:
    """
    Uploads the source code and starts a cloud training run.
    Resulting logs are streamed to the console.

    Parameters:
        model: The model to train. Its name is the workspace model id.
        dataset_id: Ready dataset version id, such as ``dsv_...``.
        params: Hyperparameters checked against the model and included in
            the runtime request. Omitted values use the declared defaults.
        source: Directory zipped as the Docker build context. A relative
            path is resolved from the file that calls ``train``, not from the
            working directory. The directory must contain a Dockerfile,
            pyproject.toml, and uv.lock, and those files must be included in
            the archive. Their paths are the image definition. When omitted,
            the directory comes from the project config or the working
            directory.
        exclude: Extra gitignore patterns left out of the uploaded archive.
            These apply with ``.gitignore`` and the default tool directories.
            A string is one pattern. A sequence is several. Patterns are
            relative to the source root.
        target: Python module path the cloud runtime imports, such as
            ``main:app``.
        api_key: Zygo Cloud API key. Defaults to ZYGO_CLOUD_API_KEY env var.

    Returns:
        The finished model training run. A failed or cancelled status is
        included and is not raised.
    """
    console = make_console()
    # todo: simplify. this is confusing, there should be a single direct call and not these layers of indirection.
    resolved = resolve_source_path(source)
    directory = source_directory(resolved)
    source_label = format_source_dir(directory)
    with progress(console, f"Packing source code from {source_label}") as step:
        body_params = request_params(model._validate_params(params))
        archive, image, source_hash, excluded = build_archive(resolved, exclude=exclude)
        step.complete(f"Packed source code from {source_label}")
        show_excluded(console, excluded)

        client = TrainingClient(require_api_key(api_key))

        size = format_archive_size(len(archive))
        step.update(f"Uploading {size} of source code")

        source_id = client.ensure_source(archive, source_hash)
        step.complete(f"Uploaded {size} of source code")

        step.update("Starting training run")
        run = client.start_run(
            StartModelTrainingRunRequest(
                type="model_training_run",
                source_id=source_id,
                target=require_target(target),
                model_id=require_model_id(model),
                dataset_version_id=require_dataset_id(dataset_id),
                image=image,
                params=body_params,
            )
        )
        return follow_run(
            run,
            client.training_status,
            client.iter_build_logs,
            client.iter_logs,
            console,
            step=step,
        )
