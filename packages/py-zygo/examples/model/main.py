from dataclasses import dataclass
from pathlib import Path
import sys
from typing import Annotated

from PIL import Image
import zygo
from pydantic import BaseModel


@zygo.features
class Features(zygo.Features):
    image: zygo.Image
    label: zygo.ClassLabel = zygo.ClassLabel("black", "white")


@dataclass
class ConstantClassifier:
    label: int


class Prediction(BaseModel):
    label: int


app = zygo.Model("constant-classifier")


@app.train
def train(
    dataset: zygo.Dataset[Features], *, ctx: zygo.TrainingContext
) -> None:
    label = next(iter(dataset)).label
    with ctx.store() as store:
        store.put("label.txt", str(label).encode())



@app.load
def load(store: zygo.TrainingStore) -> ConstantClassifier:
    return ConstantClassifier(label=int(store.get("label.txt")))


@app.infer
def infer(model: ConstantClassifier, image: zygo.Image) -> Prediction:
    """Ignore the image and always predict the learned label."""
    return Prediction(label=model.label)


if __name__ == "__main__":
    dataset_uri = sys.argv[1] if len(sys.argv) > 1 else "data.parquet"
    ctx = zygo.TrainingContext.local(zygo.DataUri(Path("bundle").resolve().as_uri() + "/"))
    app.run_train(dataset_uri, ctx=ctx)
    model = app.run_load(ctx.store())
    print(app.run_infer(model, image=Image.new("RGB", (1, 1))))
