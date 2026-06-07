from dataclasses import dataclass
import shutil
from typing import override

from zygo.backends.exec.protocol import ExecCommandBuilder


@dataclass(frozen=True)
class PackagesExecCommand:
    python_version: str
    packages: list[str]


@dataclass(frozen=True)
class PackagesExecCommandBuilder(ExecCommandBuilder):
    cmd: PackagesExecCommand

    @override
    def validate_system(self) -> None:
        if shutil.which("uv") is None:
            raise RuntimeError(
                "uv is not installed or not on PATH. Use uv to install the packages."
                + "Install it: https://docs.astral.sh/uv/getting-started/installation/"
            )

    @override
    def build(self, module_path: str) -> str:
        with_flags = " ".join(f"--with {pkg}" for pkg in self.cmd.packages)
        return f"uv run --python {self.cmd.python_version} {with_flags} {module_path}"
