"""Workflow deployment API client."""

from __future__ import annotations

from pydantic import ValidationError

from zygo.cloud.api import (
    CloudApiClient,
    CloudApiError,
    DeployWorkflowRequest,
    DeployWorkflowResponse,
)

__all__ = ["WorkflowApiClient"]

_DEPLOY_PATH = "/api/v1/workflows/deploy"


class WorkflowApiClient:
    """Create or update workflow records in Zygo Cloud."""

    def __init__(self, client: CloudApiClient | None = None) -> None:
        self._client = client if client is not None else CloudApiClient()

    def deploy(
        self,
        *,
        workflow_id: str,
        target: str,
        source_id: str,
        image_id: str,
    ) -> DeployWorkflowResponse:
        """Deploy a workflow using ready source and a started image build."""

        request = DeployWorkflowRequest(
            workflow_id=workflow_id,
            target=target,
            source_id=source_id,
            image_id=image_id,
        )
        response = self._client.post_json(_DEPLOY_PATH, request.model_dump(mode="json"))
        try:
            return DeployWorkflowResponse.model_validate(response)
        except ValidationError as error:
            raise CloudApiError(
                None, f"Invalid workflow deployment response: {error}"
            ) from error
