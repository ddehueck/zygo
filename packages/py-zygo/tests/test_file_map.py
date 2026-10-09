from typing import cast

import pytest

from zygo import Channel
from zygo.store import DataUri
from zygo.workflow.codecs import CodecDecodeError, CodecEncodeError, FileMap


def test_file_map_round_trip():
    codec = FileMap
    value = {"z": "memory:///output.txt", "a": DataUri("file:///input.txt")}

    payload = codec.encode(value)

    assert payload == b'{"a":"file:///input.txt","z":"memory:///output.txt"}'
    assert codec.decode(payload) == {
        "a": DataUri("file:///input.txt"),
        "z": DataUri("memory:///output.txt"),
    }
    assert isinstance(value["z"], str)
    assert codec.format.extension == "json"
    assert Channel(id="files", codec=codec).value_type == dict[str, DataUri]


def test_file_map_rejects_empty_map():
    codec = FileMap
    with pytest.raises(CodecEncodeError, match="at least one file"):
        codec.encode({})
    with pytest.raises(CodecDecodeError, match="at least one file"):
        codec.decode(b"{}")


def test_file_map_normalizes_uri_strings():
    codec = FileMap
    uri = "file://relative.txt"
    assert codec.decode(codec.encode({"file": uri})) == {"file": DataUri(uri)}


@pytest.mark.parametrize(
    "value",
    [[], {1: "file:///a"}, {"a": 1}, {"a": "relative.txt"}, {"a": "file:///folder/"}],
)
def test_file_map_rejects_invalid_values(value: object):
    with pytest.raises(CodecEncodeError):
        FileMap.encode(cast("dict[str, str | DataUri]", value))


@pytest.mark.parametrize(
    "payload",
    [
        b"[]",
        b'{"a":1}',
        b'{"a":"relative.txt"}',
        b'{"a":"file:///folder/"}',
        b"{",
        b"\xff",
    ],
)
def test_file_map_rejects_invalid_payloads(payload: bytes):
    with pytest.raises(CodecDecodeError):
        FileMap.decode(payload)
