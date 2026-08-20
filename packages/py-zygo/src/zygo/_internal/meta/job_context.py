from __future__ import annotations

from typing import override

from zygo._internal.ipc.v0.types import TagInserted, write_stdout_ipc_message
from zygo.context import JobContext, TagsProtocol
from zygo.store._internal.impl import StoreImpl


class JobContextImpl(JobContext):
    def __init__(self, *, store: StoreImpl) -> None:
        super().__init__()
        self.store = store
        self.tags = TagsImpl()


class TagsImpl(TagsProtocol):
    @override
    def add(self, name: str, value: str) -> None:
        write_stdout_ipc_message(TagInserted(name=name, value=value))
