"""Build the source and image inputs and deploy a workflow to Zygo Cloud."""

from __future__ import annotations

from typing import TYPE_CHECKING

from zygo.cloud.api import CloudApiClient, DeployWorkflowResponse
from zygo.cloud.images import ImageApiClient
from zygo.cloud.source import SourceApiClient
from zygo.cloud.workflow.client import WorkflowApiClient
from zygo.cloud.workflow.project import resolve_deployment_project

if TYPE_CHECKING:
    from pathlib import Path

    from zygo.cloud.api import DeployWorkflowResponse

__all__ = ["deploy_workflow"]


def deploy_workflow(*, project_dir: str | Path, target: str) -> DeployWorkflowResponse:
    """Deploy a local Python workflow without importing its module locally.

    ``project_dir`` must contain ``pyproject.toml`` and ``uv.lock``. ``target``
    follows the Python module syntax accepted by the workflow API, such as
    ``workflow`` or ``my_package.workflow:my_workflow``. The selected Python
    version is read from ``[tool.zygo.cloud].python_version`` in the project
    ``pyproject.toml``.
    """

    project = resolve_deployment_project(project_dir, target)
    client = CloudApiClient()
    source = SourceApiClient(client).ensure_source(project.source_archive)
    image = ImageApiClient(client).create(
        python_version=project.python_version,
        pyproject_toml=project.pyproject_toml,
        uv_lock=project.uv_lock,
    )
    return WorkflowApiClient(client).deploy(
        workflow_id=project.workflow_id,
        target=project.target,
        source_id=source.source_id,
        image_id=image.image_id,
    )
