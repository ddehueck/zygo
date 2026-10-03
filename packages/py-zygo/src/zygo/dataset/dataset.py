from __future__ import annotations

import json
from pathlib import Path
import random
from typing import TYPE_CHECKING, cast, overload

from fsspec.core import url_to_fs
import pyarrow as pa
import pyarrow.dataset as pads
from pyarrow.fs import FSSpecHandler, PyFileSystem
import pyarrow.parquet as pq

from zygo.dataset.builder import (
    DEFAULT_SHARD_SIZE_MB,
    DatasetBuilder,
    shard_size_bytes,
    shard_tables,
)
from zygo.dataset.features import ClassLabel, Features

if TYPE_CHECKING:
    from collections.abc import Callable, Iterator, Mapping

    from zygo.store import DataUri


class Dataset[T]:
    """A Parquet file or sharded directory, optionally decoded with Features.

    URIs use the same fsspec backends as the workflow store. Opening reads
    metadata and validates features, without loading all rows into memory.
    """

    def __init__(
        self,
        source: pads.Dataset,
        *,
        uri: str,
        features: type[T] | None = None,
        _batch_reader: Callable[[list[str] | None], Iterator[pa.RecordBatch]]
        | None = None,
        _row_count: int | None = None,
    ) -> None:
        self._source: pads.Dataset = source
        self.uri = uri
        self._features = features
        self._batch_reader = _batch_reader
        self._row_count = _row_count

    @classmethod
    def builder(
        cls,
        output_path: str | Path,
        *,
        schema: type[Features],
        shard_size_mb: int = DEFAULT_SHARD_SIZE_MB,
        overwrite: bool = False,
    ) -> DatasetBuilder:
        """
        Build a local dataset directory.
        e.g.

        >>> with Dataset.builder("mydataset/", schema=Features) as builder:
        ...     builder.add({"image": image, "label": 0})
        """
        return DatasetBuilder(
            output_path, schema=schema, shard_size_mb=shard_size_mb, overwrite=overwrite
        )

    @property
    def features(self) -> type[T] | None:
        return self._features

    @overload
    @classmethod
    def open(
        cls,
        uri: str | DataUri,
        *,
        features: None = None,
        storage_options: Mapping[str, object] | None = None,
    ) -> Dataset[dict[str, object]]: ...

    @overload
    @classmethod
    def open[F: Features](
        cls,
        uri: str | DataUri,
        *,
        features: type[F],
        storage_options: Mapping[str, object] | None = None,
    ) -> Dataset[F]: ...

    @classmethod
    def open[F: Features](
        cls,
        uri: str | DataUri,
        *,
        features: type[F] | None = None,
        storage_options: Mapping[str, object] | None = None,
    ) -> Dataset[F] | Dataset[dict[str, object]]:
        location = str(uri)
        filesystem, path = url_to_fs(location, **dict(storage_options or {}))
        source = pads.dataset(
            path,
            filesystem=PyFileSystem(FSSpecHandler(filesystem)),
            format="parquet",
        )
        if features is not None:
            features.validate_arrow_schema(source.schema)
            return Dataset[F](source, uri=location, features=features)
        return Dataset[dict[str, object]](source, uri=location)

    @property
    def arrow_schema(self) -> pa.Schema:
        return self._source.schema

    def with_features[F: Features](self, features: type[F]) -> Dataset[F]:
        """Validate and bind features to an already-open dataset."""
        features.validate_arrow_schema(self.arrow_schema)
        return Dataset(
            self._source,
            uri=self.uri,
            features=features,
            _batch_reader=self._batch_reader,
            _row_count=self._row_count,
        )

    def where(self, **equals: object) -> Dataset[T]:
        """Create a lazy view matching all keyword equalities.

        Chained calls combine with AND. ``None`` matches null values.
        Filtering reads no rows and preserves the URI and feature decoding.
        """
        predicate = pads.scalar(value=True)
        for name, value in equals.items():
            field = pads.field(name)
            predicate &= field.is_null() if value is None else field == value
        # Bind the expression against the schema now, without reading any rows.
        source = self._source.filter(predicate)
        if self._batch_reader is None:
            return Dataset(source, uri=self.uri, features=self.features)

        def read(columns: list[str] | None) -> Iterator[pa.RecordBatch]:
            required = (
                None if columns is None else list(dict.fromkeys([*columns, *equals]))
            )
            schema = (
                self.arrow_schema
                if required is None
                else pa.schema([self.arrow_schema.field(name) for name in required])
            )
            scanner = pads.Scanner.from_batches(
                self._batches(required),
                schema=schema,
                columns=columns,
                filter=predicate,
            )
            yield from scanner.to_batches()

        return Dataset(
            self._source, uri=self.uri, features=self.features, _batch_reader=read
        )

    def class_names(self, column: str) -> tuple[str, ...]:
        """Return the full ordered ClassLabel vocabulary without reading rows.

        Filtered views retain all classes, including those absent from the view.
        """
        field = self.arrow_schema.field(column)
        metadata = field.metadata or {}
        if b"class_names" not in metadata and self.features is not None:
            features = cast("type[Features]", self.features)
            if column in features.to_schema().names:
                metadata = features.to_schema().field(column).metadata or {}
        encoded = metadata.get(b"class_names")
        if encoded is None:
            raise ValueError(f"Field {column!r} has no ClassLabel class names")
        names: object = json.loads(encoded)
        if not isinstance(names, list) or not all(
            isinstance(name, str) for name in names
        ):
            raise ValueError(f"Field {column!r} has invalid ClassLabel class names")
        return ClassLabel(*cast("list[str]", names)).names

    def sample(
        self,
        *,
        per_group: int,
        group_by: str | tuple[str, ...],
        seed: int | None = None,
    ) -> Dataset[T]:
        """Select up to ``per_group`` rows per group, uniformly without replacement.

        Scan only grouping columns and keep a reservoir of row positions per group.
        The returned view reads encoded rows in bounded batches when consumed and
        retains source order. Smaller groups keep all rows. A seed is reproducible
        for unchanged data and scan order. Source files must remain unchanged.
        """
        if type(per_group) is not int or per_group <= 0:
            raise ValueError("per_group must be a positive integer row count")
        columns = (group_by,) if isinstance(group_by, str) else group_by
        if not columns or any(not isinstance(name, str) for name in columns):
            raise ValueError("group_by must contain one or more column names")
        if len(set(columns)) != len(columns):
            raise ValueError("group_by columns must be unique")
        for name in columns:
            field = self.arrow_schema.field(name)
            if (
                pa.types.is_nested(field.type)
                or pa.types.is_binary(field.type)
                or pa.types.is_large_binary(field.type)
            ):
                raise TypeError(
                    f"Grouping field {name!r} must be a scalar, non-binary column"
                )

        positions = self._sample_positions(columns, per_group, seed)
        return Dataset(
            self._source,
            uri=self.uri,
            features=self.features,
            _batch_reader=lambda projected: self._selected_batches(
                positions, projected
            ),
            _row_count=len(positions),
        )

    def _sample_positions(
        self, columns: tuple[str, ...], per_group: int, seed: int | None
    ) -> list[int]:
        rng = random.Random(seed)  # noqa: S311 - Reproducible sampling, not security.
        reservoirs: dict[tuple[object, ...], list[int]] = {}
        counts: dict[tuple[object, ...], int] = {}
        position = 0
        for batch in self._batches(list(columns)):
            for row in batch.to_pylist():
                key = tuple(row[name] for name in columns)
                count = counts.get(key, 0) + 1
                counts[key] = count
                reservoir = reservoirs.setdefault(key, [])
                if len(reservoir) < per_group:
                    reservoir.append(position)
                    position += 1
                    continue
                replacement = rng.randrange(count)
                if replacement < per_group:
                    reservoir[replacement] = position
                position += 1
        return sorted(index for group in reservoirs.values() for index in group)

    def _selected_batches(
        self, positions: list[int], columns: list[str] | None
    ) -> Iterator[pa.RecordBatch]:
        if not positions:
            return
        offset = 0
        selected = 0
        for batch in self._batches(columns):
            end = offset + batch.num_rows
            indices: list[int] = []
            while selected < len(positions) and positions[selected] < end:
                indices.append(positions[selected] - offset)
                selected += 1
            if indices:
                yield batch.take(pa.array(indices, type=pa.int64()))
            if selected == len(positions):
                return
            offset = end

    @staticmethod
    def concat[U](*datasets: Dataset[U]) -> Dataset[U]:
        """Lazily concatenate datasets with identical schemas and feature bindings.

        Input order and duplicates are preserved. No rows are read at creation.
        """
        if not datasets:
            raise ValueError("concat requires at least one dataset")
        first = datasets[0]
        for dataset in datasets[1:]:
            if not first.arrow_schema.equals(dataset.arrow_schema, check_metadata=True):
                raise ValueError(
                    "Concatenated datasets must have identical Arrow schemas"
                )
            if dataset.features is not first.features:
                raise ValueError(
                    "Concatenated datasets must have identical feature bindings"
                )

        def read(columns: list[str] | None) -> Iterator[pa.RecordBatch]:
            for dataset in datasets:
                yield from dataset._batches(columns)

        return Dataset(
            first._source, uri=first.uri, features=first.features, _batch_reader=read
        )

    def write(
        self, output_path: str | Path, *, shard_size_mb: int = DEFAULT_SHARD_SIZE_MB
    ) -> Dataset[T]:
        """Stream encoded rows to a new local Parquet directory, without decoding.

        The destination must be empty. Return an independently readable dataset.

        """
        max_bytes = shard_size_bytes(shard_size_mb)
        path = Path(output_path)
        path.mkdir(parents=True, exist_ok=True)
        if any(path.iterdir()):
            raise FileExistsError(f"Output directory {path} must be empty")
        for index, table in enumerate(
            shard_tables(self._batches(), max_bytes=max_bytes)
        ):
            with (path / f"part-{index}.parquet").open("xb") as output:
                pq.write_table(table, output)
        if not any(path.iterdir()):
            pq.write_table(
                pa.Table.from_batches([], schema=self.arrow_schema),
                path / "part-0.parquet",
            )
        return Dataset(
            pads.dataset(str(path), format="parquet"),
            uri=str(path),
            features=self.features,
        )

    def _batches(self, columns: list[str] | None = None) -> Iterator[pa.RecordBatch]:
        if self._batch_reader is not None:
            yield from self._batch_reader(columns)
        else:
            yield from self._source.to_batches(
                columns=columns,
                batch_size=1024,
                batch_readahead=1,
                fragment_readahead=1,
                use_threads=False,
            )

    def __len__(self) -> int:
        if self._row_count is not None:
            return self._row_count
        if self._batch_reader is not None:
            return sum(batch.num_rows for batch in self._batches([]))
        return self._source.count_rows()

    def __iter__(self) -> Iterator[T]:
        """
        Read batches incrementally, yielding decoded features or raw rows.

        Yields:
            T: Either decoded features (if `features` is set) or raw row dictionaries.

        NB: This is useful for exploration and debugging, but this does not take
        full advantage of arrow performance as it decodes each row individually
        rather than using vectorized Arrow operations.
        """
        for batch in self._batches():
            for row in batch.to_pylist():
                if self.features is None:
                    yield cast("T", row)
                else:
                    features = cast("type[Features]", self.features)
                    yield cast("T", features.decode(row))
