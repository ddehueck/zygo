"""Build the source and image inputs and deploy a workflow to Zygo Cloud."""

from __future__ import annotations

import inspect
import sys
from typing import TYPE_CHECKING

from zygo.cloud.api import CloudApiClient
from zygo.cloud.image_build_logs import ImageBuildLogsApiClient
from zygo.cloud.images import ImageApiClient
from zygo.cloud.progress import ProgressLogger
from zygo.cloud.source import SourceApiClient
from zygo.cloud.workflow.client import WorkflowApiClient
from zygo.cloud.workflow.project import (
    infer_workflow_target,
    resolve_deployment_project,
)
from zygo.workflow import Workflow

if TYPE_CHECKING:
    from pathlib import Path
    from types import FrameType

    from zygo.cloud.api import DeployWorkflowResponse
    from zygo.cloud.workflow.project import DeploymentModuleContext

__all__ = ["deploy_workflow"]


def deploy_workflow(
    *,
    workflow: Workflow,
    project_dir: str | Path,
) -> DeployWorkflowResponse:
    """Deploy a workflow from a local Python project.

    We assume the ``project_dir`` is a UV project to build the OCI image the workflow
    needs to run in a cloud environment. Specifically, it must contain a
    ``pyproject.toml`` file and a ``uv.lock`` file.

    A python version must be specified in the ``pyproject.toml`` file in the
    ``[tool.zygo.cloud]`` section or in a ``.python-version`` file.
    If both are present, they must resolve to the same Python version.
    """

    if not isinstance(workflow, Workflow):
        raise TypeError("workflow must be a zygo.workflow.Workflow instance")

    frame = inspect.currentframe()
    caller = frame.f_back if frame is not None else None
    module_contexts = _deployment_module_contexts(workflow, caller)
    del caller, frame

    progress = ProgressLogger()
    with progress.stage(f"Packaging workflow source code from {project_dir!r}"):
        target, target_file = infer_workflow_target(
            project_dir, workflow, module_contexts
        )
        project = resolve_deployment_project(
            project_dir,
            workflow,
            target,
            target_file,
        )

    client = CloudApiClient()
    with progress.stage("Uploading workflow source code"):
        source = SourceApiClient(client).ensure_source(project.source_archive)

    with progress.stage("Building Python image"):
        image = ImageApiClient(client).create(
            python_version=project.python_version,
            pyproject_toml=project.pyproject_toml,
            uv_lock=project.uv_lock,
        )
        if not image.reused:
            _watch_image_build_logs(client, image.image_id, progress)

    with progress.stage("Finalizing workflow deployment"):
        return WorkflowApiClient(client).deploy(
            workflow_id=project.workflow_id,
            target=project.target,
            source_id=source.source_id,
            image_id=image.image_id,
        )


def _watch_image_build_logs(
    client: CloudApiClient, image_id: str, progress: ProgressLogger
) -> None:
    progress.wait_for_logs("Waiting for image build logs…")
    for event in ImageBuildLogsApiClient(client).watch(image_id):
        if event.message is not None:
            progress.log(event.message)


def _deployment_module_contexts(
    workflow: Workflow, caller: FrameType | None
) -> tuple[DeploymentModuleContext, ...]:
    contexts: list[DeploymentModuleContext] = []
    origin_name = workflow._definition_module_name
    if origin_name is not None:
        origin_module = sys.modules.get(origin_name)
        if origin_module is not None:
            origin_file = getattr(origin_module, "__file__", None)
            if not isinstance(origin_file, str):
                origin_file = workflow._definition_file
            if isinstance(origin_file, str):
                contexts.append((origin_name, origin_file, vars(origin_module)))

    if caller is not None:
        caller_globals = caller.f_globals
        caller_name = caller_globals.get("__name__")
        caller_file = caller_globals.get("__file__")
        if isinstance(caller_file, str):
            context = (
                caller_name if isinstance(caller_name, str) else None,
                caller_file,
                caller_globals,
            )
            if not any(
                existing[0] == context[0] and existing[1] == context[1]
                for existing in contexts
            ):
                contexts.append(context)
    return tuple(contexts)
