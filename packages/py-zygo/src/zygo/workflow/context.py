from __future__ import annotations

from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from zygo.tags import TagsProtocol
    from zygo.workflow.store import WorkflowStore


class JobContext(Protocol):
    """Provides Zygo-specific helpers for interacting with the workflow system."""

    store: WorkflowStore
    """A Zygo-managed store for reading and writing workflow data."""

    tags: TagsProtocol
    """A tag manager for associating filterable tags with the job."""
