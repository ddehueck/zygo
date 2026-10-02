from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path
from types import TracebackType
from typing import Self

import pyarrow as pa
import pyarrow.parquet as pq

from zygo.dataset.features import Features

DEFAULT_SHARD_SIZE = 10_000


class DatasetBuilder:
    """Write validated Python rows to numbered Parquet shards in a local directory."""

    def __init__(
        self,
        path: Path | str,
        *,
        schema: type[Features],
        shard_size: int = DEFAULT_SHARD_SIZE,
        overwrite: bool = False,
    ) -> None:
        if type(shard_size) is not int or shard_size <= 0:
            raise ValueError("shard_size must be a positive integer row count")
        self.schema = schema
        self.path = Path(path)
        self.shard_size = shard_size
        self.overwrite = overwrite
        self._arrow_schema = schema.to_schema()
        self._rows: list[dict[str, object]] = []
        self._shard_index = 0
        self._active = False
        self._closed = False

    def __enter__(self) -> Self:
        if self._active or self._closed:
            raise RuntimeError("DatasetBuilder cannot be reused")

        self.path.mkdir(parents=True, exist_ok=True)

        has_existing_files = any(self.path.iterdir())
        if not self.overwrite and has_existing_files:
            raise FileExistsError(f"Output directory {self.path} must be empty")

        self._active = True
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        if exc_type is None:
            self.close()
        else:
            self._rows.clear()
            self._active = False
            self._closed = True

    def add(self, data: object) -> None:
        """Validate and buffer one mapping or object with feature attributes."""
        if not self._active:
            raise RuntimeError("Use DatasetBuilder inside a with block")
        row = self.schema.encode(data)
        # Catch storage errors such as integer overflow before buffering the row.
        pa.Table.from_pylist([row], schema=self._arrow_schema)
        self._rows.append(row)
        if len(self._rows) >= self.shard_size:
            self._write_shard()

    def add_many(self, data: Iterable[object]) -> None:
        """Add rows incrementally without materializing the input iterable."""
        for row in data:
            self.add(row)

    def close(self) -> None:
        """Write remaining rows, or an empty first shard, and close the builder."""
        if self._closed:
            return
        if not self._active:
            raise RuntimeError("Use DatasetBuilder inside a with block")
        try:
            if self._rows or self._shard_index == 0:
                self._write_shard()
        finally:
            self._active = False
            self._closed = True

    def _write_shard(self) -> None:
        table = pa.Table.from_pylist(self._rows, schema=self._arrow_schema)
        path = self.path / f"{self._shard_index + 1:04d}.parquet"

        mode = "wb" if self.overwrite else "xb"
        with path.open(mode) as output:
            pq.write_table(table, output)
        self._rows.clear()
        self._shard_index += 1
