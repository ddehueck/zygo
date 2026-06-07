from typing import assert_never

from zygo.backends.exec.conda import CondaExecCommand, CondaExecCommandBuilder
from zygo.backends.exec.packages import (
    PackagesExecCommand,
    PackagesExecCommandBuilder,
)
from zygo.backends.exec.poetry import PoetryExecCommand, PoetryExecCommandBuilder
from zygo.backends.exec.protocol import ExecCommandBuilder
from zygo.backends.exec.uv import UvExecCommand, UvExecCommandBuilder

type ExecCommand = (
    UvExecCommand | PoetryExecCommand | CondaExecCommand | PackagesExecCommand
)


def get_exec_command_builder(cmd: ExecCommand) -> ExecCommandBuilder:
    match cmd:
        case UvExecCommand():
            return UvExecCommandBuilder(cmd)
        case PoetryExecCommand():
            return PoetryExecCommandBuilder(cmd)
        case CondaExecCommand():
            return CondaExecCommandBuilder(cmd)
        case PackagesExecCommand():
            return PackagesExecCommandBuilder(cmd)
        case _ as unreachable:
            assert_never(unreachable)
