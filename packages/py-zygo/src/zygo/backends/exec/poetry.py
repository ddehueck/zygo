from dataclasses import dataclass
import shutil
from typing import override

from zygo.backends.exec.protocol import ExecCommandBuilder


@dataclass(frozen=True)
class PoetryExecCommand:
    directory: str


@dataclass(frozen=True)
class PoetryExecCommandBuilder(ExecCommandBuilder):
    cmd: PoetryExecCommand

    @override
    def validate_system(self) -> None:
        if shutil.which("poetry") is None:
            raise RuntimeError(
                "poetry is not installed or not on PATH. "
                + "Install it: https://python-poetry.org/docs/#installation"
            )

    @override
    def build(self, module_path: str) -> str:
        return f"poetry run --directory {self.cmd.directory} python {module_path}"
