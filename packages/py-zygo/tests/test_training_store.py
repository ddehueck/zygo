from pathlib import Path

import pytest

from zygo import DataUri, ModelStore
from zygo.transport import LocalTransport


def store_at(path: Path) -> ModelStore:
    return ModelStore(
        root=DataUri(path.as_uri() + "/"),
        ipc_transport=LocalTransport(),
    )


def test_model_store_uses_injected_directory(tmp_path: Path):
    store = store_at(tmp_path)
    uri = store.put("label.txt", b"7")
    assert store.get("label.txt") == b"7"
    assert str(uri) == (tmp_path / "label.txt").as_uri()
    assert store_at(tmp_path).get("label.txt") == b"7"


def test_native_save_and_resume_at_same_path(tmp_path: Path):
    (tmp_path / "weights.bin").write_bytes(b"weights")
    resumed = store_at(tmp_path)
    assert resumed.get("weights.bin") == b"weights"
    resumed.put("state.bin", b"resumed")
    assert (tmp_path / "state.bin").read_bytes() == b"resumed"


def test_failed_caller_preserves_writes(tmp_path: Path):
    store = store_at(tmp_path)
    with pytest.raises(ValueError, match="training failed"):
        store.put("partial.bin", b"partial")
        raise ValueError("training failed")
    assert (tmp_path / "partial.bin").read_bytes() == b"partial"


def test_empty_store_is_allowed(tmp_path: Path):
    store = store_at(tmp_path)
    assert not store.exists("missing.bin")


def test_store_open_and_delete(tmp_path: Path):
    store = store_at(tmp_path)
    with store.open("nested/label.txt", "w") as artifact:
        artifact.write("7")
    store.put("temporary.bin", b"temporary")
    store.delete("temporary.bin")
    assert not store.exists("temporary.bin")
    with store.open("nested/label.txt") as artifact:
        assert artifact.read() == "7"
