from dataclasses import dataclass
from typing import Protocol, Self


class FileExtension(str):
    """A file extension normalized without leading dots."""

    def __new__(cls, value: str) -> Self:
        return str.__new__(cls, value.lstrip("."))

    def with_leading_dot(self) -> str:
        return f".{self}"


@dataclass(frozen=True)
class FileFormat:
    """What the resulting file should look like for a codec."""

    extension: FileExtension


class CodecError(ValueError):
    """Base error raised while encoding or decoding a channel value."""


class CodecEncodeError(CodecError):
    """Raised when a Python value cannot be encoded by a codec."""


class CodecDecodeError(CodecError):
    """Raised when a payload cannot be decoded by a codec."""


class Codec[T](Protocol):
    """Transforms typed Python values to and from stored bytes."""

    @property
    def value_type(self) -> type[T]: ...

    @property
    def format(self) -> FileFormat: ...

    def encode(self, value: T, /) -> bytes: ...

    def decode(self, payload: bytes, /) -> T: ...
