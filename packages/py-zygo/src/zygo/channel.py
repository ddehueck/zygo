from dataclasses import dataclass

from zygo.types import ChannelId


@dataclass(frozen=True)
class Channel:
    """A workflow channel identified by a user-defined ID."""

    id: ChannelId
