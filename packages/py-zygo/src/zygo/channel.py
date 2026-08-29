from typing import override

from zygo.codecs import Codec
from zygo.types import ChannelId


class Channel[T]:
    """A workflow channel identified by a user-defined ID."""

    def __init__(self, id: str, codec: Codec[T]) -> None:
        super().__init__()
        self.id = ChannelId(id)
        self.codec = codec

    @property
    def value_type(self) -> type[T]:
        return self.codec.value_type

    @property
    def is_scalar(self) -> bool:
        return self.codec.value_type is not list

    @override
    def __repr__(self) -> str:
        return f"Channel(id={self.id!s}, codec={self.codec!r})"
