"""Shared transports for in-process Zygo execution."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

    from zygo.cli.v0.types import IpcMessage


class LocalTransport:
    """Discard IPC events when running without an orchestrator."""

    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None:
        del messages
