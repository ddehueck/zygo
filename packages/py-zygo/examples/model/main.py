from dataclasses import dataclass
from pathlib import Path
import sys
from typing import Annotated

from PIL import Image
import zygo


@zygo.features
class Features(zygo.Features):
    image: zygo.Image
    label: Annotated[int, zygo.ClassLabel("black")]


@dataclass
class ConstantClassifier:
    label: int


class Prediction(zygo.Struct):
    label: int


app = zygo.Model("constant-classifier")


@app.train
def train(
    dataset: zygo.Dataset[Features], *, ctx: zygo.TrainingContext
) -> zygo.ModelBundle:
    label = next(iter(dataset)).label
    artifact = ctx.store.put("label.txt", str(label).encode())
    prefix = zygo.DataUri(str(artifact).rsplit("/", 1)[0] + "/")
    return ctx.bundle(prefix)


@app.load
def load(bundle: zygo.ModelBundle) -> ConstantClassifier:
    with bundle.open("label.txt") as artifact:
        return ConstantClassifier(label=int(artifact.read()))


@app.infer
def infer(model: ConstantClassifier, image: zygo.Image) -> Prediction:
    """Ignore the image and always predict the learned label."""
    return Prediction(label=model.label)


if __name__ == "__main__":
    dataset_uri = sys.argv[1] if len(sys.argv) > 1 else "data.parquet"
    ctx = zygo.TrainingContext.local(zygo.DataUri(Path("bundle").resolve().as_uri() + "/"))
    bundle = app.run_train(dataset_uri, ctx=ctx)
    model = app.run_load(bundle)
    print(app.run_infer(model, image=Image.new("RGB", (1, 1))))
