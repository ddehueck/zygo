"""Terminal status for a cloud training run."""

from __future__ import annotations

from collections import deque
from contextlib import contextmanager
from pathlib import Path
import time
from typing import TYPE_CHECKING, assert_never, override

from rich.console import Console, Group
from rich.live import Live
from rich.spinner import Spinner
from rich.text import Text

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Iterator, Sequence

    from rich.console import RenderableType

    from zygo.cloud.train.api import ModelTrainingRun, RunLogLine

_BYTES_PER_MB = 1_000_000
_POLL_SECONDS = 2.0
_LOG_LINES = 20
_PLAIN_WIDTH = 120
_FINAL_STATUSES = frozenset({"succeeded", "cancelled", "failed"})
_BUILD_PHASES = frozenset({
    "BUILD",
    "DOWNLOAD_SOURCE",
    "FINALIZING",
    "INSTALL",
    "POST_BUILD",
    "PRE_BUILD",
    "PROVISIONING",
    "QUEUED",
    "SUBMITTED",
    "UPLOAD_ARTIFACTS",
})


def show_excluded(console: Console, paths: Sequence[str]) -> None:
    """Print each path left out, indented under the packed check."""
    for relative in paths:
        console.print(f"  [dim]excluding[/dim] {relative}")


def format_archive_size(size_bytes: int) -> str:
    """Format a packed archive as decimal megabytes, such as ``1.5 MB``."""
    return f"{size_bytes / _BYTES_PER_MB:.1f} MB"


def format_source_dir(directory: Path) -> str:
    """Format a source directory relative to the working directory.

    The path keeps at least one folder name, such as ``./src/`` or
    ``./something/code/``.
    """
    cwd = Path.cwd().resolve()
    resolved = directory.resolve()
    if not resolved.name:
        return f"{resolved.as_posix()}/"
    if resolved == cwd:
        return f"./{resolved.name}/"
    relative = resolved.relative_to(cwd, walk_up=True)
    if any(part not in {".", ".."} for part in relative.parts):
        text = relative.as_posix()
        if text.startswith(".."):
            return f"{text}/"
        return f"./{text}/"
    ups = sum(part == ".." for part in relative.parts)
    climb = "/".join([".."] * (ups + 1))
    return f"{climb}/{resolved.name}/"


def make_console() -> Console:
    """Console used for training progress.

    A terminal that can redraw keeps that terminal's width. Plain stdout
    and stderr use 120 columns so log lines are not wrapped at 80.
    """
    return _prepare_console(Console())


def _prepare_console(console: Console) -> Console:
    """Widen a console that cannot redraw. Leave a live terminal alone."""
    if not _can_redraw(console):
        _use_plain_width(console)
    return console


def _use_plain_width(console: Console) -> None:
    """Fix the line width when this console only prints, and cannot redraw.

    Both dimensions have to be set. A dumb terminal otherwise reports 80
    columns even after the width is changed on its own.
    """
    console.size = (_PLAIN_WIDTH, console.size.height)


def _can_redraw(console: Console) -> bool:
    """True when this console can redraw a live region in place.

    The stream's own ``isatty`` decides this. ``FORCE_COLOR`` and
    ``TTY_INTERACTIVE`` can mark a console interactive when it still cannot
    move the cursor. A dumb terminal reports a tty and still cannot redraw.
    """
    if console.is_dumb_terminal:
        return False
    isatty = getattr(console.file, "isatty", None)
    if not callable(isatty):
        return False
    try:
        return bool(isatty())
    except (ValueError, OSError):
        return False


