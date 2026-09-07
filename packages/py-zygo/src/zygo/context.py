from typing import Protocol

from zygo.store import StoreProtocol


class TagsProtocol(Protocol):
    """A tag is an alpha-numeric string that can only contain the characters `a-z`, `A-Z`, `0-9`, `_`, and `-`. Tags are used to filter jobs in the workflow system."""
    def add(self, value: str) -> None: ...


class JobContext(Protocol):
    """Provides Zygo-specific helpers for interacting with the workflow system."""

    store: StoreProtocol
    """A Zygo-managed store for reading and writing workflow data."""

    tags: TagsProtocol
    """A tag manager for associating filterable tags with the job."""
