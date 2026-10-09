from __future__ import annotations

from contextlib import contextmanager
from time import perf_counter
from typing import TYPE_CHECKING

from zygo.cli.importer import Importer
from zygo.cli.log import _zygo_logger
from zygo.cli.v0.transport import build_transport
from zygo.dataset import Dataset
from zygo.ml.executor import train as executor_train
from zygo.ml.model import Model
from zygo.ml.store import ModelStore
from zygo.store import DataUri

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping

    from zygo.cli.v0.transport import IpcTransport
    from zygo.cli.v0.types import ModelTrainCommand, StoreConfig


@contextmanager
def _timed(start: str, done: str) -> Generator[None, None, None]:
    _zygo_logger.info("%s", start)
    started = perf_counter()
    try:
        yield
    finally:
        _zygo_logger.info("%s in %.1fs", done, perf_counter() - started)


def execute(command: ModelTrainCommand) -> None:
    train(
        target=command.target,
        dataset_config=command.dataset_config,
        store_config=command.store_config,
        params=command.params,
        ipc_transport=build_transport(command.http_config),
    )


def train(
    *,
    target: str,
    dataset_config: StoreConfig,
    store_config: StoreConfig,
    ipc_transport: IpcTransport,
    params: Mapping[str, object] | None = None,
) -> None:
    with _timed("Loading model for training...", "Loaded model"):
        model = Importer.from_target(target).load(Model)

    with _timed("Loading dataset for training...", "Loaded dataset"):
        dataset = Dataset.open(
            dataset_config.root_uri, storage_options=dataset_config.kwargs
        )
    store = ModelStore(
        root=DataUri(store_config.root_uri),
        ipc_transport=ipc_transport,
        kwargs=store_config.kwargs,
    )
    with _timed("Starting model training...", "Finished model training"):
        executor_train(model=model, dataset=dataset, store=store, params=params)
