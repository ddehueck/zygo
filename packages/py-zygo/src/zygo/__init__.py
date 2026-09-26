from fsspec import register_implementation  # type: ignore

from zygo.channel import Channel
from zygo.context import JobContext
from zygo.store import DataUri
from zygo.workflow import Workflow

register_implementation(  # ruff: ignore[non-empty-init-module]
    "zygo", "zygo._internal.cloud.fsspec_backend.ZygoFileSystem"
)

__all__ = [
    "Channel",
    "DataUri",
    "JobContext",
    "Workflow",
]
