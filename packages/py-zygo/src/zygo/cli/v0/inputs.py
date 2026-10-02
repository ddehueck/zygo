"""Validated CLI inputs and explicit mappings to generated protocol dataclasses.

Input policy belongs here rather than in generated.py. Executors and response
serializers continue to use the generated protocol types.
"""

from typing import Annotated

from pydantic import BaseModel, ConfigDict, Field, StringConstraints, field_validator

from zygo.cli.v0.types import (
    DatasetConfig,
    HttpConfig,
    JobRunArgs,
    StoreConfig,
    WorkflowStoreConfig,
)
from zygo.store import DataUri

type JsonValue = (
    str
    | int
    | Annotated[float, Field(allow_inf_nan=False)]
    | bool
    | list[JsonValue]
    | dict[str, JsonValue]
    | None
)
type NonemptyString = Annotated[str, StringConstraints(min_length=1)]
type HeaderName = Annotated[str, StringConstraints(strip_whitespace=True, min_length=1)]
type HeaderValue = Annotated[str, StringConstraints(strip_whitespace=True)]
type PositiveFiniteNumber = Annotated[float, Field(gt=0, allow_inf_nan=False)]


class CliInput(BaseModel):
    model_config = ConfigDict(strict=True, extra="forbid", frozen=True)


class JobRunInput(CliInput):
    job_id: str
    data_reference_uri: str
    workflow_run_id: str
    job_run_id: str

    def to_protocol(self) -> JobRunArgs:
        return JobRunArgs(
            job_id=self.job_id,
            data_reference_uri=self.data_reference_uri,
            workflow_run_id=self.workflow_run_id,
            job_run_id=self.job_run_id,
        )


class DatasetConfigInput(CliInput):
    uri: NonemptyString
    kwargs: dict[str, JsonValue] = Field(default_factory=dict)

    def to_protocol(self) -> DatasetConfig:
        return DatasetConfig(uri=self.uri, kwargs=self.kwargs)


class StoreConfigInput(CliInput):
    root_uri: str
    kwargs: dict[str, JsonValue] = Field(default_factory=dict)

    @field_validator("root_uri")
    @classmethod
    def normalize_root_uri(cls, value: str) -> str:
        uri = value if "://" in value else f"file://{value}"
        return DataUri(uri).uri

    def to_protocol(self) -> StoreConfig:
        return StoreConfig(root_uri=self.root_uri, kwargs=self.kwargs)


class WorkflowStoreConfigInput(CliInput):
    job: StoreConfigInput
    workflow: StoreConfigInput
    cache: StoreConfigInput

    def to_protocol(self) -> WorkflowStoreConfig:
        return WorkflowStoreConfig(
            job=self.job.to_protocol(),
            workflow=self.workflow.to_protocol(),
            cache=self.cache.to_protocol(),
        )


class HttpConfigInput(CliInput):
    url: str
    headers: dict[HeaderName, HeaderValue] = Field(default_factory=dict)
    timeout: PositiveFiniteNumber = 30.0
    max_retries: Annotated[int, Field(ge=0)] = 3
    retry_interval: PositiveFiniteNumber = 5.0

    @field_validator("url")
    @classmethod
    def validate_url(cls, value: str) -> str:
        if not value.startswith(("http://", "https://")):
            raise ValueError("must be a full http:// or https:// URL")
        return value

    def to_protocol(self) -> HttpConfig:
        return HttpConfig(
            url=self.url,
            headers=self.headers,
            timeout=self.timeout,
            max_retries=self.max_retries,
            retry_interval=self.retry_interval,
        )
