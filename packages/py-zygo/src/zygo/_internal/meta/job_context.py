from __future__ import annotations

from typing import TYPE_CHECKING, override

from zygo._internal.ipc.v0.types import TagInserted
from zygo.context import JobContext, TagsProtocol

if TYPE_CHECKING:
    from zygo._internal.ipc.v0.transport import IpcTransport
    from zygo.store._internal.impl import StoreImpl


class JobContextImpl(JobContext):
    def __init__(self, *, store: StoreImpl, ipc_transport: IpcTransport) -> None:
        super().__init__()
        self.store = store
        self.tags = TagsImpl(ipc_transport=ipc_transport)


class TagsImpl(TagsProtocol):
    def __init__(self, *, ipc_transport: IpcTransport) -> None:
        self.ipc_transport = ipc_transport

    @override
    def add(self, value: str) -> None:
        cleaned_value = value.strip()
        self.ipc_transport.emit(TagInserted(value=cleaned_value))
