"""Validated, JSON-serializable inputs for model training."""

from __future__ import annotations

import json
from typing import Self

from pydantic import BaseModel, ConfigDict, model_validator
from pydantic_core import PydanticSerializationError


class HyperParams(BaseModel):
    """Declare training inputs with typed fields and optional defaults."""

    model_config = ConfigDict(
        extra="forbid",
        validate_default=True,
        validate_assignment=True,
        revalidate_instances="always",
        allow_inf_nan=False,
        ser_json_inf_nan="constants",
    )

    @model_validator(mode="after")
    def _require_json_serializable(self) -> Self:
        try:
            _ = json.dumps(
                self.model_dump(mode="json", warnings="error"), allow_nan=False
            )
        except (TypeError, ValueError, PydanticSerializationError) as error:
            raise ValueError(
                f"Hyperparameters must be JSON serializable: {error}"
            ) from error
        return self
