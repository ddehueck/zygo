from collections.abc import Mapping
from typing import override

from zygo.store import DataUri
from zygo.workflow.codecs.base import (
    Codec,
    CodecDecodeError,
    CodecEncodeError,
    FileExtension,
    FileFormat,
)
from zygo.workflow.codecs.json import Json
from zygo.workflow.codecs.primitives import String


class Folder(Codec[DataUri]):
    """
    Represents a folder and stores its path as a string.
    """

    @property
    @override
    def value_type(self) -> type[DataUri]:
        return DataUri

    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(extension=FileExtension(".txt"))

    @override
    def encode(self, value: str | DataUri) -> bytes:
        if isinstance(value, DataUri):
            value = value.uri
        if not value.endswith("/"):
            raise CodecEncodeError(f"Folder path must end with '/', got {value}")
        return String().encode(value)

    @override
    def decode(self, value: bytes) -> DataUri:
        result = String().decode(value)
        return DataUri(result)


class File(Codec[DataUri]):
    """
    Represents a file and stores its path as a string.
    """

    @property
    @override
    def value_type(self) -> type[DataUri]:
        return DataUri

    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(extension=FileExtension(".txt"))

    @override
    def encode(self, value: str | DataUri) -> bytes:
        if isinstance(value, DataUri):
            value = value.uri
        if value.endswith("/"):
            raise CodecEncodeError(f"File path must not end with '/', got {value}")
        return String().encode(value)

    @override
    def decode(self, value: bytes) -> DataUri:
        result = String().decode(value)
        return DataUri(result)


class FileMap(Codec[dict[str, DataUri]]):
    """A map of file URIs stored as JSON."""

    def __init__(self) -> None:
        super().__init__()
        self._json = Json(dict[str, str])

    @property
    @override
    def value_type(self) -> type[dict[str, DataUri]]:
        return dict[str, DataUri]

    @property
    @override
    def format(self) -> FileFormat:
        return self._json.format

    @staticmethod
    def _parse_file_uri(value: object) -> DataUri:
        if not isinstance(value, (str, DataUri)):
            raise ValueError("FileMap values must be strings or DataUri objects")
        uri = DataUri(value) if isinstance(value, str) else value
        if uri.uri.endswith("/"):
            raise ValueError(f"File path must not end with '/', got {uri.uri}")
        return uri

    @override
    def encode(self, value: Mapping[str, str | DataUri], /) -> bytes:
        if not isinstance(value, Mapping):  # pyright: ignore[reportUnnecessaryIsInstance]
            raise CodecEncodeError("FileMap expected a string-keyed mapping")
        if not value:
            raise CodecEncodeError("FileMap must contain at least one file")
        try:
            normalized = {
                key: self._parse_file_uri(uri).uri for key, uri in value.items()
            }
        except ValueError as error:
            raise CodecEncodeError(str(error)) from error
        return self._json.encode(normalized)

    @override
    def decode(self, payload: bytes, /) -> dict[str, DataUri]:
        values = self._json.decode(payload)
        if not values:
            raise CodecDecodeError("FileMap must contain at least one file")
        try:
            return {key: self._parse_file_uri(uri) for key, uri in values.items()}
        except ValueError as error:
            raise CodecDecodeError(str(error)) from error
