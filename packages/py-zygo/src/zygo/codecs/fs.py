import re
from typing import override

from zygo.store import DataUri
from zygo.codecs.primitives import String
from zygo.codecs.base import (
    Codec,
    CodecDecodeError,
    CodecEncodeError,
    FileExtension,
    FileFormat,
)

class Folder(String):
    """
    Represents a folder and stores its path as a string.
    """
    @override
    def encode(self, value: str | DataUri) -> bytes:
        if isinstance(value, DataUri):
            value = value.uri
        if not value.endswith('/'):
            raise CodecEncodeError(f"Folder path must end with '/', got {value}")
        return super().encode(value)

    @override
    def decode(self, value: bytes) -> DataUri:
        result = super().decode(value)
        return DataUri(result)

class File(String):
    """
    Represents a file and stores its path as a string.
    """
    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(extension=FileExtension(".txt"))

    @override
    def encode(self, value: str | DataUri) -> bytes:
        if isinstance(value, DataUri):
            value = value.uri
        if value.endswith('/'):
            raise CodecEncodeError(f"File path must not end with '/', got {value}")
        return super().encode(value)

    @override
    def decode(self, value: bytes) -> DataUri:
        result = super().decode(value)
        return DataUri(result)
