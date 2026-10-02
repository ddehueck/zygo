from __future__ import annotations

from collections.abc import Iterator, Mapping
from pathlib import Path
from typing import cast, overload

from fsspec.core import url_to_fs
import pyarrow as pa
import pyarrow.dataset as pads
from pyarrow.fs import FSSpecHandler, PyFileSystem

from zygo.dataset.builder import DEFAULT_SHARD_SIZE, DatasetBuilder
from zygo.dataset.features import Features
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
    ) -> None:
        self._source: pads.Dataset = source
        self.uri = uri
        self._features = features

    @classmethod
    def builder(
        cls,
        output_path: str | Path,
        *,
        schema: type[Features],
        shard_size: int = DEFAULT_SHARD_SIZE,
        overwrite: bool = False,
    ) -> DatasetBuilder:
        """
        Build a local dataset directory.
        e.g.

        >>> with Dataset.builder("mydataset/", schema=Features) as builder:
        ...     builder.add({"image": image, "label": 0})
        """
        return DatasetBuilder(
            output_path, schema=schema, shard_size=shard_size, overwrite=overwrite
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
        return Dataset(self._source, uri=self.uri, features=features)

    def __len__(self) -> int:
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
        for batch in self._source.to_batches():
            for row in batch.to_pylist():
                if self.features is None:
                    yield cast("T", row)
                else:
                    features = cast("type[Features]", self.features)
                    yield cast("T", features.decode(row))
