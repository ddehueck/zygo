from io import BytesIO, StringIO
from pathlib import Path
import sys
import time
from typing import Literal, override
from urllib.request import Request
from zipfile import ZipFile

import pytest
from rich.console import Console

from zygo import Dataset, HyperParams, Model, TrainingContext
from zygo.cloud.train import client as train_client, train
from zygo.cloud.train.api import (
    DeclareSourceRequest,
    ImageDefinition,
    ModelTrainingRun,
    RunLogLine,
    StartModelTrainingRunRequest,
    run_build_logs_url,
    run_logs_url,
    run_status_url,
)
from zygo.cloud.train.client import TrainingClient
from zygo.cloud.train.display import (
    Progress,
    _collect_logs,
    _format_elapsed,
    _LogTail,
    _prepare_console,
    _StepSpinner,
    follow_run,
    format_archive_size,
    make_console,
    progress,
)


class _Params(HyperParams):
    epochs: int = 10
    seed: int = 1


def _model(name: str, *, with_params: bool) -> Model:
    model = Model(name)
    if with_params:

        @model.train
        def train_with_params(
            dataset: Dataset[object], *, params: _Params, ctx: TrainingContext
        ) -> None:
            del dataset, params, ctx

    else:

        @model.train
        def train_without_params(
            dataset: Dataset[object], *, ctx: TrainingContext
        ) -> None:
            del dataset, ctx

    return model


def _capture_archive(monkeypatch: pytest.MonkeyPatch) -> list[bytes]:
    archives: list[bytes] = []

    def ensure_source(self: TrainingClient, archive: bytes, source_hash: str) -> str:
        del self, source_hash
        archives.append(archive)
        return "src_1"

    def start_run(
        self: TrainingClient, request: StartModelTrainingRunRequest
    ) -> ModelTrainingRun:
        del self, request
        return ModelTrainingRun(id="trn_1", status="running")

    monkeypatch.setattr(TrainingClient, "ensure_source", ensure_source)
    monkeypatch.setattr(TrainingClient, "start_run", start_run)
    _stub_idle_logs(monkeypatch)
    return archives


def _capture_request(
    monkeypatch: pytest.MonkeyPatch,
) -> list[dict[str, object]]:
    posted: list[dict[str, object]] = []

    def ensure_source(self: TrainingClient, archive: bytes, source_hash: str) -> str:
        del self, archive, source_hash
        return "src_1"

    def start_run(
        self: TrainingClient, request: StartModelTrainingRunRequest
    ) -> ModelTrainingRun:
        del self
        posted.append(request.model_dump(mode="json"))
        return ModelTrainingRun(id="trn_1", status="running")

    monkeypatch.setattr(TrainingClient, "ensure_source", ensure_source)
    monkeypatch.setattr(TrainingClient, "start_run", start_run)
    _stub_idle_logs(monkeypatch)
    return posted


