"""Codecs for values stored as human-readable files."""

from __future__ import annotations

import csv
from io import StringIO
from pathlib import Path
import tempfile
from typing import TYPE_CHECKING, ClassVar

from zygo.workflow.codecs.base import (
    CodecDecodeError,
    CodecEncodeError,
    FileExtension,
    FileFormat,
)

if TYPE_CHECKING:
    from collections.abc import Callable


class _TemporaryFileValue:
    """A decoded file kept on disk for the lifetime of its value."""

    _extension: ClassVar[str]

    def __init__(self, payload: bytes) -> None:
        self._directory = tempfile.TemporaryDirectory()
        self.path = Path(self._directory.name) / f"value.{self._extension}"
        self.path.write_bytes(payload)

    def close(self) -> None:
        self._directory.cleanup()


class Csv(_TemporaryFileValue):
    """A decoded CSV file value and codec for CSV-compatible Python objects.

    Use the class itself as a channel codec. Values decoded from a channel expose
    a temporary local ``path``, so libraries such as pandas and Polars can read
    them without Zygo depending on either library.
    """

    _extension: ClassVar[str] = "csv"
    value_type: ClassVar[type[Csv]]
    format: ClassVar[FileFormat] = FileFormat(extension=FileExtension("csv"))

    @classmethod
    def encode(cls, value: object, /) -> bytes:
        if isinstance(value, Csv):
            return value.path.read_bytes()
        if isinstance(value, bytes):
            return value
        if isinstance(value, str):
            return value.encode("utf-8")

        for method_name in ("to_csv", "write_csv"):
            method = getattr(value, method_name, None)
            if callable(method):
                return cls._encode_writer_result(
                    method, method_name
                )

        to_list = getattr(value, "tolist", None)
        if callable(to_list):
            return cls._encode_rows(to_list())

        if isinstance(value, (list, tuple)):
            return cls._encode_rows(value)

        raise CodecEncodeError(
            f"Csv expected text or a CSV-compatible object, got {type(value).__name__}"
        )

    @classmethod
    def _encode_writer_result(
        cls, method: Callable[..., object], method_name: str
    ) -> bytes:
        try:
            if method_name == "to_csv":
                try:
                    result = method(index=False)
                except TypeError:
                    result = method()
            else:
                result = method()
        except Exception as error:
            raise CodecEncodeError(str(error)) from error

        if isinstance(result, bytes):
            return result
        if isinstance(result, str):
            return result.encode("utf-8")
        raise CodecEncodeError(
            f"{method_name}() must return str or bytes, got {type(result).__name__}"
        )

    @classmethod
    def _encode_rows(cls, rows: object) -> bytes:
        if not isinstance(rows, (list, tuple)):
            raise CodecEncodeError("Csv tabular values must convert to rows")

        if rows and not isinstance(rows[0], (list, tuple)):
            rows = [rows]

        output = StringIO(newline="")
        try:
            writer = csv.writer(output, lineterminator="\n")
            writer.writerows(rows)
        except (csv.Error, TypeError) as error:
            raise CodecEncodeError(str(error)) from error
        return output.getvalue().encode("utf-8")

    @classmethod
    def decode(cls, payload: bytes, /) -> Csv:
        try:
            payload.decode("utf-8")
        except UnicodeDecodeError as error:
            raise CodecDecodeError("Csv payload is not valid UTF-8") from error
        return cls(payload)

    @classmethod
    def accepts_output_type(cls, value_type: object) -> bool:
        if value_type is str:
            return True
        if not isinstance(value_type, type):
            return False
        return any(
            callable(getattr(value_type, method_name, None))
            for method_name in ("to_csv", "write_csv", "tolist")
        )


class Txt(_TemporaryFileValue):
    """A decoded text file value and codec for text-convertible Python objects."""

    _extension: ClassVar[str] = "txt"
    value_type: ClassVar[type[Txt]]
    format: ClassVar[FileFormat] = FileFormat(extension=FileExtension("txt"))

    @classmethod
    def encode(cls, value: object, /) -> bytes:
        if isinstance(value, Txt):
            return value.path.read_bytes()
        if isinstance(value, bytes):
            try:
                value.decode("utf-8")
            except UnicodeDecodeError as error:
                raise CodecEncodeError("Txt value is not valid UTF-8") from error
            return value
        try:
            return str(value).encode("utf-8")
        except Exception as error:
            raise CodecEncodeError(str(error)) from error

    @classmethod
    def decode(cls, payload: bytes, /) -> Txt:
        try:
            payload.decode("utf-8")
        except UnicodeDecodeError as error:
            raise CodecDecodeError("Txt payload is not valid UTF-8") from error
        return cls(payload)

    @classmethod
    def accepts_output_type(cls, _value_type: object) -> bool:
        return _value_type is not type(None)


# These class-valued codecs are also the runtime types of decoded file values.
Csv.value_type = Csv
Txt.value_type = Txt


__all__ = ["Csv", "Txt"]
