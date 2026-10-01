"""Persisted model artifacts addressed by a store prefix."""

from __future__ import annotations

from contextlib import AbstractContextManager
from dataclasses import dataclass
from typing import TYPE_CHECKING, BinaryIO, cast

import fsspec  # pyright: ignore[reportMissingTypeStubs]

from zygo.store import DataUri

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping


@dataclass(frozen=True)
class ModelBundle:
    """A reference to a stored artifact prefix, not a local directory or key.

    The URI must end in '/'. During training, use TrainingContext.bundle() to
    create this reference only after writing artifacts through ctx.store.
    Direct construction restores references for already-persisted bundles.
    """

    uri: DataUri

    def __post_init__(self) -> None:
        if not isinstance(cast("object", self.uri), DataUri):
            raise TypeError("ModelBundle requires a DataUri")
        if not self.uri.uri.endswith("/") or not self.uri.path.endswith("/"):
            raise ValueError("ModelBundle URI must be a prefix ending in '/', not a key")

    def artifact(self, name: str) -> DataUri:
        """Address a named artifact within this prefix."""
        if "\\" in name or any(part in {"", ".", ".."} for part in name.split("/")):
            raise ValueError("Artifact name must be a nonempty relative key without traversal")
        return DataUri(f"{self.uri}{name}")

    def open(
        self,
        name: str,
        *,
        storage_options: Mapping[str, object] | None = None,
    ) -> AbstractContextManager[BinaryIO]:
        """Open a stored artifact for binary reading using its fsspec backend."""
        open_file = cast(
            "Callable[..., AbstractContextManager[BinaryIO]]",
            fsspec.open,  # pyright: ignore[reportUnknownMemberType]
        )
        return open_file(str(self.artifact(name)), "rb", **dict(storage_options or {}))
