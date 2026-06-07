from typing import Protocol


class ExecCommandBuilder(Protocol):
    def validate_system(self) -> None:
        raise NotImplementedError("validate_system is not implemented")

    def build(self, module_path: str) -> str:
        raise NotImplementedError("build is not implemented")
