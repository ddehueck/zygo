from __future__ import annotations

from typing import TYPE_CHECKING

from zygo.cli.importer import Importer
from zygo.cli.v0.transport import build_transport
from zygo.cli.v0.types import ModelTrainCommand
from zygo.dataset import Dataset
from zygo.ml.executor import train as executor_train
from zygo.ml.model import Model
from zygo.ml.store import ModelStore
from zygo.store import DataUri

if TYPE_CHECKING:
    from zygo.cli.v0.transport import IpcTransport
    from zygo.cli.v0.types import StoreConfig


def execute(command: ModelTrainCommand) -> None:
    train(
        target=command.target,
        dataset_config=command.dataset_config,
        store_config=command.store_config,
        ipc_transport=build_transport(command.http_config),
    )


def train(
    *,
    target: str,
    dataset_config: StoreConfig,
    store_config: StoreConfig,
    ipc_transport: IpcTransport,
) -> None:
    model = Importer.from_target(target).load(Model)
    dataset = Dataset.open(
        dataset_config.root_uri, storage_options=dataset_config.kwargs
    )
    store = ModelStore(
        root=DataUri(store_config.root_uri),
        ipc_transport=ipc_transport,
        kwargs=store_config.kwargs,
    )
    executor_train(model=model, dataset=dataset, store=store)
