from fsspec import register_implementation

from zygo.store import DataUri
from zygo.tags import TagsProtocol
from zygo.workflow import Channel, JobContext, Workflow
from zygo.workflow.codecs import Csv, Txt

register_implementation(  # ruff: ignore[non-empty-init-module]
    "zygo", "zygo.cloud.fsspec_backend.ZygoFileSystem"
)

__all__ = [
    "Channel",
    "Csv",
    "DataUri",
    "JobContext",
    "TagsProtocol",
    "Txt",
    "Workflow",
]
