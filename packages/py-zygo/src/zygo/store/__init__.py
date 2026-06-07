"""Store module for key-value storage with scope-based isolation."""

from zygo.store.protocol import StoreProtocol
from zygo.store.types import Reference, Scope, StoreOptions

__all__ = [
    "Reference",
    "Scope",
    "StoreOptions",
    "StoreProtocol",
]
