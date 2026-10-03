"""In-process execution of registered model training and inference hooks."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING
from uuid import uuid4

from zygo.dataset.dataset import Dataset
from zygo.ml.store import ModelStore
from zygo.store import DataUri
from zygo.transport import LocalTransport

if TYPE_CHECKING:
    from collections.abc import Mapping

    from zygo.ml.hyperparams import HyperParams
    from zygo.ml.model import Model


@dataclass
class _TrainingContext:
    store: ModelStore


def train[T](
    *,
    model: Model,
    dataset: str | Path | DataUri | Dataset[T],
    params: HyperParams | Mapping[str, object] | None = None,
) -> ModelStore:
    """Run training synchronously and return its persistent artifact store.

    Hyperparameters may be an instance or a mapping. Omitted parameters use
    the declared type's field defaults. Paths and URIs are opened as Parquet
    datasets. Each invocation uses a
    unique store under ``./zygo/models/`` and leaves artifacts in place,
    including partial artifacts if the training hook raises an exception.
    """
    if isinstance(dataset, Path):
        training_dataset = Dataset.open(dataset.resolve().as_uri())
    elif isinstance(dataset, (str, DataUri)):
        training_dataset = Dataset.open(dataset)
    else:
        training_dataset = dataset

    root = Path.cwd() / "zygo" / "models" / uuid4().hex
    store = ModelStore(
        root=DataUri(root.as_uri() + "/"), ipc_transport=LocalTransport()
    )
    model.run_train(training_dataset, ctx=_TrainingContext(store=store), params=params)
    return store


def infer(*, model: Model, store: ModelStore, data: object) -> object:
    """Load a model from its store and return the inference hook's result.

    Input data is passed through unchanged, so callers supply the type
    expected by their registered inference hook. Each call loads the model
    anew. Loading and inference exceptions propagate to the caller.
    """
    loaded_model = model.run_load(store)
    return model.run_infer(loaded_model, data)
