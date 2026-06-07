from dataclasses import dataclass
import shutil
from typing import override

from zygo.backends.exec.protocol import ExecCommandBuilder


@dataclass(frozen=True)
class CondaExecCommand:
    prefix_path: str


@dataclass(frozen=True)
class CondaExecCommandBuilder(ExecCommandBuilder):
    cmd: CondaExecCommand

    @override
    def validate_system(self) -> None:
        if shutil.which("conda") is None:
            raise RuntimeError(
                "conda is not installed or not on PATH. "
                + "Install it: https://docs.conda.io/projects/conda/en/latest/user-guide/install/index.html"
            )

    @override
    def build(self, module_path: str) -> str:
        return f"conda run --prefix {self.cmd.prefix_path} --no-capture-output python {module_path}"
