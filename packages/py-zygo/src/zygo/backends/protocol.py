"""
A backend is class that provides a way to deploy and run the jobs of a workflow.

It is responsible for:
- Deploying the workflow source code to a target environment
- Providing an entrypoint that the orchestrator can use to run a job

Because the different jobs can have different dependencies, the backend must
know how to resolve the dependencies for each job.

If a remote backend is used, the backend must:
- Implement the `deploy` method to deploy the workflow source code to a target environment
- Return a remote entrypoint that the orchestrator can use to run a job. e.g. an HTTP endpoint.

A user will specify the backend to use in a specific workflow run.
The mental model is that "a workflow runs on a backend".

The module path (i.e. the absolute path to the Python module containing the workflow)
is determined automatically at ``workflow.run()`` time and passed to ``deploy()``.
Backends derive the project root (cwd) from ``module_path.parent``.

```python
workflow = Workflow(
    id="my_workflow",
    backend=MyBackend(),
)
```

1. A user specifies the backend to use in a specific workflow run.
2. The backend calls a `deploy` method and has access to the workflow source code root.
"""

from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from zygo.store.types import StoreOptions
from zygo.types import Environment, JobName


@dataclass(frozen=True)
class RemoteEntrypoint:
    url: str
    headers: dict[str, str]


@dataclass(frozen=True)
class LocalEntrypoint:
    cwd: Path
    exec: str  # This will be a posix/windows shell command that can be execution with a --run_jobs_arg
    env: dict[str, str] | None = None


@dataclass(frozen=True)
class JobConfig:
    id: JobName
    environment: Environment
    env: dict[str, str] | None = None


type Entrypoint = RemoteEntrypoint | LocalEntrypoint


class Backend(Protocol):
    """
    A backend is a class that provides a way to deploy and run the jobs of a workflow.

    ```python
    my_backend = MyBackend(store_uri="s3://my-bucket/my-workflow", api_key="my-api-key")
    ```
    """

    @property
    def allow_local_store(self) -> bool:
        """Whether to allow a local filesystem store to be used.
        e.g. if a local filesystem store is used, this backend will raise an error.
        """
        raise NotImplementedError("Backend's allow_local_store is not implemented.")

    @property
    def store_options(self) -> StoreOptions:
        """The options for the store to use for the workflow."""
        raise NotImplementedError("Backend's store_options is not implemented.")

    @property
    def orchestrator_uri(self) -> str:
        """The URI of the orchestrator to use for the workflow."""
        raise NotImplementedError("Backend's orchestrator is not implemented.")

    def deploy(
        self, jobs: list[JobConfig], content_hash: str, module_path: Path
    ) -> dict[JobName, Entrypoint]:
        """Deploy the workflow source code to a target environment.

        Args:
            jobs: A list of job configurations.
            content_hash: The content hash of the workflow source code to act as a cache key.
            module_path: Absolute path to the Python module that contains the workflow.
                         Determined automatically by ``workflow.run()``.
                         Backends derive the project root (cwd) from ``module_path.parent``.

        Returns:
            A dictionary of job names to entrypoints.
        """
        raise NotImplementedError("Backend's deploy is not implemented.")
