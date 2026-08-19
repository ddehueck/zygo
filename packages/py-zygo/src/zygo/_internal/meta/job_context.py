from __future__ import annotations

from typing import override

from zygo.context import JobContext, TagsProtocol
from zygo.store._internal.impl import StoreImpl


class JobContextImpl(JobContext):
    def __init__(self, *, store: StoreImpl) -> None:
        super().__init__()
        self.store = store
        self.tags = TagsImpl()


# NB: This is just a dummy implementation of TagsProtocol for testing purposes.
class TagsImpl(TagsProtocol):
    def __init__(self) -> None:
        super().__init__()
        self._tags: dict[str, str] = {}

    @override
    def add(self, name: str, value: str) -> None:
        raise NotImplementedError
