from __future__ import annotations

import sys
from typing import TYPE_CHECKING, Any, override

from zygo._internal.python.fsspec import FsspecUri
from zygo.backends.exec import get_exec_command_builder
from zygo.backends.exec.packages import PackagesExecCommand
from zygo.backends.exec.uv import UvExecCommand
from zygo.backends.protocol import (
    Backend,
    LocalEntrypoint,
)
from zygo.store import StoreOptions
from zygo.types import JobFnName

if TYPE_CHECKING:
    from pathlib import Path

    from zygo.backends.exec import ExecCommand
    from zygo.backends.protocol import Entrypoint, JobConfig
    from zygo.types import Environment, JobName


class DefaultBackend(Backend):
    """
    The default backend for local development and single-machine workflows.

    Store:
        Accepts any fsspec-compatible URI as ``data_dir``, including local
        paths (``file://``, ``memory://``) and remote object stores
        (``s3://``, ``gs://``, etc.).  Local filesystem stores are permitted
        (``allow_local_store = True``).

    Execution:
        All jobs run on the **local machine**.  ``deploy()`` returns a
        ``LocalEntrypoint`` for every job — no remote infrastructure is
        provisioned.  The working directory is set to ``data_dir``.
        The exec command is resolved from each job's ``Environment``:
        currently only ``uv`` is supported (triggered by ``uv_lock``
        or used as the default).

    The module path (absolute path to the workflow's Python module) is
    supplied by the workflow at ``deploy()`` time.  The project root
    (used as cwd for local execution) is derived as ``module_path.parent``.

    Example:
        ```python
        workflow = Workflow(id="dev")
        ...

        backend = DefaultBackend(data_dir="file://data/")
        workflow.run(channel=Channel(name="input"), uri="file://input.csv", backend=backend)
        ```
    """

    def __init__(
        self,
        *,
        store_uri: str,
        store_options: dict[str, Any] | None = None,  # pyright: ignore[reportExplicitAny]
    ) -> None:
        self._store_uri = FsspecUri(store_uri)
        self._store_options = store_options or {}
        super().__init__()

    @property
    @override
    def store_options(self) -> StoreOptions:
        return StoreOptions(root_uri=self._store_uri, kwargs=dict(self._store_options))

    @property
    @override
    def allow_local_store(self) -> bool:
        return True

    @property
    @override
    def orchestrator_uri(self) -> str:
        return "localhost:50051"

    @override
    def deploy(
        self, jobs: list[JobConfig], content_hash: str, module_path: Path
    ) -> dict[JobName, Entrypoint]:
        project_root = module_path.parent
        return {
            job.id: LocalEntrypoint(
                cwd=project_root,
                exec=self._build_exec(
                    environment=job.environment,
                    module_path=module_path,
                    job_fn_name=JobFnName(job.id),
                ),
                env=job.env,
            )
            for job in jobs
        }

    @staticmethod
    def _build_exec(
        *, environment: Environment, module_path: Path, job_fn_name: JobFnName
    ) -> str:
        """Resolve an exec shell command from the job's environment spec."""
        python_version = (
            environment.python_version
            or f"{sys.version_info.major}.{sys.version_info.minor}"
        )

        cmd: ExecCommand | None = None
        if environment.uv_lock:
            cmd = UvExecCommand(
                python_version=python_version,
                project_path=environment.uv_lock,
            )
        elif environment.packages:
            cmd = PackagesExecCommand(
                python_version=python_version,
                packages=environment.packages,
            )

        if cmd is None:
            raise ValueError(
                "Cannot run this job environment on this backend. Please specify a uv_lock file."
            )

        builder = get_exec_command_builder(cmd)
        builder.validate_system()
        base_command = builder.build(str(module_path))
        return f"{base_command} --job-fn-name {job_fn_name}"
