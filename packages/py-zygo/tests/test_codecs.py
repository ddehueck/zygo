import pytest

from zygo import Channel
from zygo.codecs import Boolean, Bytes, FileExtension, Float, Integer, Json, String


@pytest.mark.parametrize(
    ("value", "expected"),
    [(".json", "json"), ("json", "json"), ("..json", "json"), ("...", "")],
)
def test_file_extension_removes_leading_dots(value: str, expected: str) -> None:
    extension = FileExtension(value)

    assert extension == expected
    assert isinstance(extension, str)


def test_codec_formats_use_only_normalized_file_extensions() -> None:
    formats = [Bytes.format, String.format, Json(int).format]

    assert [format.extension for format in formats] == ["bin", "txt", "json"]
    assert all(not hasattr(format, "content_type") for format in formats)


@pytest.mark.parametrize(
    ("codec", "value", "value_type"),
    [
        (Boolean, True, bool),
        (Bytes, b"hello", bytes),
        (Float, 1.5, float),
        (Integer, 42, int),
        (String, "hello", str),
    ],
)
def test_primitive_codecs_are_ready_to_use(codec, value, value_type) -> None:
    channel = Channel(id="value", codec=codec)
    other_channel = Channel(id="other", codec=codec)

    assert channel.codec is other_channel.codec is codec
    assert channel.value_type is value_type
    assert codec.decode(codec.encode(value)) == value
