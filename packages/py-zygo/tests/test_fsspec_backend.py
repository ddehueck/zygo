from io import BytesIO
from unittest.mock import MagicMock

import fsspec
import pytest

from zygo.cloud import fsspec_backend
from zygo.cloud.datasets.api import (
    CommitRequest,
    DataRepositoryApiClient,
    DataRepositoryApiError,
)


def test_environment_configuration(monkeypatch):
    monkeypatch.setenv("ZYGO_CLOUD_API_HOST", "https://example.com")
    monkeypatch.setenv("ZYGO_CLOUD_API_KEY", "scoped-token")
    fs = fsspec.filesystem("zygo")
    assert fs._client._api_host == "https://example.com"
    assert fs._client._api_bearer_auth == "scoped-token"


def test_complete_uri_routes_to_repository(monkeypatch):
    dataset = MagicMock()
    dataset.id = "workflow-one"
    dataset.open.return_value = BytesIO(b"upstream")
    dataset.ls.return_value = [{"name": "jobs/output.csv", "type": "file", "size": 8}]
    constructor = MagicMock(return_value=dataset)
    monkeypatch.setattr(fsspec_backend, "Dataset", constructor)
    fs = fsspec_backend.ZygoFileSystem(
        api_host="https://example.com", api_bearer_auth="token"
    )
    with fs.open("zygo://workflow-one/jobs/output.csv", "rb") as stream:
        assert stream.read() == b"upstream"
    constructor.assert_called_with("workflow-one", client=fs._client)
    dataset.open.assert_called_once()
    assert dataset.open.call_args.args == ("jobs/output.csv", "rb")
    assert fs.ls("zygo://workflow-one/jobs", detail=False) == [
        "workflow-one/jobs/output.csv"
    ]


def test_commit_retries_new_head(monkeypatch):
    commit = MagicMock(
        side_effect=[DataRepositoryApiError(409, "head changed"), "committed"]
    )
    monkeypatch.setattr(DataRepositoryApiClient, "commit", commit)
    monkeypatch.setattr(fsspec_backend.time, "sleep", lambda _: None)
    client = fsspec_backend._WorkflowRepositoryClient("https://example.com", "token")
    client.prepare_upload = MagicMock(return_value=MagicMock(head_commit_id="new-head"))
    request = MagicMock(spec=CommitRequest)
    assert client.commit("workflow-one", request) == "committed"
    request.model_copy.assert_called_once_with(
        update={"expected_head_commit_id": "new-head"}
    )


def test_permission_error_is_not_retried(monkeypatch):
    commit = MagicMock(side_effect=DataRepositoryApiError(403, "denied"))
    monkeypatch.setattr(DataRepositoryApiClient, "commit", commit)
    client = fsspec_backend._WorkflowRepositoryClient("https://example.com", "token")
    with pytest.raises(DataRepositoryApiError):
        client.commit("workflow-one", MagicMock())
    assert commit.call_count == 1
