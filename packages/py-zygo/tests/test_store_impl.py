"""Tests for WorkflowStore."""

from __future__ import annotations

from typing import TYPE_CHECKING, cast, override

import fsspec  # type: ignore
import pytest

from zygo.cli.v0.types import DataReferenceInserted, StoreConfig, WorkflowStoreConfig
from zygo.store import DataUri
from zygo.store._internal.base import BaseStore
from zygo.workflow.store import WorkflowStore
from zygo.workflow.types import JobRunContext, JobRunId, WorkflowRunId

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence
    from pathlib import Path

    from fsspec.spec import AbstractFileSystem  # type: ignore

    from zygo.cli.v0.types import IpcMessage


class _NoopTransport:
    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None:
        pass


class _RecordingTransport(_NoopTransport):
    def __init__(self) -> None:
        super().__init__()
        self.messages: list[IpcMessage] = []

    @override
    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None:
        if isinstance(messages, (list, tuple)):
            self.messages.extend(messages)
        else:
            self.messages.append(cast("IpcMessage", messages))


def test_base_store_put_emits_data_reference(tmp_path: Path) -> None:
    transport = _RecordingTransport()
    store = BaseStore(root=DataUri(f"file://{tmp_path}/"), ipc_transport=transport)

    uri = store.put("output", b"data")

    assert store.get(uri) == b"data"
    assert transport.messages == [
        DataReferenceInserted(type="data_reference_inserted", data_reference=str(uri))
    ]


def test_base_store_failed_put_does_not_emit(tmp_path: Path) -> None:
    transport = _RecordingTransport()
    store = BaseStore(root=DataUri(f"file://{tmp_path}/"), ipc_transport=transport)
    (tmp_path / "output").mkdir()

    with pytest.raises(IsADirectoryError):
        store.put("output", b"data")

    assert transport.messages == []


def test_scoped_store_put_emits_once_per_write(tmp_path: Path) -> None:
    transport = _RecordingTransport()
    store = WorkflowStore(
        context=_make_context(str(tmp_path)),
        config=_make_config(str(tmp_path)),
        ipc_transport=transport,
    )
    uris = [
        store.scope(scope).put("output", b"data")
        for scope in ("job", "workflow", "cache")
    ]

    assert transport.messages == [
        DataReferenceInserted(type="data_reference_inserted", data_reference=str(uri))
        for uri in uris
    ]


def _make_context(
    root: str, *, workflow_run_id: str = "wf1", job_run_id: str = "job1"
) -> JobRunContext:
    return JobRunContext(
        workflow_run_id=WorkflowRunId(workflow_run_id),
        job_run_id=JobRunId(job_run_id),
        input=DataUri(f"file://{root}/dummy"),
    )


def _make_config(root: str) -> WorkflowStoreConfig:
    return WorkflowStoreConfig(
        job=StoreConfig(root_uri=f"file://{root}/jobs/"),
        workflow=StoreConfig(root_uri=f"file://{root}/workflows/"),
        cache=StoreConfig(root_uri=f"file://{root}/cache/"),
    )


def _make_store(root: str) -> WorkflowStore:
    return WorkflowStore(
        context=_make_context(root),
        config=_make_config(root),
        ipc_transport=_NoopTransport(),
    )


def test_store_passes_kwargs_to_fsspec(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    filesystem = cast("Callable[..., AbstractFileSystem]", fsspec.filesystem)
    calls: list[tuple[str, dict[str, object]]] = []

    def capture_filesystem(protocol: str, **kwargs: object) -> AbstractFileSystem:
        calls.append((protocol, kwargs))
        return filesystem(protocol, **kwargs)

    monkeypatch.setattr(
        "zygo.store._internal.util.fsspec.filesystem", capture_filesystem
    )
    config = _make_config(str(tmp_path))
    config.job.kwargs = {"auto_mkdir": True}
    config.workflow.kwargs = {"auto_mkdir": False}
    config.cache.kwargs = {"auto_mkdir": True, "skip_instance_cache": True}
    store = WorkflowStore(
        context=_make_context(str(tmp_path)),
        config=config,
        ipc_transport=_NoopTransport(),
    )
    assert calls[-1] == ("file", {"auto_mkdir": True})
    store.scope("workflow")
    assert calls[-1] == ("file", {"auto_mkdir": False})
    store.scope("cache")
    assert calls[-1] == ("file", {"auto_mkdir": True, "skip_instance_cache": True})


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

    uri = store.scope("job").put("my-key", data)
    assert isinstance(uri, DataUri)
    assert uri.path == str(tmp_path / "jobs" / "wr=wf1" / "jr=job1" / "my-key")
    assert store.get(uri) == data
    assert not (working_dir / "my-key").exists()


def test_get_by_uri(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """get() accepts a DataUri and reads directly from it."""
    working_dir = tmp_path / "working"
    working_dir.mkdir()
    monkeypatch.chdir(working_dir)
    store = _make_store(str(tmp_path))
    data = b"ref-based read"

    uri = store.scope("job").put("ref_key", data)
    assert uri.path == str(tmp_path / "jobs" / "wr=wf1" / "jr=job1" / "ref_key")
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

    store.scope("job").put("x", b"y")
    assert store.scope("job").exists("x") is True

    store.scope("job").delete("x")
    assert store.scope("job").exists("x") is False


def test_cache_scope_is_not_partitioned_by_run(tmp_path: Path) -> None:
    config = _make_config(str(tmp_path))
    first = WorkflowStore(
        context=_make_context(str(tmp_path)),
        config=config,
        ipc_transport=_NoopTransport(),
    )
    second = WorkflowStore(
        context=_make_context(str(tmp_path), workflow_run_id="wf2", job_run_id="job2"),
        config=config,
        ipc_transport=_NoopTransport(),
    )
    uri = first.scope("cache").put("value", b"cached data")
    assert uri.path == str(tmp_path / "cache" / "value")
    assert second.scope("cache").get("value") == b"cached data"
    assert not second.scope("job").exists("value")
