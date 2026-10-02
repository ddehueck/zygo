from __future__ import annotations

from typing import TYPE_CHECKING

from zygo._internal.tags import TagsImpl
from zygo.workflow.context import JobContext

if TYPE_CHECKING:
    from zygo.cli.v0.transport import IpcTransport
    from zygo.workflow.store import WorkflowStore


class JobContextImpl(JobContext):
    def __init__(self, *, store: WorkflowStore, ipc_transport: IpcTransport) -> None:
        super().__init__()
        self.store = store
        self.tags = TagsImpl(ipc_transport=ipc_transport)
