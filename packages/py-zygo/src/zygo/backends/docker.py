from __future__ import annotations

import logging
from pathlib import Path
import shutil
import subprocess
from typing import TYPE_CHECKING, Any, override

from zygo._internal.python.fsspec import FsspecUri
from zygo.backends.protocol import Backend, LocalEntrypoint
from zygo.store import StoreOptions
from zygo.types import JobFnName

if TYPE_CHECKING:
    from zygo.backends.protocol import Entrypoint, JobConfig
    from zygo.types import JobName

logger = logging.getLogger(__name__)


class DockerBackend(Backend):
    """
    A backend that builds a user-provided Dockerfile per job and generates
    ``docker run`` commands.

    Store:
        Mirrors :class:`DefaultBackend` — accepts any fsspec-compatible URI as
        ``store_uri`` and permits local filesystem stores.

    Execution:
        For each job the backend:

        1. Builds the image from the supplied ``dockerfile`` (tagged
           ``<image_prefix>-<job_id>:<hash>``).
        2. Returns a :class:`LocalEntrypoint` whose ``exec`` is a
           ``docker run`` command that reproduces the same CLI interface the
           orchestrator expects.

        The container runs with ``--network=host`` so it can reach a
        local orchestrator, and resource constraints (cpu / memory / gpu) from
        the ``Environment`` are translated to the matching Docker flags.

    Example::

        backend = DockerBackend(
            store_uri="./zygo",
            dockerfile="examples/Dockerfile",
            build_context=".",          # defaults to module_path.parent
            image_prefix="myapp",
            orchestrator_uri="host.docker.internal:50051",  # default; use localhost:50051 on Linux
        )
        workflow.run(channel=input_ch, uri="file://data.csv", backend=backend)
    """

    def __init__(
        self,
        *,
        store_uri: str,
        dockerfile: str | Path,
        store_options: dict[str, Any] | None = None,  # pyright: ignore[reportExplicitAny]
        build_context: str | Path | None = None,
        orchestrator_uri: str = "host.docker.internal:50051",
    ) -> None:
        self._store_uri = FsspecUri(store_uri)
        self._store_options = store_options or {}
        self._dockerfile = Path(dockerfile)
        self._image_prefix = "zygo"
        self._build_context = Path(build_context) if build_context is not None else None
        self._orchestrator_uri = orchestrator_uri
        super().__init__()

    # ── Backend protocol ─────────────────────────────────────────────

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
        return self._orchestrator_uri

    @override
    def deploy(
        self, jobs: list[JobConfig], content_hash: str, module_path: Path
    ) -> dict[JobName, Entrypoint]:
        self._require_docker()
        project_root = module_path.parent
        context = self._resolve_build_context(project_root)
        relative_module = module_path.relative_to(context)

        dockerfile = (
            self._dockerfile
            if self._dockerfile.is_absolute()
            else context / self._dockerfile
        )
        if not dockerfile.is_file():
            raise FileNotFoundError(f"Dockerfile not found: {dockerfile}")

        entrypoints: dict[JobName, Entrypoint] = {}
        for job in jobs:
            image_tag = self._build_image(
                job=job,
                content_hash=content_hash,
                context=context,
                dockerfile=dockerfile,
            )
            exec_cmd = self._build_run_command(
                image_tag=image_tag,
                relative_module=relative_module,
                job=job,
                job_fn_name=JobFnName(job.id),
                context=context,
            )
            entrypoints[job.id] = LocalEntrypoint(
                cwd=context,
                exec=exec_cmd,
                env=job.env,
            )
        return entrypoints

    # ── Image building ───────────────────────────────────────────────

    def _build_image(
        self,
        *,
        job: JobConfig,
        content_hash: str,
        context: Path,
        dockerfile: Path,
    ) -> str:
        image_tag = f"{self._image_prefix}-{job.id}:{content_hash[:12]}"

        logger.info("Building Docker image %s for job %s", image_tag, job.id)
        subprocess.run(
            [
                "docker",
                "build",
                "-t",
                image_tag,
                "-f",
                str(dockerfile),
                ".",
            ],
            cwd=str(context),
            check=True,
        )

        return image_tag

    # ── Run command generation ───────────────────────────────────────

    def _build_run_command(
        self,
        *,
        image_tag: str,
        relative_module: Path,
        job: JobConfig,
        job_fn_name: JobFnName,
        context: Path,
    ) -> str:
        parts: list[str] = ["docker", "run", "--rm", "--network=host"]

        env = job.environment

        if env.cpu is not None:
            parts.extend(["--cpus", str(env.cpu)])
        if env.memory is not None:
            parts.extend(["--memory", f"{env.memory}m"])
        if env.gpu is not None:
            gpu_count = env.gpu_count or 1
            devices = ",".join(str(i) for i in range(gpu_count))
            parts.extend(["--gpus", f'"device={devices}"'])

        if self._store_uri.is_local():
            host_store_path = (context / self._store_uri.path).resolve()
            parts.extend(["-v", f"{host_store_path}:{host_store_path}"])

        if job.env:
            for key, value in job.env.items():
                parts.extend(["-e", f"{key}={value}"])

        parts.append(image_tag)
        parts.extend([
            "python",
            str(relative_module),
            "--job-fn-name",
            str(job_fn_name),
        ])

        return " ".join(parts)

    # ── Helpers ──────────────────────────────────────────────────────

    def _resolve_build_context(self, project_root: Path) -> Path:
        if self._build_context is None:
            return project_root
        if self._build_context.is_absolute():
            return self._build_context
        return (project_root / self._build_context).resolve()

    @staticmethod
    def _require_docker() -> None:
        if shutil.which("docker") is None:
            raise RuntimeError(
                "docker is not installed or not on PATH. "
                + "Install it: https://docs.docker.com/get-docker/"
            )
