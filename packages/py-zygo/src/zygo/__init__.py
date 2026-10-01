from fsspec import register_implementation  # type: ignore

from zygo.channel import Channel
from zygo.context import JobContext
from zygo.ml import (
    ClassLabel,
    Dataset,

    Image,
    Model,
    ModelBundle,
    Struct,
    TrainingContext,
)
from zygo.ml.features import Features, features
from zygo.store import DataUri
from zygo.workflow import Workflow

register_implementation(  # ruff: ignore[non-empty-init-module]
    "zygo", "zygo._internal.cloud.fsspec_backend.ZygoFileSystem"
)

__all__ = [
    "Channel",
    "ClassLabel",
    "DataUri",
    "Dataset",
    "features",
    "Features",
    "Image",
    "JobContext",
    "Model",
    "ModelBundle",
    "Struct",
    "TrainingContext",
    "Workflow",
]
