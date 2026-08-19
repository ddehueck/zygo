import math
from typing import override

from zygo.codecs.base import (
    Codec,
    CodecDecodeError,
    CodecEncodeError,
    FileFormat,
)


class Bytes(Codec[bytes]):
    """Raw binary bytes."""

    @property
    @override
    def value_type(self) -> type[bytes]:
        return bytes

    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(
            content_type="application/octet-stream",
            extension=".bin",
        )

    @override
    def encode(self, value: bytes, /) -> bytes:
        if type(value) is not bytes:
            raise CodecEncodeError(f"Bytes expected bytes, got {type(value).__name__}")
        return value

    @override
    def decode(self, payload: bytes, /) -> bytes:
        return payload


class String(Codec[str]):
    """A string encoded as UTF-8 text."""

    @property
    @override
    def value_type(self) -> type[str]:
        return str

    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(
            content_type="text/plain; charset=utf-8",
            extension=".txt",
        )

    @override
    def encode(self, value: str, /) -> bytes:
        if type(value) is not str:
            raise CodecEncodeError(f"String expected str, got {type(value).__name__}")
        return value.encode("utf-8")

    @override
    def decode(self, payload: bytes, /) -> str:
        try:
            return payload.decode("utf-8")
        except UnicodeDecodeError as error:
            raise CodecDecodeError("String payload is not valid UTF-8") from error


class Integer(Codec[int]):
    """An integer encoded as canonical base-10 ASCII text."""

    @property
    @override
    def value_type(self) -> type[int]:
        return int

    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(
            content_type="text/plain; charset=us-ascii",
            extension=".txt",
        )

    @override
    def encode(self, value: int, /) -> bytes:
        if type(value) is not int:
            raise CodecEncodeError(f"Integer expected int, got {type(value).__name__}")
        return str(value).encode("ascii")

    @override
    def decode(self, payload: bytes, /) -> int:
        try:
            encoded = payload.decode("ascii")
        except UnicodeDecodeError as error:
            raise CodecDecodeError("Integer payload is not ASCII") from error

        return int(encoded)


class Float(Codec[float]):
    """A finite float encoded as round-trippable ASCII text."""

    @property
    @override
    def value_type(self) -> type[float]:
        return float

    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(
            content_type="text/plain; charset=us-ascii",
            extension=".txt",
        )

    @override
    def encode(self, value: float, /) -> bytes:
        if type(value) is not float:
            raise CodecEncodeError(f"Float expected float, got {type(value).__name__}")
        if not math.isfinite(value):
            raise CodecEncodeError("Float does not support NaN or infinity")
        return repr(value).encode("ascii")

    @override
    def decode(self, payload: bytes, /) -> float:
        try:
            encoded = payload.decode("ascii")
        except UnicodeDecodeError as error:
            raise CodecDecodeError("Float payload is not ASCII") from error

        if not encoded or encoded.strip() != encoded:
            raise CodecDecodeError("Float payload is not canonical decimal text")

        try:
            value = float(encoded)
        except ValueError as error:
            raise CodecDecodeError("Float payload is not valid decimal text") from error

        if not math.isfinite(value):
            raise CodecDecodeError("Float payload contains NaN or infinity")
        return value


class Boolean(Codec[bool]):
    """A boolean encoded as lowercase ASCII text."""

    @property
    @override
    def value_type(self) -> type[bool]:
        return bool

    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(
            content_type="text/plain; charset=us-ascii",
            extension=".txt",
        )

    @override
    def encode(self, value: bool, /) -> bytes:
        if type(value) is not bool:
            raise CodecEncodeError(f"Boolean expected bool, got {type(value).__name__}")
        return b"true" if value else b"false"

    @override
    def decode(self, payload: bytes, /) -> bool:
        if payload == b"true":
            return True
        if payload == b"false":
            return False
        raise CodecDecodeError("Boolean payload must be 'true' or 'false'")
