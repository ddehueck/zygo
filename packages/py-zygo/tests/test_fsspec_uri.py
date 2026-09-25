from pathlib import Path

import pytest

from zygo.store import DataUri


def test_local_path_is_normalized_to_absolute_file_uri(tmp_path: Path) -> None:
    path = tmp_path / "example.txt"

    uri = DataUri(f"file://{path}")

    assert uri.protocol == "file"
    assert uri.path == str(path)
    assert uri.key == "example.txt"
    assert uri.is_local()
    assert uri.is_absolute()
    assert str(uri) == f"file://{path}"


def test_relative_path_is_made_absolute() -> None:
    uri = DataUri("file://example.txt")

    assert uri.uri == f"file://{Path('example.txt').resolve()}"
    assert uri.is_absolute()


@pytest.mark.parametrize("path", ["example.txt", "/example.txt"])
def test_uri_requires_explicit_protocol(path: str) -> None:
    with pytest.raises(ValueError, match="explicit protocol"):
        DataUri(path)

    assert DataUri.try_parse(path) is None
    assert not DataUri.is_valid(path)


def test_try_parse_and_is_valid() -> None:
    parsed = DataUri.try_parse("memory:///bucket/example.txt")

    assert parsed is not None
    assert parsed.protocol == "memory"
    assert parsed.key == "example.txt"
    assert DataUri.is_valid("memory:///bucket/example.txt")


def test_zygo_uri_does_not_require_backend_credentials() -> None:
    uri = DataUri("zygo://runs/input.json")

    assert uri.protocol == "zygo"
    assert uri.path == "runs/input.json"
    assert DataUri.is_valid("zygo://runs/input.json")


@pytest.mark.parametrize("uri", ["zygo://", "file://"])
def test_empty_path_is_invalid(uri: str) -> None:
    with pytest.raises(ValueError, match="Empty path"):
        DataUri(uri)


def test_invalid_uri_raises_and_try_parse_returns_none() -> None:
    with pytest.raises(ValueError, match="Invalid fsspec URI"):
        DataUri("unsupported-protocol://bucket/file")

    assert DataUri.try_parse("unsupported-protocol://bucket/file") is None
    assert not DataUri.is_valid("unsupported-protocol://bucket/file")
