from dataclasses import dataclass

from zygo.types import ChannelName


@dataclass(frozen=True)
class Channel:
    """A named channel for message passing between tasks."""

    name: ChannelName
