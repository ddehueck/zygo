"""Plain model artifact storage."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, override

from zygo.store._internal.base import BaseStore
from zygo.store.types import DataUri

if TYPE_CHECKING:
    from types import TracebackType


class ModelStore(BaseStore):
    """
    A persistent model directory to store model artifacts.
    """
    pass
