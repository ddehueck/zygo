"""Persisted model artifacts addressed by a store prefix."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, BinaryIO, cast

import fsspec  # pyright: ignore[reportMissingTypeStubs]

from zygo.store import DataUri

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping
    from contextlib import AbstractContextManager


@dataclass(frozen=True)
class ModelBundle:
    """A reference to a stored artifact prefix, not a local directory or key.

    The URI must end in '/'. This reference is not part of the training
    or loading hook contract. The runtime selects and injects their stores.
    """

    uri: DataUri

    def __post_init__(self) -> None:
        if not isinstance(cast("object", self.uri), DataUri):
            raise TypeError("ModelBundle requires a DataUri")
        if not self.uri.uri.endswith("/") or not self.uri.path.endswith("/"):
            raise ValueError(
                "ModelBundle URI must be a prefix ending in '/', not a key"
            )

    def artifact(self, name: str) -> DataUri:
        """Address a named artifact within this prefix."""
        if "\\" in name or any(part in {"", ".", ".."} for part in name.split("/")):
            raise ValueError(
                "Artifact name must be a nonempty relative key without traversal"
            )
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
            fsspec.open,
        )
        return open_file(str(self.artifact(name)), "rb", **dict(storage_options or {}))
