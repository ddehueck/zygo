from fsspec import register_implementation  # type: ignore

from zygo.dataset.dataset import Dataset
from zygo.dataset.features import ClassLabel, Features, Image, features
from zygo.ml import (
    Model,
    ModelStore,
    TrainingContext,
    infer,
    train,
)
from zygo.store import DataUri
from zygo.tags import TagsProtocol
from zygo.workflow import Channel, JobContext, Workflow

register_implementation(  # ruff: ignore[non-empty-init-module]
    "zygo", "zygo._internal.cloud.fsspec_backend.ZygoFileSystem"
)

__all__ = [
    "Channel",
    "ClassLabel",
    "DataUri",
    "Dataset",
    "Features",
    "Image",
    "JobContext",
    "Model",
    "ModelStore",
    "TagsProtocol",
    "TrainingContext",
    "Workflow",
    "features",
    "infer",
    "train",
]
