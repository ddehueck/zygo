"""Tests for StoreImpl."""

from __future__ import annotations

from typing import TYPE_CHECKING, cast

import fsspec  # type: ignore

from zygo.store import DataUri
from zygo.store._internal.impl import StoreImpl
from zygo.types import JobRunContext, JobRunId, WorkflowRunId

if TYPE_CHECKING:
    from collections.abc import Callable
    from pathlib import Path

    from fsspec.spec import AbstractFileSystem  # type: ignore
    import pytest

    from zygo.cli.v0.types import IpcMessage


class _NoopTransport:
    def emit(self, message: IpcMessage) -> None:
        pass


def _make_context(
    root: str, *, workflow_run_id: str = "wf1", job_run_id: str = "job1"
) -> JobRunContext:
    return JobRunContext(
        workflow_run_id=WorkflowRunId(workflow_run_id),
        job_run_id=JobRunId(job_run_id),
        input=DataUri(f"file://{root}/dummy"),
    )


def _make_store(root: str) -> StoreImpl:
    return StoreImpl(
        context=_make_context(root),
        root=DataUri(f"file://{root}"),
        ipc_transport=_NoopTransport(),
    )


def test_store_passes_kwargs_to_fsspec(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    filesystem = cast("Callable[..., AbstractFileSystem]", fsspec.filesystem)
    calls: list[tuple[str, dict[str, str]]] = []

    def capture_filesystem(protocol: str, **kwargs: str) -> AbstractFileSystem:
        calls.append((protocol, kwargs))
        return filesystem(protocol, **kwargs)

    monkeypatch.setattr(
        "zygo.store._internal.util.fsspec.filesystem", capture_filesystem
    )
    StoreImpl(
        context=_make_context(str(tmp_path)),
        root=DataUri(f"file://{tmp_path}"),
        ipc_transport=_NoopTransport(),
        kwargs={"auto_mkdir": "true"},
    )
    assert calls[-1] == ("file", {"auto_mkdir": "true"})


def test_get_data_uri(tmp_path: Path) -> None:
    """get() reads bytes from a data URI reference."""
    store = _make_store(str(tmp_path))
    input_path = tmp_path / "input"
    input_path.write_bytes(b"hello from uri")
    uri = DataUri(f"file://{input_path}")

    assert store.get(uri) == b"hello from uri"


def test_put_get_round_trip(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Put then get returns bytes under the store root, not the working directory."""
    working_dir = tmp_path / "working"
    working_dir.mkdir()
    monkeypatch.chdir(working_dir)
    store = _make_store(str(tmp_path))
    data = b"hello store"

    uri = store.put("my-key", data)
    assert isinstance(uri, DataUri)
    assert uri.path == str(tmp_path / "wr=wf1" / "jr=job1" / "my-key")
    assert store.get(uri) == data
    assert not (working_dir / "my-key").exists()


def test_get_by_uri(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """get() accepts a DataUri and reads directly from it."""
    working_dir = tmp_path / "working"
    working_dir.mkdir()
    monkeypatch.chdir(working_dir)
    store = _make_store(str(tmp_path))
    data = b"ref-based read"

    uri = store.put("ref_key", data)
    assert uri.path == str(tmp_path / "wr=wf1" / "jr=job1" / "ref_key")
    assert store.get(uri) == data
    assert not (working_dir / "ref_key").exists()


def test_put_with_explicit_uri(tmp_path: Path) -> None:
    store = _make_store(str(tmp_path))
    target = tmp_path / "explicit"

    uri = store.put(f"file://{target}", b"explicit data")

    assert uri.path == str(target)
    assert target.read_bytes() == b"explicit data"


def test_put_exists_delete_exists(tmp_path: Path) -> None:
    """Put creates key, delete removes it: exists and delete behave correctly."""
    store = _make_store(str(tmp_path))

    store.put("x", b"y")
    assert store.exists("x") is True

    store.delete("x")
    assert store.exists("x") is False
