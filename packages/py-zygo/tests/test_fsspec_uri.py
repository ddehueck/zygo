from pathlib import Path

import pytest

from zygo.store import DataUri


def test_local_path_is_normalized_to_absolute_file_uri(tmp_path: Path) -> None:
    path = tmp_path / "example.txt"

    uri = DataUri(str(path))

    assert uri.protocol == "file"
    assert uri.path == str(path)
    assert uri.key == "example.txt"
    assert uri.is_local()
    assert uri.is_absolute()
    assert str(uri) == str(path)


def test_relative_path_is_made_absolute() -> None:
    uri = DataUri("example.txt")

    assert uri.uri == f"file://{Path('example.txt').resolve()}"
    assert uri.is_absolute()


def test_try_parse_and_is_valid() -> None:
    parsed = DataUri.try_parse("memory:///bucket/example.txt")

    assert parsed is not None
    assert parsed.protocol == "memory"
    assert parsed.key == "example.txt"
    assert DataUri.is_valid("memory:///bucket/example.txt")


def test_invalid_uri_raises_and_try_parse_returns_none() -> None:
    with pytest.raises(ValueError, match="Invalid fsspec URI"):
        DataUri("unsupported-protocol://bucket/file")

    assert DataUri.try_parse("unsupported-protocol://bucket/file") is None
    assert not DataUri.is_valid("unsupported-protocol://bucket/file")