class Progress:
    """A pending step at the bottom of the screen.

    Finished steps and their metadata are printed above it and stay there.
    Build logs belong to the pending step, so they leave with it. A step
    that keeps its logs prints each line above the spinner, and those lines
    stay after the step finishes.

    When the console cannot redraw in place, each step is printed as a plain
    line and every log line is printed as it arrives.
    """

    def __init__(
        self, console: Console, spinner: _StepSpinner, live: Live | None = None
    ) -> None:
        super().__init__()
        self._console = console
        self._live = live
        self._spinner = spinner
        self._tail: _LogTail | None = None
        self._keep_logs = False
        self._kept_line = False
        self._status_line: str | None = None

    def update(self, message: str) -> None:
        """Start the next step, resetting its elapsed time."""
        self._tail = None
        self._keep_logs = False
        self._kept_line = False
        self._spinner.reset(message)
        if self._live is None:
            self._write_status(message)
            return
        self._live.update(self._spinner, refresh=True)

    def keep_logs(self) -> None:
        """Print later log lines above the spinner so they stay on screen."""
        self._keep_logs = True
        self._tail = None

    def complete(self, message: str) -> None:
        """Leave a green check for a step that finished."""
        self._console.print(f"[green]✓[/green] {message}")

    def watch(self, message: str) -> None:
        """Show this step with its log window underneath."""
        if self._live is None:
            self._write_status(message)
            return
        if self._keep_logs:
            self._show_kept(message)
            return
        if self._tail is None:
            self._tail = _LogTail(message)
        else:
            self._tail.set_status(message)
        self._live.update(self._tail.render(), refresh=True)

    def add_log(self, line: RunLogLine) -> None:
        """Append one log line to the pending step."""
        if self._live is None or self._keep_logs:
            self._print_kept(line)
            self._kept_line = True
            if self._live is not None:
                self._live.update(self._spinner, refresh=True)
            return
        if self._tail is None:
            self._tail = _LogTail(self._spinner.label)
        self._tail.add(line)
        self._live.update(self._tail.render(), refresh=True)

    def _write_status(self, message: str) -> None:
        """Print a step label once, when the console cannot redraw it."""
        if message == self._status_line:
            return
        self._status_line = message
        self._console.print(message)

    def _show_kept(self, message: str) -> None:
        """Keep the spinner under any lines already printed for this step."""
        live = self._live
        if live is None:
            return
        if self._spinner.label != message:
            self._spinner.reset(message)
        if self._kept_line:
            live.update(self._spinner, refresh=True)
            return
        waiting = Text("waiting for logs...", style="dim")
        live.update(Group(self._spinner, waiting), refresh=True)

    def _print_kept(self, line: RunLogLine) -> None:
        """Print one log line so it stays on screen."""
        style = "red" if line.stream == "stderr" else ""
        text = line.line if line.line else " "
        self._console.print(Text(text, style=style))

    def settle(self) -> Console:
        """Remove the pending step and its logs, and return the console."""
        self._tail = None
        if self._live is not None:
            self._live.update(Text(""), refresh=True)
        return self._console


@contextmanager
def progress(console: Console, message: str) -> Generator[Progress, None, None]:
    """Show the current step until the caller leaves this block.

    The label counts how long that step has been running. A finished step
    is printed above the spinner and remains after this block ends. The
    spinner and any log window do not.

    When the console cannot redraw in place, the label and each log line
    are printed and stay in the console.
    """
    if not _can_redraw(console):
        step = Progress(console, _StepSpinner(message))
        step.update(message)
        yield step
        return
    spinner = _StepSpinner(message)
    with Live(
        spinner, console=console, refresh_per_second=12.5, transient=True
    ) as live:
        yield Progress(console, spinner, live)


