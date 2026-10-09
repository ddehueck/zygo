"""Request and response contract for cloud training."""

import re
from typing import Annotated, Any, Literal, cast
from urllib.parse import quote

from pydantic import (
    BaseModel,
    ConfigDict,
    Field,
    SerializerFunctionWrapHandler,
    field_validator,
    model_serializer,
)

from zygo.cloud.api import ZYGO_CLOUD_HOST

_CONFIG = ConfigDict(extra="forbid", protected_namespaces=())
_RELATIVE_PATH = re.compile(
    r"^(?!/)(?!.*//)(?!.*(?:^|/)\.{1,2}(?:/|$))"
    r"(?!.*[\\\x00-\x1F\x7F-\x9F]).*[^/]$"
)


class CloudModel(BaseModel):
    """JSON object exchanged with the Zygo Cloud API."""

    model_config = _CONFIG


class CloudRequest(CloudModel):
    """POST body that names its own route."""

    @classmethod
    def method(cls) -> str:
        return "POST"


class DeclareSourceRequest(CloudRequest):
    """Resolve a source archive by hash."""

    source_hash: str

    @classmethod
    def url(cls) -> str:
        return f"{ZYGO_CLOUD_HOST}/api/v1/source"


class SourceAwaitingUpload(CloudModel):
    """The archive hash is new, so the client must PUT the ZIP."""

    status: Literal["awaiting_upload"]
    source_hash: str
    upload_url: str
    expires_at: str


class SourceReady(CloudModel):
    """A stored archive the runtime request can reference."""

    status: Literal["ready"]
    source_id: str
    source_hash: str
    reused: bool


SourceDeclaration = Annotated[
    SourceAwaitingUpload | SourceReady,
    Field(discriminator="status"),
]


class ImageDefinition(CloudModel):
    """Paths of the image files inside the source ZIP.

    The server reads these files from the archive and hashes them. The
    archive is the Docker build context and is extracted into the sandbox
    when the run starts. ``force_rebuild`` is omitted when false so the
    server default applies.
    """

    dockerfile: str = Field(min_length=1, max_length=1024)
    pyproject: str = Field(min_length=1, max_length=1024)
    uv_lock: str = Field(min_length=1, max_length=1024)
    force_rebuild: bool = False

    @field_validator("dockerfile", "pyproject", "uv_lock")
    @classmethod
    def _relative_archive_path(cls, value: str) -> str:
        if _RELATIVE_PATH.fullmatch(value) is None:
            raise ValueError("must be a relative path inside the source archive")
        return value

    @model_serializer(mode="wrap")
    def _omit_default_rebuild(
        self, handler: SerializerFunctionWrapHandler
    ) -> dict[str, object]:
        dumped: object = handler(self)
        if not isinstance(dumped, dict):
            raise TypeError("Image definition did not serialize to an object")
        data = cast("dict[str, object]", dumped)
        if data.get("force_rebuild") is False:
            data.pop("force_rebuild")
        return data


class StartModelTrainingRunRequest(CloudRequest):
    """Body for starting a model training run.

    The request is sent once. ``params`` is the validated hyperparameter
    object, or an empty object when the model declares none.
    """

    type: Literal["model_training_run"]
    source_id: str
    target: str  # python module path e.g. `main:workflow`
    model_id: str
    dataset_version_id: str
    image: ImageDefinition
    params: dict[str, Any]

    @classmethod
    def url(cls) -> str:
        return f"{ZYGO_CLOUD_HOST}/api/v1/runs"


class ModelTrainingRun(CloudModel):
    """A workflow or model training run.

    ``building`` means the image is still building. ``failed`` carries
    ``message``. A failed or cancelled run is a normal result, not an
    exception.
    """

    id: str
    status: Literal["building", "running", "succeeded", "cancelled", "failed"]
    message: str | None = None


class RunLogLine(CloudModel):
    """One server-sent line from a run log or an image build log."""

    stream: Literal["stdout", "stderr"]
    sequence: int = Field(ge=0)
    line: str
    created_at: str


def run_status_url(run_id: str) -> str:
    """URL for the current status of a workflow or model training run."""
    return f"{ZYGO_CLOUD_HOST}/api/v1/runs/{quote(run_id, safe='')}/status"


def run_logs_url(run_id: str) -> str:
    """URL for the server-sent stdout and stderr of a run."""
    return f"{ZYGO_CLOUD_HOST}/api/v1/runs/{quote(run_id, safe='')}/logs"


def run_build_logs_url(run_id: str) -> str:
    """URL for the server-sent image build log of a run."""
    return f"{ZYGO_CLOUD_HOST}/api/v1/runs/{quote(run_id, safe='')}/build/logs"
