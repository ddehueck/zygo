from __future__ import annotations

import hashlib
import io
from pathlib import Path, PurePosixPath
import tempfile
from typing import TYPE_CHECKING, Self, override

import pyarrow as pa
import pyarrow.parquet as pq
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator

if TYPE_CHECKING:
    from collections.abc import Buffer, Mapping
    from typing import BinaryIO


MANIFEST_FILENAME = "_manifest.json"


class FileManifestEntry(BaseModel):
    model_config = ConfigDict(frozen=True, strict=True, extra="forbid")

    filename: str = Field(min_length=1)
    size_bytes: int = Field(ge=0)
    sha256: str = Field(pattern=r"^[0-9a-f]{64}$")

    @field_validator("filename")
    @classmethod
    def validate_filename(cls, filename: str) -> str:
        logical = PurePosixPath(filename)
        if (
            logical.is_absolute()
            or ".." in logical.parts
            or logical.as_posix() != filename
            or filename == "."
            or "\\" in filename
        ):
            raise ValueError(
                "filename must be a normalized dataset-relative POSIX path"
            )
        return filename


class DatasetManifest(BaseModel):
    model_config = ConfigDict(frozen=True, strict=True, extra="forbid")

    schema_version: int = Field(ge=1, le=1)
    files: dict[str, FileManifestEntry]

    @model_validator(mode="after")
    def validate_filenames(self) -> Self:
        for filename, entry in self.files.items():
            if filename != entry.filename:
                raise ValueError(
                    f"Manifest key does not match entry filename: {filename}"
                )
        return self


class _HashingWriter(io.RawIOBase):
    """Forward-only sink hashing every byte, including the Parquet footer."""

    def __init__(self, output: BinaryIO) -> None:
        super().__init__()
        self._output = output
        self.digest = hashlib.sha256()
        self.size_bytes = 0

    @override
    def writable(self) -> bool:
        return True

    @override
    def tell(self) -> int:
        return self.size_bytes

    @override
    def write(self, data: Buffer, /) -> int:
        view = memoryview(data)
        size = self._output.write(view)
        if size != view.nbytes:
            raise OSError("Incomplete Parquet output write")
        self.digest.update(view)
        self.size_bytes += size
        return size

    @override
    def flush(self) -> None:
        self._output.flush()


def write_parquet(
    table: pa.Table, path: Path, *, filename: str, overwrite: bool = False
) -> FileManifestEntry:
    """Hash the complete serialized file without rereading it."""
    with (
        path.open("wb" if overwrite else "xb") as output,
        _HashingWriter(output) as writer,
    ):
        with pa.PythonFile(writer, mode="w") as sink:
            pq.write_table(table, sink)
            sink.flush()
        return FileManifestEntry(
            filename=filename,
            size_bytes=writer.size_bytes,
            sha256=writer.digest.hexdigest(),
        )


def publish_manifest(root: Path, files: Mapping[str, FileManifestEntry]) -> None:
    """Atomically publish metadata only after all shard writes have succeeded."""
    manifest = DatasetManifest(schema_version=1, files=dict(files))
    temporary: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=root, prefix="._manifest-", delete=False
        ) as output:
            temporary = Path(output.name)
            output.write(manifest.model_dump_json(indent=2))
            output.write("\n")
        temporary.replace(root / MANIFEST_FILENAME)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