def follow_run(  # ruff: ignore[too-many-arguments]
    run: ModelTrainingRun,
    refresh: Callable[[str], ModelTrainingRun],
    build_logs: Callable[[str], Iterator[RunLogLine]],
    run_logs: Callable[[str], Iterator[RunLogLine]],
    console: Console,
    *,
    step: Progress | None = None,
) -> ModelTrainingRun:
    """Follow build logs, then training logs, until the run settles.

    While the status is ``building``, build logs stream under the image
    step. Until the first line arrives, a muted waiting message is shown
    under the spinner. A stream that ends is followed again until the
    status changes. Lines already shown are skipped when a stream is
    reopened. Lines that share a sequence are kept, because one stored
    batch uses one sequence. When the console cannot redraw in place, those
    build lines are printed as they arrive and stay in the console.

    When the image is ready, the log window clears and the image check
    stays. Training logs then stream under a new step labeled with the run
    id. Each training line is printed as it arrives and stays on screen.
    That step continues until the run succeeds, fails, or is cancelled.
    The outcome is printed under those lines and includes the run id. It
    is not raised.
    """
    if step is not None:
        return _follow_run(run, refresh, build_logs, run_logs, step)
    if run.status in {"failed", "cancelled"}:
        show_outcome(run, console)
        return run
    with progress(console, _opening_label(run)) as owned:
        return _follow_run(run, refresh, build_logs, run_logs, owned)


def _follow_run(
    run: ModelTrainingRun,
    refresh: Callable[[str], ModelTrainingRun],
    build_logs: Callable[[str], Iterator[RunLogLine]],
    run_logs: Callable[[str], Iterator[RunLogLine]],
    step: Progress,
) -> ModelTrainingRun:
    if run.status == "building":
        run = _follow_build(run, refresh, build_logs, step)
    if run.status in {"failed", "cancelled"}:
        _finish(run, step)
        return run
    _mark_image_built(step)
    finished = _follow_training(run, refresh, run_logs, step)
    _finish(finished, step)
    return finished


def _follow_build(
    run: ModelTrainingRun,
    refresh: Callable[[str], ModelTrainingRun],
    logs: Callable[[str], Iterator[RunLogLine]],
    step: Progress,
) -> ModelTrainingRun:
    seen: dict[tuple[str, int], int] = {}
    while run.status == "building":
        step.watch("Building image")
        _collect_logs(logs(run.id), seen, step)
        run = refresh(run.id)
        if run.status == "building":
            time.sleep(_POLL_SECONDS)
    return run


def _follow_training(
    run: ModelTrainingRun,
    refresh: Callable[[str], ModelTrainingRun],
    logs: Callable[[str], Iterator[RunLogLine]],
    step: Progress,
) -> ModelTrainingRun:
    label = _training_label(run.id)
    step.update(label)
    step.keep_logs()
    seen: dict[tuple[str, int], int] = {}
    while True:
        step.watch(label)
        _collect_logs(logs(run.id), seen, step)
        if run.status in _FINAL_STATUSES:
            return run
        run = refresh(run.id)
        if run.status in _FINAL_STATUSES:
            return run
        time.sleep(_POLL_SECONDS)


def _mark_image_built(step: Progress) -> None:
    """Clear the build log window and leave the image check."""
    step.settle()
    step.complete("Image built")


def _finish(run: ModelTrainingRun, step: Progress) -> None:
    """Clear the log window and print the outcome that should remain."""
    show_outcome(run, step.settle())


class _LogTail:
    """A status spinner above the latest log lines."""

    def __init__(self, status: str) -> None:
        super().__init__()
        self._lines: deque[Text] = deque(maxlen=_LOG_LINES)
        self._status = _spinner(status)

    def set_status(self, status: str) -> None:
        """Show a spinner for the current phase, keeping its clock if it matches."""
        current = self._status
        if isinstance(current, _StepSpinner) and current.label == status:
            return
        self._status = _spinner(status)

    def add(self, line: RunLogLine) -> None:
        """Keep this line, dropping the oldest once the window is full."""
        style = "red" if line.stream == "stderr" else ""
        text = line.line if line.line else " "
        self._lines.append(Text(text, style=style, no_wrap=True, overflow="ellipsis"))

    def render(self) -> Group:
        """The status line above the latest logs, or a waiting message."""
        if self._lines:
            return Group(self._status, *self._lines)
        return Group(self._status, Text("waiting for logs...", style="dim"))


