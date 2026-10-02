from __future__ import annotations

from typing import TYPE_CHECKING, override

from zygo.cli.v0.types import TagInserted
from zygo.tags import TagsProtocol

if TYPE_CHECKING:
    from zygo.cli.v0.transport import IpcTransport


class TagsImpl(TagsProtocol):
    def __init__(self, *, ipc_transport: IpcTransport) -> None:
        super().__init__()
        self.ipc_transport = ipc_transport

    @override
    def add(self, value: str) -> None:
        cleaned_value = value.strip()
        self.ipc_transport.emit(TagInserted(type="tag_inserted", value=cleaned_value))
