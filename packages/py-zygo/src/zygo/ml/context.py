from __future__ import annotations

from typing import Protocol

from zygo.ml.store import ModelStore


class TrainingContext(Protocol):
    """Runtime-injected model storage for one training execution at a time."""

    store: ModelStore
    """A Zygo-managed store for reading and writing workflow data."""
