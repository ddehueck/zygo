"""Validated CLI inputs and explicit mappings to generated protocol dataclasses.

Input policy belongs here rather than in generated.py. Executors and response
serializers continue to use the generated protocol types.
"""

from typing import Annotated, Literal, Self

from pydantic import (
    BaseModel,
    ConfigDict,
    Field,
    StringConstraints,
    field_validator,
    model_validator,
)

from zygo.cli.v0.types import (

    HttpConfig,
    JobRunArgs,
    ModelTrainCommand,
    StoreConfig,
    WorkflowMetadataCommand,
    WorkflowRunCommand,
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


class WorkflowRunCommandInput(CliInput):
    command: Literal["workflow_run"]
    target: NonemptyString
    args: JobRunInput
    job_store_config: StoreConfigInput | None = None
    workflow_store_config: StoreConfigInput | None = None
    cache_store_config: StoreConfigInput | None = None
    http_config: HttpConfigInput | None = None

    @field_validator(
        "job_store_config", "workflow_store_config", "cache_store_config", mode="before"
    )
    @classmethod
    def reject_null_store_config(cls, value: object) -> object:
        if value is None:
            raise ValueError("must be a store configuration object when supplied")
        return value

    @model_validator(mode="after")
    def validate_store_configs(self) -> Self:
        configs = (
            self.job_store_config,
            self.workflow_store_config,
            self.cache_store_config,
        )
        if any(config is not None for config in configs) and not all(
            config is not None for config in configs
        ):
            raise ValueError(
                "job_store_config, workflow_store_config, and cache_store_config "
                "must be supplied together"
            )
        return self

    def to_protocol(self) -> WorkflowRunCommand:
        return WorkflowRunCommand(
            command=self.command,
            target=self.target,
            args=self.args.to_protocol(),
            job_store_config=(
                self.job_store_config.to_protocol() if self.job_store_config else None
            ),
            workflow_store_config=(
                self.workflow_store_config.to_protocol()
                if self.workflow_store_config
                else None
            ),
            cache_store_config=(
                self.cache_store_config.to_protocol() if self.cache_store_config else None
            ),
            http_config=self.http_config.to_protocol() if self.http_config else None,
        )


class WorkflowMetadataCommandInput(CliInput):
    command: Literal["workflow_metadata"]
    target: NonemptyString

    def to_protocol(self) -> WorkflowMetadataCommand:
        return WorkflowMetadataCommand(command=self.command, target=self.target)


class ModelTrainCommandInput(CliInput):
    command: Literal["model_train"]
    target: NonemptyString
    dataset_config: StoreConfigInput
    store_config: StoreConfigInput
    http_config: HttpConfigInput | None = None

    def to_protocol(self) -> ModelTrainCommand:
        return ModelTrainCommand(
            command=self.command,
            target=self.target,
            dataset_config=self.dataset_config.to_protocol(),
            store_config=self.store_config.to_protocol(),
            http_config=self.http_config.to_protocol() if self.http_config else None,
        )


type CliCommandInput = Annotated[
    WorkflowRunCommandInput | WorkflowMetadataCommandInput | ModelTrainCommandInput,
    Field(discriminator="command"),
]
