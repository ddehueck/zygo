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


class _TaskProgressColumn(DownloadColumn):
    @override
    def render(self, task: Task) -> Text:
        unit = task.fields.get("unit")
        if unit is not None and task.total is not None:
            return Text(f"{task.completed:.0f}/{task.total:.0f} {unit}")
        return super().render(task) if task.total else Text("")


class _TaskSpeedColumn(TransferSpeedColumn):
    @override
    def render(self, task: Task) -> Text:
        unit = task.fields.get("unit")
        if unit is not None and task.total is not None:
            return Text(f"{task.speed:.1f} {unit}/s") if task.speed else Text("")
        if unit is not None:
            return Text("")
        return super().render(task) if task.total else Text("")


class ProgressTracker:
    """Display a dataset edit summary and its long-running stages."""

    def __init__(self) -> None:
        self._console = Console()
        self._rich = self._console.is_terminal
        self._progress: Progress | None = None
        self._stage_task_id: TaskID | None = None
        self._stage_total: int | None = None
        self._stage_completed = 0
        self._stage_unit = "items"
        self._stage_log_interval = 1
        self._stage_next_log = 1
        self._stage_last_reported = 0
        self._upload_total_blobs = 0
        self._upload_completed_blobs = 0
        self._upload_already_present = 0

    def display_data_change_summary(
        self, *, additions: int, file_removals: int, directory_removals: int
    ) -> None:
        """Print the operation summary before work begins."""

        self._log("Dataset changes")
        self._log(f"  Additions: {additions} files")
        self._log(
            f"  Removals: {file_removals + directory_removals} "
            f"({file_removals} files, {directory_removals} directories)"
        )

    def log_ignored_file(self, path: str) -> None:
        """Report a file skipped by an ignore pattern."""

        self._log(f"  Ignoring file: {path} (matched ignore pattern)", style="yellow")

    def log_ignored_dir(self, path: str) -> None:
        """Report a directory skipped by an ignore pattern."""

        self._log(
            f"  Ignoring directory: {path} (matched ignore pattern)", style="yellow"
        )

    @contextmanager
    def stage(
        self, label: str, *, total: int | None = None, unit: str = "items"
    ) -> Generator[None, None, None]:
        """Show a spinner while a stage runs, then mark its outcome."""

        succeeded = False
        self._stage_total = total
        self._stage_completed = 0
        self._stage_unit = unit
        self._stage_log_interval = max(1, ((total or 1) + 19) // 20)
        self._stage_next_log = self._stage_log_interval
        self._stage_last_reported = 0
        progress: Progress | None = None
        if self._rich:
            progress = Progress(
                SpinnerColumn(),
                TextColumn("{task.description}", markup=False),
                BarColumn(bar_width=18),
                _TaskProgressColumn(binary_units=False),
                _TaskSpeedColumn(),
                console=self._console,
                transient=False,
            )
            progress.start()
            self._progress = progress
            self._stage_task_id = progress.add_task(label, total=total, unit=unit)
        else:
            self._log(f"▶ {label}")
        try:
            yield
            if (
                not self._rich
                and self._stage_total is not None
                and self._stage_completed != self._stage_last_reported
            ):
                self._report_stage_progress()
            succeeded = True
        except BaseException:
            self._log(f"✗ {label}", style="red")
            raise
        finally:
            if progress is not None:
                progress.stop()
            self._progress = None
            self._stage_task_id = None
            self._stage_total = None
        if succeeded:
            self._log(f"✓ {label}", style="green")

    def advance_stage(self, amount: int = 1) -> None:
        """Advance a counted stage and report occasional plain-text updates."""

        self._stage_completed += amount
        if self._progress is not None and self._stage_task_id is not None:
            self._progress.advance(self._stage_task_id, amount)
        elif (
            self._stage_total is not None
            and self._stage_completed >= self._stage_next_log
        ):
            self._report_stage_progress()
            self._stage_next_log += self._stage_log_interval

    def configure_upload(
        self,
        *,
        total_blobs: int,
        upload_blobs: int,
    ) -> None:
        """Turn the active upload stage into aggregate blob progress."""

        self._upload_total_blobs = total_blobs
        self._upload_completed_blobs = 0
        self._upload_already_present = total_blobs - upload_blobs
        self._stage_total = total_blobs
        self._stage_completed = self._upload_already_present
        self._stage_unit = "blobs"
        if self._progress is None or self._stage_task_id is None:
            if not self._rich:
                if upload_blobs:
                    self._stage_log_interval = max(1, (upload_blobs + 19) // 20)
                    self._stage_next_log = self._stage_log_interval
                    self._log(f"  {self._upload_description()}")
                else:
                    self._log(
                        "  Upload data · "
                        f"{'all blobs already present' if total_blobs else 'no blobs to upload'}"
                    )
            return

        if upload_blobs:
            self._progress.update(
                self._stage_task_id,
                description=self._upload_description(),
                total=total_blobs,
                completed=self._upload_already_present,
                unit="blobs",
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
                unit="blobs",
            )

    def configure_upload_batches(self, total_blobs: int) -> None:
        """Initialize blob progress for uploads prepared in batches."""

        self._upload_total_blobs = total_blobs
        self._upload_completed_blobs = 0
        self._upload_already_present = 0
        self._stage_total = total_blobs
        self._stage_unit = "blobs"
        self._stage_completed = 0
        self._stage_log_interval = max(1, (total_blobs + 19) // 20)
        self._stage_next_log = self._stage_log_interval
        if self._progress is not None and self._stage_task_id is not None:
            self._progress.update(
                self._stage_task_id,
                description=(
                    self._upload_description()
                    if total_blobs
                    else "Upload data · no blobs to upload"
                ),
                total=total_blobs or None,
                completed=0,
                unit="blobs",
            )

    def record_uploads_already_present(self, count: int) -> None:
        """Count blobs that the server already has and did not request."""

        self._upload_already_present += count
        if count:
            self.advance_stage(count)
        if self._progress is not None and self._stage_task_id is not None:
            self._progress.update(
                self._stage_task_id,
                description=self._upload_description(),
            )
        elif count:
            self._log(f"  {self._upload_already_present} blobs already present")

    @contextmanager
    def upload_file(
        self, path: str, size: int
    ) -> Generator[Callable[[int], None], None, None]:
        """Display an indented transfer row and yield its byte progress callback."""

        if self._progress is None:
            try:
                yield lambda _: None
            except BaseException:
                raise
            else:
                self._upload_completed_blobs += 1
                self.advance_stage()
                self._log(f"  ✓ {path}", style="green")
            return

        task_id = self._progress.add_task(f"  {path}", total=size)
        try:
            yield lambda amount: self._advance_upload(task_id, amount)
        except BaseException:
            raise
        else:
            self._upload_completed_blobs += 1
            self.advance_stage()
            if self._stage_task_id is not None:
                self._progress.update(
                    self._stage_task_id,
                    description=self._upload_description(),
                )
            self._log(f"  ✓ {path}", style="green")
        finally:
            self._progress.remove_task(task_id)

    def _advance_upload(self, task_id: TaskID, amount: int) -> None:
        if self._progress is None:
            return
        self._progress.advance(task_id, amount)

    def _upload_description(self) -> str:
        if not self._upload_total_blobs:
            return "Upload data · no blobs to upload"
        resolved_blobs = self._upload_completed_blobs + self._upload_already_present
        if (
            resolved_blobs == self._upload_total_blobs
            and not self._upload_completed_blobs
        ):
            return f"Upload data · all {self._upload_total_blobs} blobs already present"
        description = "Upload data"
        if self._upload_already_present:
            description += f" · {self._upload_already_present} already present"
        return description

    def _log(self, message: str, *, style: str | None = None) -> None:
        self._console.print(message, style=style)
        self._console.file.flush()

    def _report_stage_progress(self) -> None:
        if self._stage_total is None:
            return
        completed = min(self._stage_completed, self._stage_total)
        self._log(f"  {completed}/{self._stage_total} {self._stage_unit}")
        self._stage_last_reported = completed