class _StepSpinner(Spinner):
    """A step label with a clock that counts in seconds, then minutes."""

    def __init__(self, label: str) -> None:
        super().__init__("dots", style="status.spinner")
        self.label = label
        self._started = time.monotonic()

    def reset(self, label: str) -> None:
        """Start a new step at zero seconds."""
        self.label = label
        self._started = time.monotonic()

    @override
    def render(self, time: float) -> RenderableType:
        elapsed = _format_elapsed(_seconds_since(self._started))
        self.text = Text.assemble(self.label, " ", (elapsed, "dim"))
        return super().render(time)


def _seconds_since(started: float) -> float:
    return time.monotonic() - started


def _format_elapsed(seconds: float) -> str:
    """Format a short duration as seconds, or minutes and seconds."""
    whole = max(0, int(seconds))
    minutes, secs = divmod(whole, 60)
    if minutes == 0:
        return f"{secs}s"
    return f"{minutes}m {secs}s"


def _spinner(status: str) -> _StepSpinner:
    return _StepSpinner(status)


def _opening_label(run: ModelTrainingRun) -> str:
    if run.status == "building":
        return "Building image"
    return _training_label(run.id)


def _training_label(run_id: str) -> str:
    return f"Training ({run_id})"


def _collect_logs(
    lines: Iterator[RunLogLine],
    seen: dict[tuple[str, int], int],
    step: Progress,
) -> None:
    """Show log lines, skipping a prefix already shown on an earlier stream.

    Lines from one batch share a sequence, so a sequence can appear more
    than once. The count of lines already shown for that stream and
    sequence is what gets skipped when the stream is reopened.
    """
    try:
        _record_unseen_logs(lines, seen, step)
    except TimeoutError:
        return


def _record_unseen_logs(
    lines: Iterator[RunLogLine],
    seen: dict[tuple[str, int], int],
    step: Progress,
) -> None:
    index_in_stream: dict[tuple[str, int], int] = {}
    for line in lines:
        key = (line.stream, line.sequence)
        index = index_in_stream.get(key, 0)
        index_in_stream[key] = index + 1
        if index < seen.get(key, 0):
            continue
        seen[key] = index + 1
        step.add_log(line)


def show_outcome(run: ModelTrainingRun, console: Console) -> None:
    """Print a single line for a run that is no longer building."""
    outcome = _outcome_text(run)
    if outcome is not None:
        console.print(outcome)


def _outcome_text(run: ModelTrainingRun) -> Text | None:
    """The status line for a run that is no longer building."""
    match run.status:
        case "building":
            return None
        case "failed":
            detail = _with_run_id(failure_message(run.message), run.id)
            markup = f"[red]✗[/red] {detail}"
        case "cancelled":
            markup = f"[yellow]✗[/yellow] Training was cancelled ({run.id})"
        case "succeeded":
            markup = f"[green]✓[/green] Training succeeded ({run.id})"
        case "running":
            markup = f"[green]✓[/green] {_training_label(run.id)}"
        case unreachable:
            assert_never(unreachable)
    return Text.from_markup(markup)


def _with_run_id(message: str, run_id: str) -> str:
    text = message[:-1] if message.endswith(".") else message
    return f"{text} ({run_id})"


def failure_message(message: str | None) -> str:
    """Explain a failed run.

    The server stores a failed image build as the CodeBuild phase that
    failed, such as ``BUILD``. Other failures already include a sentence.
    """
    text = (message or "").strip()
    phases = [part.strip() for part in text.split(",") if part.strip()]
    if not phases:
        return "Training run failed."
    if not all(part in _BUILD_PHASES for part in phases):
        return text if text.endswith(".") else f"Training run failed: {text}"
    if len(phases) == 1:
        return f"The image build failed in the {phases[0]} phase."
    return f"The image build failed in these phases: {', '.join(phases)}."
