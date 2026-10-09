"""
Defines the image config and parsing that zygo uses to build images
to run user defined code within.
"""

from pathlib import Path
import tomllib

from pydantic import BaseModel, Field


class ZygoCloudProjectConfig(BaseModel):
    src_dir: Path = Field(default_factory=Path.cwd)
    dockerfile: str | None = None
    uv_lockfile: str


def from_toml(pyproject_toml_path: str) -> ZygoCloudProjectConfig:
    """Load a Zygo cloud project config from a TOML file.

    Looks for a tools.zygo.cloud section in the project's pyproject.toml file.
    """
    with Path(pyproject_toml_path).open("rb") as f:
        pyproject_data = tomllib.load(f)

    return ZygoCloudProjectConfig.model_validate(
        pyproject_data["tools"]["zygo"]["cloud"]
    )
