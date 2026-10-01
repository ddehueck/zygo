from pathlib import Path

import pytest

from zygo import DataUri, TrainingContext


def context_at(path: Path) -> TrainingContext:
    return TrainingContext.local(DataUri(path.as_uri() + "/"))


def test_training_store_uses_injected_directory(tmp_path):
    with context_at(tmp_path).store() as store:
        uri = store.put("label.txt", b"7")
        assert store.path == tmp_path
        assert store.get("label.txt") == b"7"
    assert str(uri) == (tmp_path / "label.txt").as_uri()
    assert context_at(tmp_path).store().get("label.txt") == b"7"


def test_native_save_and_resume_at_same_path(tmp_path):
    with context_at(tmp_path).store() as original:
        (original.path / "weights.bin").write_bytes(b"weights")
    with context_at(tmp_path).store() as resumed:
        assert resumed.get("weights.bin") == b"weights"
        resumed.put("state.bin", b"resumed")
    assert (tmp_path / "state.bin").read_bytes() == b"resumed"


def test_failed_block_preserves_writes(tmp_path):
    with (
        pytest.raises(ValueError, match="training failed"),
        context_at(tmp_path).store() as store,
    ):
        store.put("partial.bin", b"partial")
        raise ValueError("training failed")
    assert (tmp_path / "partial.bin").read_bytes() == b"partial"
    with pytest.raises(RuntimeError, match="inside its context"):
        _ = store.path


def test_empty_store_is_allowed(tmp_path):
    with context_at(tmp_path).store():
        pass


def test_store_open_and_delete(tmp_path):
    with context_at(tmp_path).store() as store:
        with store.open("nested/label.txt", "w") as artifact:
            artifact.write("7")
        store.put("temporary.bin", b"temporary")
        store.delete("temporary.bin")
        assert not store.exists("temporary.bin")
    with context_at(tmp_path).store().open("nested/label.txt") as artifact:
        assert artifact.read() == "7"
