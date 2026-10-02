from fsspec import register_implementation  # type: ignore


from zygo.ml import (
    ClassLabel,
    Dataset,
    Image,
    Model,
    ModelStore,
    TrainingContext,
)
from zygo.ml.features import Features, features
from zygo.store import DataUri
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
    "TrainingContext",
    "Workflow",
    "features",
]
