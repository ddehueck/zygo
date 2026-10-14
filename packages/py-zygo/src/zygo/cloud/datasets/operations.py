"""Typed operations produced by dataset edits."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from pathlib import Path

__all__ = [
    "AddFileOperation",
    "Operation",
    "RemoveDirOperation",
    "RemoveFileOperation",
]


@dataclass(frozen=True)
class AddFileOperation:
    """Add a local file's contents at a path in a dataset."""

    type: Literal["add_file"] = field(default="add_file", init=False)
    path: str
    hash: str
    size: int
    source: Path


@dataclass(frozen=True)
class RemoveFileOperation:
    """Remove a file at a path in a dataset."""

    type: Literal["remove_file"] = field(default="remove_file", init=False)
    path: str


@dataclass(frozen=True)
class RemoveDirOperation:
    """Remove a directory and its entries from a dataset."""

    type: Literal["remove_dir"] = field(default="remove_dir", init=False)
    path: str


type Operation = AddFileOperation | RemoveFileOperation | RemoveDirOperation
