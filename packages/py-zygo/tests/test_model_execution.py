import pyarrow as pa
import pyarrow.dataset as pads
import pytest

from zygo import Dataset, DataUri, Model, TrainingContext, TrainingStore


def test_runtime_selects_training_and_loading_roots(tmp_path):
    app = Model("labels")

    @app.train
    def train(dataset: Dataset, *, ctx: TrainingContext) -> None:
        del dataset
        with ctx.store() as store:
            store.put("label.txt", b"7")

    @app.load
    def load(store: TrainingStore) -> int:
        return int(store.get("label.txt"))

    @app.infer
    def infer(model: int) -> int:
        return model

    dataset = Dataset(pads.dataset(pa.table({"label": [7]})), uri="memory")
    training = TrainingContext.local(DataUri((tmp_path / "training").as_uri() + "/"))
    assert app.run_train(dataset, ctx=training) is None
    assert app.run_load(training.store()) == 7

    selected = TrainingContext.local(DataUri((tmp_path / "selected").as_uri() + "/"))
    with selected.store() as store:
        store.put("label.txt", b"9")
    assert app.run_infer(app.run_load(selected.store())) == 9


def test_training_rejects_return_annotation():
    app = Model("invalid")
    with pytest.raises(TypeError, match="Training must return None"):

        @app.train
        def train(dataset: Dataset, *, ctx: TrainingContext) -> int:
            del dataset, ctx
            return 1


def test_training_rejects_non_none_runtime_result(tmp_path):
    app = Model("invalid-result")

    @app.train
    def train(dataset: Dataset, *, ctx: TrainingContext) -> None:
        del dataset, ctx
        return 1

    dataset = Dataset(pads.dataset(pa.table({"label": [7]})), uri="memory")
    ctx = TrainingContext.local(DataUri(tmp_path.as_uri() + "/"))
    with pytest.raises(TypeError, match="not None"):
        app.run_train(dataset, ctx=ctx)


def test_training_does_not_require_artifacts(tmp_path):
    app = Model("no-output")

    @app.train
    def train(dataset: Dataset, *, ctx: TrainingContext) -> None:
        pass

    dataset = Dataset(pads.dataset(pa.table({"label": [7]})), uri="memory")
    ctx = TrainingContext.local(DataUri(tmp_path.as_uri() + "/"))
    assert app.run_train(dataset, ctx=ctx) is None
