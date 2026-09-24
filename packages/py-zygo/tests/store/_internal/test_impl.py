"""Tests for StoreImpl."""

from __future__ import annotations

from typing import TYPE_CHECKING


from zygo._internal.fsspec import FsspecUri
from zygo.store import Reference, StoreOptions
from zygo.store._internal.impl import StoreImpl
from zygo.types import JobRunContext

if TYPE_CHECKING:
    from pathlib import Path

    from zygo.cli.v0.types import IpcMessage


class _NoopTransport:
    def emit(self, message: IpcMessage) -> None:
        pass


def _make_context(
    root: str, *, workflow_run_id: str = "wf1", job_run_id: str = "job1"
) -> JobRunContext:
    return JobRunContext(
        workflow_run_id=workflow_run_id,
        job_run_id=job_run_id,
        data_ref=Reference(key="dummy", uri=FsspecUri(f"file://{root}/dummy")),
    )


def _make_store(root: str) -> StoreImpl:
    return StoreImpl(
        context=_make_context(root),
        options=StoreOptions(root_uri=FsspecUri(f"file://{root}")),
        ipc_transport=_NoopTransport(),
    )


def test_get_data_uri(tmp_path: Path) -> None:
    """get() reads bytes from a data URI reference."""
    store = _make_store(str(tmp_path))
    ref = Reference(key="hello", uri=FsspecUri("data:,Hello%2C%20World%21"))

    assert store.get(ref) == b"Hello, World!"


def test_put_get_round_trip(tmp_path: Path) -> None:
    """Put then get returns the same bytes: core read/write path and URI building."""
    store = _make_store(str(tmp_path))
    data = b"hello store"

    ref = store.put("my_key", data)
    assert ref.key == "my_key"
    assert store.get("my_key") == data


def test_get_by_reference(tmp_path: Path) -> None:
    """get() accepts a Reference and reads directly from its URI."""
    store = _make_store(str(tmp_path))
    data = b"ref-based read"

    ref = store.put("ref_key", data)
    assert store.get(ref) == data


def test_put_exists_delete_exists(tmp_path: Path) -> None:
    """Put creates key, delete removes it: exists and delete behave correctly."""
    store = _make_store(str(tmp_path))

    store.put("x", b"y")
    assert store.exists("x") is True

    store.delete("x")
    assert store.exists("x") is False
