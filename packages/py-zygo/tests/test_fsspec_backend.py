# fsspec's untyped open signature obscures the binary mode in strict checking.
# pyright: reportUnknownMemberType=false

from collections.abc import Sequence
from io import BytesIO
import json
from typing import IO, cast
from urllib.request import Request

import pytest

from zygo._internal.cloud import fsspec_backend
from zygo.cli.v0.types import IpcMessage
from zygo.store import DataUri
from zygo.store._internal.impl import StoreImpl
from zygo.types import JobRunContext, JobRunId, WorkflowRunId


class _NoopTransport:
    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None:
        pass


@pytest.fixture
def transfers(
    monkeypatch: pytest.MonkeyPatch,
) -> list[tuple[str, str, bytes | None, str | None]]:
    calls: list[tuple[str, str, bytes | None, str | None]] = []

    def urlopen(request: Request, timeout: int) -> BytesIO:
        url = request.full_url
        calls.append((
            request.get_method(),
            url,
            cast("bytes | None", request.data),
            request.get_header("Authorization"),
        ))
        if url.endswith("/presign"):
            assert timeout == 30
            return BytesIO(
                json.dumps({"url": "https://storage.example/object"}).encode()
            )

        assert timeout == 120
        if request.get_method() == "GET":
            return BytesIO(b"object contents")
        return BytesIO()

    monkeypatch.setattr(fsspec_backend, "urlopen", urlopen)
    return calls


def test_read_streams_from_presigned_url(
    transfers: list[tuple[str, str, bytes | None, str | None]],
) -> None:
    fs = fsspec_backend.ZygoFileSystem(
        api_host="https://api.zygo.cloud",
        api_bearer_auth="test-secret",
        skip_instance_cache=True,
    )

    with cast("IO[bytes]", fs.open("zygo://folder/object name", "rb")) as file:
        assert file.read(6) == b"object"
        assert file.read() == b" contents"

    assert transfers == [
        (
            "POST",
            "https://api.zygo.cloud/v1/store/folder%2Fobject%20name/presign",
            b'{"method": "GET"}',
            "Bearer test-secret",
        ),
        ("GET", "https://storage.example/object", None, None),
    ]


def test_store_put_with_zygo_backend(
    transfers: list[tuple[str, str, bytes | None, str | None]],
) -> None:
    store = StoreImpl(
        context=JobRunContext(
            workflow_run_id=WorkflowRunId("wf1"),
            job_run_id=JobRunId("job1"),
            input=DataUri("zygo://runs/input"),
        ),
        root=DataUri("zygo://runs"),
        ipc_transport=_NoopTransport(),
        kwargs={
            "api_host": "https://api.zygo.cloud",
            "api_bearer_auth": "test-secret",
            "skip_instance_cache": True,
        },
    )

    uri = store.put("output.txt", b"output")

    assert str(uri) == "zygo://runs/wr=wf1/jr=job1/output.txt"
    assert transfers == [
        (
            "POST",
            "https://api.zygo.cloud/v1/store/runs%2Fwr%3Dwf1%2Fjr%3Djob1%2Foutput.txt/presign",
            b'{"method": "PUT"}',
            "Bearer test-secret",
        ),
        ("PUT", "https://storage.example/object", b"output", None),
    ]


def test_write_puts_once_on_close(
    transfers: list[tuple[str, str, bytes | None, str | None]],
) -> None:
    fs = fsspec_backend.ZygoFileSystem(
        api_host="https://api.zygo.cloud",
        api_bearer_auth="test-secret",
        skip_instance_cache=True,
    )

    with cast("IO[bytes]", fs.open("zygo://file", "wb")) as file:
        file.write(b"first")
        file.write(b" second")
        assert transfers == []

    assert transfers == [
        (
            "POST",
            "https://api.zygo.cloud/v1/store/file/presign",
            b'{"method": "PUT"}',
            "Bearer test-secret",
        ),
        ("PUT", "https://storage.example/object", b"first second", None),
    ]
