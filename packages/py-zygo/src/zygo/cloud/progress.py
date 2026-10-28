"""Progress output for cloud operations."""

from __future__ import annotations

from contextlib import contextmanager
from typing import TYPE_CHECKING

from rich.console import Console
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

    @contextmanager
    def stage(self, label: str) -> Generator[None, None, None]:
        """Show a stage while it runs and report its result."""

        if not self._interactive:
            self._console.print(f"Starting: {label}", markup=False)

        try:
            if self._interactive:
                with self._console.status(Text(label), spinner="dots"):
                    yield
            else:
                yield
        except BaseException as error:
            self._print_result(label, error)
            raise
        else:
            self._print_result(label)

    def _print_result(self, label: str, error: BaseException | None = None) -> None:
        failed = error is not None
        result = Text("✗ " if failed else "✓ ", style="red" if failed else "green")
        result.append(label)
        if error is not None:
            message = str(error).strip() or type(error).__name__
            result.append(f": {message}")
        self._console.print(result)
