from dataclasses import dataclass
from pathlib import Path

import pyarrow as pa
import pyarrow.dataset as pads
import pytest

from zygo import Dataset, DataUri, Model, ModelStore, TrainingContext
from zygo.transport import LocalTransport


@dataclass
class _LocalTraining:
    store: ModelStore


def _context(path: Path) -> _LocalTraining:
    return _LocalTraining(
        store=ModelStore(
            root=DataUri(path.as_uri() + "/"),
            ipc_transport=LocalTransport(),
        )
    )


def test_runtime_selects_training_and_loading_roots(tmp_path: Path):
    app = Model("labels")

    @app.train
    def train(dataset: Dataset[object], *, ctx: TrainingContext) -> None:
        del dataset
        ctx.store.put("label.txt", b"7")

    @app.load
    def load(store: ModelStore) -> int:
        return int(store.get("label.txt"))

    @app.infer
    def infer(model: int, data: object) -> int:
        del data
        return model

    dataset = Dataset(pads.dataset(pa.table({"label": [7]})), uri="memory")
    training = _context(tmp_path / "training")
    assert app.run_train(dataset, ctx=training) is None
    assert app.run_load(training.store) == 7

    selected = _context(tmp_path / "selected")
    selected.store.put("label.txt", b"9")
    assert app.run_infer(app.run_load(selected.store), None) == 9


def test_training_rejects_return_annotation():
    app = Model("invalid")
    with pytest.raises(TypeError, match="Training must return None"):

        @app.train  # ty: ignore[invalid-argument-type]
        def train(dataset: Dataset[object], *, ctx: TrainingContext) -> int:
            del dataset, ctx
            return 1


def test_training_rejects_non_none_runtime_result(tmp_path: Path):
    app = Model("invalid-result")

    @app.train
    def train(dataset: Dataset[object], *, ctx: TrainingContext) -> None:
        del dataset, ctx
        return 1  # ty: ignore[invalid-return-type]

    dataset = Dataset(pads.dataset(pa.table({"label": [7]})), uri="memory")
    ctx = _context(tmp_path)
    with pytest.raises(TypeError, match="not None"):
        app.run_train(dataset, ctx=ctx)


def test_training_does_not_require_artifacts(tmp_path: Path):
    app = Model("no-output")

    @app.train
    def train(dataset: Dataset[object], *, ctx: TrainingContext) -> None:
        del dataset, ctx

    dataset = Dataset(pads.dataset(pa.table({"label": [7]})), uri="memory")
    ctx = _context(tmp_path)
    assert app.run_train(dataset, ctx=ctx) is None
