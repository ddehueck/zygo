from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Self

import pyarrow as pa
import pyarrow.parquet as pq

if TYPE_CHECKING:
    from collections.abc import Iterable, Iterator
    from types import TracebackType

    from zygo.dataset.features import Features

DEFAULT_SHARD_SIZE_MB = 512
BYTES_PER_MB = 1_000_000


def shard_size_bytes(shard_size_mb: int) -> int:
    if type(shard_size_mb) is not int or shard_size_mb <= 0:
        raise ValueError("shard_size_mb must be a positive integer number of MB")
    return shard_size_mb * BYTES_PER_MB


def _rows_for_bytes(batch: pa.RecordBatch, target_bytes: int) -> int:
    if batch.nbytes < target_bytes:
        return batch.num_rows
    low, high = 1, batch.num_rows
    while low < high:
        middle = (low + high) // 2
        if batch.slice(0, middle).nbytes >= target_bytes:
            high = middle
        else:
            low = middle + 1
    return low


def shard_tables(
    batches: Iterable[pa.RecordBatch], *, max_bytes: int
) -> Iterator[pa.Table]:

    pending: list[pa.RecordBatch] = []
    size = 0
    for batch in batches:
        offset = 0
        while offset < batch.num_rows:
            remaining = batch.slice(offset)
            count = _rows_for_bytes(remaining, max_bytes - size)
            chunk = remaining.slice(0, count)
            pending.append(chunk)
            size += chunk.nbytes
            offset += count
            if size >= max_bytes:
                yield pa.Table.from_batches(pending)
                pending = []
                size = 0
    if pending:
        yield pa.Table.from_batches(pending)


class DatasetBuilder:
    """Write validated Python rows to numbered Parquet shards in a local directory."""

    def __init__(
        self,
        path: Path | str,
        *,
        schema: type[Features],
        shard_size_mb: int = DEFAULT_SHARD_SIZE_MB,
        overwrite: bool = False,
    ) -> None:
        self._max_shard_bytes = shard_size_bytes(shard_size_mb)
        self.schema = schema
        self.path = Path(path)
        self.shard_size_mb = shard_size_mb
        self.overwrite = overwrite
        self._arrow_schema = schema.to_schema()
        self._rows: list[dict[str, object]] = []
        self._rows_bytes = 0
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
        table = pa.Table.from_pylist([row], schema=self._arrow_schema)
        self._rows.append(row)
        self._rows_bytes += table.nbytes
        if self._rows_bytes >= self._max_shard_bytes:
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
        self._rows_bytes = 0
        self._shard_index += 1
