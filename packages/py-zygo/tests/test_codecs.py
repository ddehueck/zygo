import pytest

from zygo.codecs import Bytes, FileExtension, Json, String


@pytest.mark.parametrize(
    ("value", "expected"),
    [(".json", "json"), ("json", "json"), ("..json", "json"), ("...", "")],
)
def test_file_extension_removes_leading_dots(value: str, expected: str) -> None:
    extension = FileExtension(value)

    assert extension == expected
    assert isinstance(extension, str)


def test_codec_formats_use_only_normalized_file_extensions() -> None:
    formats = [Bytes().format, String().format, Json(int).format]

    assert [format.extension for format in formats] == ["bin", "txt", "json"]
    assert all(not hasattr(format, "content_type") for format in formats)
