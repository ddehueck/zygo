"""Progress output for cloud operations."""

from __future__ import annotations

from contextlib import contextmanager, nullcontext
from typing import TYPE_CHECKING

from rich.console import Console, Group
from rich.live import Live
from rich.spinner import Spinner
from rich.text import Text

if TYPE_CHECKING:
    from collections.abc import Generator

__all__ = ["ProgressLogger"]


class ProgressLogger:
    """Report operation stages with a spinner or plain stderr messages."""

    def __init__(self) -> None:
        self._console = Console(stderr=True, highlight=False)
        self._interactive = (
            self._console.is_terminal and not self._console.is_dumb_terminal
        )
        self._live: Live | None = None
        self._spinner = Spinner("dots", style="cyan")
        self._logs: list[Text] = []

    def wait_for_logs(self, message: str) -> None:
        """Show a placeholder until the first log message arrives."""

        placeholder = Text(f"  {message}", style="dim")
        if self._live is not None:
            self._live.update(Group(self._spinner, placeholder), refresh=True)
        else:
            self._console.print(placeholder)

    def log(self, message: str) -> None:
        """Show muted, indented log lines beneath the current stage title."""

        lines = [Text(f"  {line}", style="dim") for line in message.splitlines()]
        if not lines:
            return
        if self._live is None:
            self._console.print(Group(*lines))
            return
        self._logs.extend(lines)
        visible_lines = max(1, self._console.height - 2)
        self._live.update(Group(self._spinner, *self._logs[-visible_lines:]))

    @contextmanager
    def stage(self, label: str) -> Generator[None, None, None]:
        """Show a stage while it runs and report its result."""

        if not self._interactive:
            self._console.print(f"Starting: {label}", markup=False)

        self._logs = []
        self._spinner = Spinner("dots", text=Text(label), style="cyan")
        display = (
            Live(self._spinner, console=self._console)
            if self._interactive
            else nullcontext()
        )
        with display as live:
            self._live = live
            try:
                yield
            except BaseException as error:
                self._print_result(label, error)
                raise
            else:
                self._print_result(label)
            finally:
                self._live = None

    def _print_result(self, label: str, error: BaseException | None = None) -> None:
        failed = error is not None
        result = Text()
        result.append("✗ " if failed else "✓ ", style="red" if failed else "green")
        result.append(label)
        if error is not None:
            message = str(error).strip() or type(error).__name__
            result.append(f": {message}")
        if self._live is not None:
            self._live.update(Group(result, *self._logs))
        else:
            self._console.print(result)
