from fsspec import register_implementation

from zygo.store import DataUri
from zygo.tags import TagsProtocol
from zygo.workflow import Channel, JobContext, Workflow

register_implementation(  # ruff: ignore[non-empty-init-module]
    "zygo", "zygo.cloud.fsspec_backend.ZygoFileSystem"
)

__all__ = [
    "Channel",
    "DataUri",
    "JobContext",
    "TagsProtocol",
    "Workflow",
]
