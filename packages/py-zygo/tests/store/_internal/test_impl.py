"""Tests for StoreImpl."""

from __future__ import annotations

from typing import TYPE_CHECKING

from zygo.store import Reference, StoreConfig
from zygo.store._internal.impl import StoreImpl
from zygo.types import JobRunContext

if TYPE_CHECKING:
    from pathlib import Path


def _make_context(
    root: str, *, workflow_run_id: str = "wf1", job_run_id: str = "job1"
) -> JobRunContext:
    return JobRunContext(
        store_config=StoreConfig(root_uri=f"file://{root}"),
        workflow_run_id=workflow_run_id,
        job_run_id=job_run_id,
        data_ref=Reference(key="dummy", scope="job", uri=f"{root}/dummy", etag="dummy"),
    )


def test_put_get_round_trip(tmp_path: Path) -> None:
    """Put then get returns the same bytes: core read/write path and URI building."""
    ctx = _make_context(str(tmp_path))
    store = StoreImpl(context=ctx)
    data = b"hello store"

    ref = store.put("my_key", data)
    assert ref.key == "my_key"
    assert store.get("my_key") == data


def test_get_by_reference(tmp_path: Path) -> None:
    """get() accepts a Reference and reads directly from its URI."""
    ctx = _make_context(str(tmp_path))
    store = StoreImpl(context=ctx)
    data = b"ref-based read"

    ref = store.put("ref_key", data)
    assert store.get(ref) == data


def test_put_exists_delete_exists(tmp_path: Path) -> None:
    """Put creates key, delete removes it: exists and delete behave correctly."""
    ctx = _make_context(str(tmp_path))
    store = StoreImpl(context=ctx)

    store.put("x", b"y")
    assert store.exists("x") is True

    store.delete("x")
    assert store.exists("x") is False
