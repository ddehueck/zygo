from dataclasses import dataclass
import shutil
from typing import override

from zygo.backends.exec.protocol import ExecCommandBuilder


@dataclass(frozen=True)
class UvExecCommand:
    python_version: str
    project_path: str


@dataclass(frozen=True)
class UvExecCommandBuilder(ExecCommandBuilder):
    cmd: UvExecCommand

    @override
    def validate_system(self) -> None:
        if shutil.which("uv") is None:
            raise RuntimeError(
                "uv is not installed or not on PATH. "
                + "Install it: https://docs.astral.sh/uv/getting-started/installation/"
            )

    @override
    def build(self, module_path: str) -> str:
        return (
            f"uv run"
            f" --locked"
            f" --python {self.cmd.python_version}"
            f" --project {self.cmd.project_path}"
            f" {module_path}"
        )
