"""Console progress display for dataset edits."""

from __future__ import annotations

from contextlib import contextmanager
from typing import TYPE_CHECKING, override

from rich.console import Console
from rich.progress import (
    BarColumn,
    DownloadColumn,
    Progress,
    SpinnerColumn,
    TextColumn,
    TransferSpeedColumn,
)
from rich.text import Text

if TYPE_CHECKING:
    from collections.abc import Callable, Generator

    from rich.progress import Task, TaskID

__all__ = ["ProgressTracker"]


class _UploadSizeColumn(DownloadColumn):
    @override
    def render(self, task: Task) -> Text:
        return super().render(task) if task.total else Text("")


class _UploadSpeedColumn(TransferSpeedColumn):
    @override
    def render(self, task: Task) -> Text:
        return super().render(task) if task.total else Text("")


class ProgressTracker:
    """Display a dataset edit summary and its long-running stages."""

    def __init__(self) -> None:
        self._console = Console()
        self._progress: Progress | None = None
        self._stage_task_id: TaskID | None = None
        self._upload_total_blobs = 0
        self._upload_completed_blobs = 0
        self._upload_already_present = 0

    def display_data_change_summary(
        self, *, additions: int, file_removals: int, directory_removals: int
    ) -> None:
        """Print the operation summary before work begins."""

        self._console.print("Dataset changes")
        self._console.print(f"  Additions: {additions} files")
        self._console.print(
            f"  Removals: {file_removals + directory_removals} "
            f"({file_removals} files, {directory_removals} directories)"
        )

    @contextmanager
    def stage(self, label: str) -> Generator[None, None, None]:
        """Show a spinner while a stage runs, then mark its outcome."""

        succeeded = False
        try:
            with Progress(
                SpinnerColumn(),
                TextColumn("{task.description}", markup=False),
                BarColumn(bar_width=18),
                _UploadSizeColumn(binary_units=False),
                _UploadSpeedColumn(),
                console=self._console,
                transient=False,
            ) as progress:
                self._progress = progress
                self._stage_task_id = progress.add_task(label, total=None)
                yield
                succeeded = True
        except BaseException:
            self._console.print(f"✗ {label}", style="red")
            raise
        finally:
            self._progress = None
            self._stage_task_id = None
        if succeeded:
            self._console.print(f"✓ {label}", style="green")

    def configure_upload(
        self,
        *,
        total_blobs: int,
        upload_blobs: int,
        upload_bytes: int,
    ) -> None:
        """Turn the active upload stage into aggregate file and byte progress."""

        if self._progress is None or self._stage_task_id is None:
            return

        self._upload_total_blobs = upload_blobs
        self._upload_completed_blobs = 0
        self._upload_already_present = total_blobs - upload_blobs
        if upload_blobs and upload_bytes:
            self._progress.update(
                self._stage_task_id,
                description=self._upload_description(),
                total=upload_bytes,
                completed=0,
            )
        elif upload_blobs:
            self._progress.update(
                self._stage_task_id,
                description=self._upload_description(),
                total=None,
            )
        else:
            self._progress.update(
                self._stage_task_id,
                description=(
                    f"Upload data · all {total_blobs} blobs already present"
                    if total_blobs
                    else "Upload data · no blobs to upload"
                ),
                total=None,
            )

    @contextmanager
    def upload_file(
        self, path: str, size: int
    ) -> Generator[Callable[[int], None], None, None]:
        """Display an indented transfer row and yield its byte progress callback."""

        if self._progress is None:
            yield lambda _: None
            return

        task_id = self._progress.add_task(f"  {path}", total=size)
        try:
            yield lambda amount: self._advance_upload(task_id, amount)
        except BaseException:
            raise
        else:
            self._upload_completed_blobs += 1
            if self._stage_task_id is not None:
                self._progress.update(
                    self._stage_task_id,
                    description=self._upload_description(),
                )

    def _advance_upload(self, task_id: TaskID, amount: int) -> None:
        if self._progress is None:
            return
        self._progress.advance(task_id, amount)
        if self._stage_task_id is not None:
            self._progress.advance(self._stage_task_id, amount)

    def _upload_description(self) -> str:
        description = (
            f"Upload data · {self._upload_completed_blobs}/"
            f"{self._upload_total_blobs} blobs"
        )
        if self._upload_already_present:
            description += f" · {self._upload_already_present} already present"
        return description