def test_train_sends_validated_hyperparameters(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _build_context(tmp_path, monkeypatch)
    posted = _capture_request(monkeypatch)

    started = train(
        _model("classifier", with_params=True),
        "dsv_ready",
        {"epochs": 5},
        api_key="test-key",
        target="main:app",
    )

    assert started.id == "trn_1"
    assert posted[0]["params"] == {"epochs": 5, "seed": 1}
    assert posted[0]["image"] == {
        "dockerfile": "Dockerfile",
        "pyproject": "pyproject.toml",
        "uv_lock": "uv.lock",
    }


def test_train_sends_hyperparameter_defaults(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _build_context(tmp_path, monkeypatch)
    posted = _capture_request(monkeypatch)

    train(
        _model("classifier", with_params=True),
        "dsv_ready",
        api_key="test-key",
        target="main:app",
    )

    assert posted[0]["params"] == {"epochs": 10, "seed": 1}


def test_train_packs_source_relative_to_the_calling_file(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    project = tmp_path / "project"
    caller_dir = project / "model"
    caller_dir.mkdir(parents=True)
    _write_image_files(project)
    (project / "app.py").write_text("print('train')\n")
    elsewhere = tmp_path / "elsewhere"
    elsewhere.mkdir()
    (elsewhere / "Dockerfile").write_text("FROM decoy\n")
    (elsewhere / "decoy.py").write_text("print('decoy')\n")
    monkeypatch.chdir(elsewhere)
    archives = _capture_archive(monkeypatch)

    code = compile(
        "train(model, 'dsv_ready', api_key='test-key', source='..', target='main:app')\n",
        str(caller_dir / "cloud.py"),
        "exec",
    )
    exec(code, {"train": train, "model": _model("classifier", with_params=False)})  # ruff: ignore[exec-builtin]

    with ZipFile(BytesIO(archives[0])) as archive:
        assert archive.namelist() == [
            "Dockerfile",
            "app.py",
            "pyproject.toml",
            "uv.lock",
        ]
        assert archive.read("Dockerfile") == b"FROM scratch\n"


def test_train_exclude_omits_matching_files(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _write_image_files(tmp_path)
    (tmp_path / "app.py").write_text("print('train')\n")
    (tmp_path / "notes.log").write_text("log\n")
    (tmp_path / "secret.pem").write_text("pem\n")
    monkeypatch.chdir(tmp_path)
    archives = _capture_archive(monkeypatch)

    train(
        _model("classifier", with_params=False),
        "dsv_ready",
        api_key="test-key",
        source=tmp_path,
        exclude=["*.log", "*.pem"],
        target="main:app",
    )

    with ZipFile(BytesIO(archives[0])) as archive:
        assert archive.namelist() == [
            "Dockerfile",
            "app.py",
            "pyproject.toml",
            "uv.lock",
        ]


def test_train_shows_excluded_files_while_packing(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _write_image_files(tmp_path)
    (tmp_path / "app.py").write_text("print('train')\n")
    (tmp_path / "notes.log").write_text("log\n")
    nested = tmp_path / "build"
    nested.mkdir()
    (nested / "output.bin").write_bytes(b"bin")
    monkeypatch.chdir(tmp_path)
    archives = _capture_archive(monkeypatch)
    console, output = _console()
    monkeypatch.setattr(
        sys.modules["zygo.cloud.train.run"], "make_console", lambda: console
    )

    train(
        _model("classifier", with_params=False),
        "dsv_ready",
        api_key="test-key",
        source=tmp_path,
        exclude=["*.log", "build/"],
        target="main:app",
    )

    text = output.getvalue()
    packed = f"Packed source code from ./{tmp_path.name}/"
    uploaded = f"Uploaded {format_archive_size(len(archives[0]))} of source code"
    assert text.count(packed) == 1
    assert text.index(packed) < text.index("  excluding build/")
    assert text.index("  excluding notes.log") < text.index(uploaded)
    assert text.index(uploaded) < text.index("Image built")
    assert text.index("Image built") < text.index("Training succeeded (trn_1)")
    assert "  excluding build/" in text
    assert "  excluding notes.log" in text
    assert "excluding app.py" not in text
    assert "excluding Dockerfile" not in text
    assert "log-" not in text


def test_train_sends_empty_params_without_hyperparameters(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _build_context(tmp_path, monkeypatch)
    posted = _capture_request(monkeypatch)

    train(
        _model("classifier", with_params=False),
        "dsv_ready",
        api_key="test-key",
        target="main:app",
    )

    assert posted[0]["params"] == {}


def test_run_routes() -> None:
    assert DeclareSourceRequest.url().endswith("/api/v1/source")
    assert StartModelTrainingRunRequest.url().endswith("/api/v1/runs")
    assert run_status_url("trn/1").endswith("/api/v1/runs/trn%2F1/status")
    assert run_logs_url("trn_1").endswith("/api/v1/runs/trn_1/logs")
    assert run_build_logs_url("trn/1").endswith("/api/v1/runs/trn%2F1/build/logs")


def test_make_console_uses_120_columns_for_plain_output(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    buffer = StringIO()
    monkeypatch.setattr(sys, "stdout", buffer)
    console = make_console()
    line = "m" * 100

    console.print(line, highlight=False)

    assert console.width == 120
    assert buffer.getvalue().splitlines() == [line]


def test_prepare_console_keeps_the_width_of_a_terminal_that_can_redraw() -> None:
    console = Console(
        file=_TTY(),
        force_terminal=True,
        no_color=True,
        highlight=False,
        width=80,
        height=24,
        _environ={"TERM": "xterm-256color"},
    )

    prepared = _prepare_console(console)

    assert prepared.width == 80


def test_prepare_console_widens_a_dumb_terminal() -> None:
    console = Console(
        file=_TTY(),
        no_color=True,
        highlight=False,
        width=40,
        height=10,
        _environ={"TERM": "dumb"},
    )

    prepared = _prepare_console(console)

    assert prepared.width == 120
    assert prepared.size.height == 10


def test_follow_run_prints_logs_when_the_console_cannot_redraw(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(time, "sleep", lambda _seconds: None)
    forced = Console(
        file=StringIO(),
        force_terminal=True,
        no_color=True,
        highlight=False,
        width=80,
    )
    dumb = Console(
        file=_TTY(),
        no_color=True,
        highlight=False,
        width=80,
        _environ={"TERM": "dumb"},
    )

    for console in (forced, dumb):
        finished = follow_run(
            ModelTrainingRun(id="trn_9", status="building"),
            lambda _run_id: ModelTrainingRun(id="trn_9", status="succeeded"),
            lambda _run_id: iter((_log("stdout", 0, "build-line"),)),
            lambda _run_id: iter((_log("stdout", 0, "epoch 1"),)),
            console,
        )
        output = console.file
        assert isinstance(output, StringIO)
        text = output.getvalue()
        assert finished.status == "succeeded"
        assert "\x1b" not in text
        assert text.index("Building image") < text.index("build-line")
        assert text.index("build-line") < text.index("Image built")
        assert text.index("Image built") < text.index("epoch 1")


def test_follow_run_shows_a_failed_build_without_raising(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(time, "sleep", lambda _seconds: None)
    console, output = _console()

    finished = follow_run(
        ModelTrainingRun(id="trn_9", status="failed", message="BUILD"),
        lambda _run_id: pytest.fail("status should not be polled after failure"),
        lambda _run_id: pytest.fail("build logs should not stream after failure"),
        lambda _run_id: pytest.fail("training logs should not stream after failure"),
        console,
    )

    assert finished.status == "failed"
    text = output.getvalue()
    assert "The image build failed in the BUILD phase (trn_9)" in text
    assert "Image built" not in text


def test_follow_run_streams_build_logs_then_training_logs(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    statuses = iter((
        ModelTrainingRun(id="trn_9", status="building"),
        ModelTrainingRun(id="trn_9", status="running"),
        ModelTrainingRun(id="trn_9", status="succeeded"),
    ))
    build_streams = [
        [_log("stdout", 0, "step 1"), _log("stderr", 1, "step 2")],
        [_log("stdout", 0, "step 1"), _log("stdout", 2, "step 3")],
    ]
    training_streams = [
        [_log("stdout", 0, "epoch 1"), _log("stdout", 0, "epoch 2")],
    ]
    labels: list[str] = []
    shown: list[str] = []
    seen: list[str] = []
    monkeypatch.setattr(time, "sleep", lambda _seconds: None)
    monkeypatch.setattr(Progress, "watch", _recording_watch(labels))
    monkeypatch.setattr(Progress, "add_log", _recording_add_log(shown))
    console, output = _console()

    def refresh(run_id: str) -> ModelTrainingRun:
        seen.append(run_id)
        return next(statuses)

    finished = follow_run(
        ModelTrainingRun(id="trn_9", status="building"),
        refresh,
        lambda _run_id: iter(build_streams.pop(0)),
        lambda _run_id: iter(training_streams.pop(0)),
        console,
    )

    text = output.getvalue()
    assert finished.status == "succeeded"
    assert seen == ["trn_9", "trn_9", "trn_9"]
    assert labels == ["Building image", "Building image", "Training (trn_9)"]
    assert shown == ["step 1", "step 2", "step 3", "epoch 1", "epoch 2"]
    assert text.count("step 1") == 1
    assert text.index("Building image") < text.index("step 1")
    assert text.index("step 1") < text.index("step 2")
    assert text.index("step 2") < text.index("step 3")
    assert text.index("step 3") < text.index("Image built")
    assert text.index("Image built") < text.index("Training (trn_9)")
    assert text.index("Training (trn_9)") < text.index("epoch 1")
    assert text.index("epoch 1") < text.index("epoch 2")
    assert text.index("epoch 2") < text.index("Training succeeded (trn_9)")
    assert "Training started" not in text


def test_follow_run_keeps_logs_when_the_console_cannot_redraw(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    build_lines = [_log("stdout", index, f"build-{index:02d}") for index in range(25)]
    run_lines = [_log("stdout", 0, f"train-{index:02d}") for index in range(25)]
    statuses = iter((
        ModelTrainingRun(id="trn_9", status="running"),
        ModelTrainingRun(id="trn_9", status="succeeded"),
    ))
    monkeypatch.setattr(time, "sleep", lambda _seconds: None)
    console, output = _console()

    finished = follow_run(
        ModelTrainingRun(id="trn_9", status="building"),
        lambda _run_id: next(statuses),
        lambda _run_id: iter(build_lines),
        lambda _run_id: iter(run_lines),
        console,
    )

    text = output.getvalue()
    assert finished.status == "succeeded"
    assert text.index("build-00") < text.index("build-05")
    assert text.index("build-05") < text.index("build-24")
    assert text.index("build-24") < text.index("Image built")
    assert text.index("Image built") < text.index("train-00")
    assert text.index("train-00") < text.index("train-24")
    assert text.index("train-24") < text.index("Training succeeded (trn_9)")


def test_follow_run_reports_a_failed_training_run_with_its_id(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(time, "sleep", lambda _seconds: None)
    console, output = _console()

    finished = follow_run(
        ModelTrainingRun(id="trn_9", status="running"),
        lambda _run_id: ModelTrainingRun(
            id="trn_9", status="failed", message="out of memory"
        ),
        lambda _run_id: pytest.fail(
            "build logs should not stream once training starts"
        ),
        lambda _run_id: iter((_log("stderr", 0, "boom"),)),
        console,
    )

    assert finished.status == "failed"
    text = output.getvalue()
    assert text.index("Image built") < text.index("boom")
    assert text.index("boom") < text.index("Training run failed: out of memory (trn_9)")


def test_follow_run_reports_a_cancelled_build_without_an_image_check(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(time, "sleep", lambda _seconds: None)
    console, output = _console()

    finished = follow_run(
        ModelTrainingRun(id="trn_9", status="building"),
        lambda _run_id: ModelTrainingRun(id="trn_9", status="cancelled"),
        lambda _run_id: iter(()),
        lambda _run_id: pytest.fail("training logs should not stream"),
        console,
    )

    assert finished.status == "cancelled"
    text = output.getvalue()
    assert "Image built" not in text
    assert "Training was cancelled (trn_9)" in text


def test_collect_logs_keeps_lines_that_share_a_sequence(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    shown: list[str] = []
    seen: dict[tuple[str, int], int] = {}
    monkeypatch.setattr(Progress, "add_log", _recording_add_log(shown))
    console, _output = _console()

    with progress(console, "Training") as step:
        _collect_logs(
            iter((
                _log("stdout", 0, "one"),
                _log("stdout", 0, "two"),
                _log("stderr", 0, "err"),
            )),
            seen,
            step,
        )
        _collect_logs(
            iter((
                _log("stdout", 0, "one"),
                _log("stdout", 0, "two"),
                _log("stdout", 0, "three"),
            )),
            seen,
            step,
        )
        _collect_logs(_timeout_after(_log("stdout", 1, "kept")), seen, step)

    assert shown == ["one", "two", "err", "three", "kept"]


def test_log_tail_shows_the_spinner_above_the_latest_lines() -> None:
    tail = _LogTail("Building image")
    for index in range(25):
        tail.add(_log("stderr" if index == 24 else "stdout", index, f"log-{index:02d}"))
    console, output = _console()

    console.print(tail.render())

    text = output.getvalue()
    assert "log-04" not in text
    assert "waiting for logs..." not in text
    assert text.index("Building image") < text.index("log-05") < text.index("log-24")


def test_archive_size_is_shown_in_decimal_megabytes() -> None:
    assert format_archive_size(1_500_000) == "1.5 MB"
    assert format_archive_size(0) == "0.0 MB"


def test_elapsed_time_counts_seconds_then_minutes() -> None:
    assert _format_elapsed(0) == "0s"
    assert _format_elapsed(59.9) == "59s"
    assert _format_elapsed(60) == "1m 0s"
    assert _format_elapsed(125) == "2m 5s"


def test_step_spinner_counts_up_and_resets_on_the_next_step(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    now = 1_000.0

    def clock() -> float:
        return now

    monkeypatch.setattr(time, "monotonic", clock)
    spinner = _StepSpinner("Packing")
    console, output = _console()
    console.print(spinner)
    assert "Packing" in output.getvalue()
    assert "0s" in output.getvalue()

    now = 1_061.0
    console, later = _console()
    console.print(spinner)
    assert "1m 1s" in later.getvalue()

    spinner.reset("Uploading")
    console, reset = _console()
    console.print(spinner)
    text = reset.getvalue()
    assert "Uploading" in text
    assert "0s" in text
    assert "1m" not in text


def test_log_tail_keeps_the_step_clock_until_the_step_changes(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    now = 50.0
    monkeypatch.setattr(time, "monotonic", lambda: now)
    tail = _LogTail("Building image")
    now = 70.0
    tail.set_status("Building image")
    console, output = _console()
    console.print(tail.render())
    assert "20s" in output.getvalue()

    tail.set_status("Training")
    console, restarted = _console()
    console.print(tail.render())
    text = restarted.getvalue()
    assert "Training" in text
    assert "0s" in text


def test_log_tail_waits_for_logs_under_the_spinner() -> None:
    tail = _LogTail("Building image")
    console, output = _console()

    console.print(tail.render())

    text = output.getvalue()
    assert text.index("Building image") < text.index("waiting for logs...")


def test_start_run_uses_the_run_routes(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sent: list[tuple[str, str, dict[str, object] | None]] = []

    def read_json(
        self: TrainingClient,
        url: str,
        *,
        method: str,
        body: dict[str, object] | None = None,
    ) -> dict[str, object]:
        del self
        sent.append((method, url, body))
        if method == "GET":
            return {"status": "running", "id": "trn_9"}
        return {"status": "building", "id": "trn_9"}

    monkeypatch.setattr(TrainingClient, "_read_json", read_json)
    client = TrainingClient("test-key")

    started = client.start_run(_training_request())
    status = client.training_status(started.id)

    assert started.status == "building"
    assert status.status == "running"
    body = _training_request().model_dump(mode="json")
    assert sent == [
        ("POST", StartModelTrainingRunRequest.url(), body),
        ("GET", run_status_url("trn_9"), None),
    ]


@pytest.mark.parametrize(
    ("method_name", "suffix"),
    [
        ("iter_logs", "/api/v1/runs/trn_9/logs"),
        ("iter_build_logs", "/api/v1/runs/trn_9/build/logs"),
    ],
)
def test_log_streams_parse_server_sent_events(
    monkeypatch: pytest.MonkeyPatch, method_name: str, suffix: str
) -> None:
    raw = "\n".join((
        ": keepalive",
        "",
        'data: {"stream":"stdout","sequence":0,"line":"hello","created_at":"2026-01-01T00:00:00Z"}',
        "",
        "event: log",
        'data: {"stream":"stderr","sequence":1,"line":"warn","created_at":"2026-01-01T00:00:01Z"}',
        "",
        'data: {"stream":"stdout","sequence":2,"line":"tail","created_at":"2026-01-01T00:00:02Z"}',
    )).encode()
    opened: list[Request] = []

    def fake_urlopen(request: Request, timeout: float) -> BytesIO:
        opened.append(request)
        assert timeout == 300
        return BytesIO(raw)

    monkeypatch.setattr(train_client, "urlopen", fake_urlopen)
    lines = list(getattr(TrainingClient("test-key"), method_name)("trn_9"))

    request = opened[0]
    assert request.get_method() == "GET"
    assert request.full_url.endswith(suffix)
    assert request.get_header("Accept") == "text/event-stream"
    assert request.get_header("Authorization") == "Bearer test-key"
    assert request.get_header("Content-type") is None
    assert [line.line for line in lines] == ["hello", "warn", "tail"]
    assert [line.stream for line in lines] == ["stdout", "stderr", "stdout"]


def _stub_idle_logs(monkeypatch: pytest.MonkeyPatch) -> None:
    def iter_logs(self: TrainingClient, run_id: str) -> object:
        del self, run_id
        return iter(())

    def training_status(self: TrainingClient, run_id: str) -> ModelTrainingRun:
        del self
        return ModelTrainingRun(id=run_id, status="succeeded")

    monkeypatch.setattr(TrainingClient, "iter_logs", iter_logs)
    monkeypatch.setattr(TrainingClient, "iter_build_logs", iter_logs)
    monkeypatch.setattr(TrainingClient, "training_status", training_status)


def _recording_watch(labels: list[str]):
    original = Progress.watch

    def watch(self: Progress, message: str) -> None:
        labels.append(message)
        original(self, message)

    return watch


def _recording_add_log(shown: list[str]):
    original = Progress.add_log

    def add_log(self: Progress, line: RunLogLine) -> None:
        shown.append(line.line)
        original(self, line)

    return add_log


def _timeout_after(line: RunLogLine):
    yield line
    raise TimeoutError


def _log(stream: Literal["stdout", "stderr"], sequence: int, line: str) -> RunLogLine:
    return RunLogLine(
        stream=stream,
        sequence=sequence,
        line=line,
        created_at="2026-01-01T00:00:00Z",
    )


def _training_request() -> StartModelTrainingRunRequest:
    return StartModelTrainingRunRequest(
        type="model_training_run",
        source_id="src_1",
        target="main:app",
        model_id="classifier",
        dataset_version_id="dsv_ready",
        image=ImageDefinition(
            dockerfile="Dockerfile",
            pyproject="pyproject.toml",
            uv_lock="uv.lock",
        ),
        params={},
    )


def _console() -> tuple[Console, StringIO]:
    buffer = StringIO()
    console = Console(
        file=buffer, force_terminal=False, no_color=True, highlight=False, width=80
    )
    return console, buffer


class _TTY(StringIO):
    """A buffer that claims to be a terminal."""

    @override
    def isatty(self) -> bool:
        return isinstance(self, _TTY)


_DOCKERFILE = "FROM scratch\n"
_PYPROJECT = "[project]\nname = 'app'\n"
_UV_LOCK = "version = 1\n"


def _write_image_files(root: Path) -> None:
    (root / "Dockerfile").write_text(_DOCKERFILE)
    (root / "pyproject.toml").write_text(_PYPROJECT)
    (root / "uv.lock").write_text(_UV_LOCK)


def _build_context(root: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    _write_image_files(root)
    monkeypatch.chdir(root)
