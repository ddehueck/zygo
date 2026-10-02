"""Model command registration and the training executor boundary."""

from __future__ import annotations

from typing import TYPE_CHECKING

from zygo.cli.v0.arguments import (
    add_publication_options,
    parse_dataset_config,
    parse_store_config,
)
from zygo.cli.v0.transport import build_transport
from zygo.cli.v0.types import DatasetConfig, StoreConfig

if TYPE_CHECKING:
    import argparse

    from zygo.cli.v0.arguments import IpcArguments
    from zygo.cli.v0.transport import IpcTransport


def configure_parser(parser: argparse.ArgumentParser) -> None:
    commands = parser.add_subparsers(dest="command", required=True)
    train_parser = commands.add_parser("train", help="Train a model (executor pending)")
    train_parser.add_argument(
        "target",
        help="Model import target, for example myproject.models:classifier",
    )
    train_parser.add_argument(
        "--dataset-config",
        required=True,
        type=parse_dataset_config,
        metavar="JSON",
        help="Dataset URI and optional backend keyword arguments as JSON",
    )
    train_parser.add_argument(
        "--store-config",
        required=True,
        type=parse_store_config,
        metavar="JSON",
        help="Output model store root URI and optional backend keyword arguments as JSON",
    )
    add_publication_options(train_parser)


def execute(args: IpcArguments) -> None:
    if args.command != "train":
        raise ValueError(f"Unsupported model command: {args.command}")
    dataset_config = args.dataset_config
    store_config = args.store_config
    if not isinstance(dataset_config, DatasetConfig):
        raise TypeError("Model training requires DatasetConfig")
    if not isinstance(store_config, StoreConfig):
        raise TypeError("Model training requires StoreConfig")
    train(
        target=args.target,
        dataset_config=dataset_config,
        store_config=store_config,
        ipc_transport=build_transport(args.http_config),
    )


def train(
    *,
    target: str,
    dataset_config: DatasetConfig,
    store_config: StoreConfig,
    ipc_transport: IpcTransport,
) -> None:
    """Execute training once the model runtime is implemented."""
    del target, dataset_config, store_config, ipc_transport
    raise NotImplementedError("The model training executor is not implemented yet")
