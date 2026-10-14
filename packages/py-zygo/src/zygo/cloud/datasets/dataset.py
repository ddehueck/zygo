"""Dataset repository handle."""

from __future__ import annotations

import re

from zygo.cloud.datasets.editor import DatasetEditor

__all__ = ["Dataset"]

_SLUG_PATTERN = re.compile(r"^[a-z0-9_-]+$")


class Dataset:
    """A versioned data repository with dataset editing helpers."""

    def __init__(self, id: str) -> None:
        if not _SLUG_PATTERN.fullmatch(id):
            raise ValueError(
                "Dataset id must contain only lowercase letters, numbers, "
                "underscores, and hyphens"
            )
        self._id = id

    @property
    def id(self) -> str:
        """The dataset's repository slug."""

        return self._id

    @property
    def slug(self) -> str:
        """The dataset's repository slug."""

        return self._id

    def edit(self) -> DatasetEditor:
        """Create an editor that collects changes in a context manager."""

        return DatasetEditor()
