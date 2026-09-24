from fsspec import register_implementation  # type: ignore

from zygo.channel import Channel
from zygo.context import JobContext
from zygo.store import Reference
from zygo.workflow import Workflow

register_implementation("zygo", "zygo._internal.cloud.fs_backend.ZygoFileSystem")

__all__ = [
    "Channel",
    "JobContext",
    "Reference",
    "Workflow",
]
