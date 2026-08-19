from typing import Protocol

from zygo.store import StoreProtocol


class TagsProtocol(Protocol):
    def add(self, name: str, value: str) -> None: ...


class JobContext(Protocol):
    """Provides Zygo-specific helpers for interacting with the workflow system."""

    store: StoreProtocol
    """A Zygo-managed store for reading and writing workflow data."""

    tags: TagsProtocol
    """A tag manager for associating filterable tags with the job."""
