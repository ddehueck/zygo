from dataclasses import dataclass

from pydantic import BaseModel

import zygo

from .dataset import BlackWhiteFeatures


@dataclass
class ConstantClassifier:
    label: int


class Prediction(BaseModel):
    label: int


app = zygo.Model("constant-classifier")


@app.train
def train(
    dataset: zygo.Dataset[BlackWhiteFeatures], *, ctx: zygo.TrainingContext
) -> None:
    label = next(iter(dataset)).label
    ctx.store.put("label.txt", str(label).encode())


@app.load
def load(store: zygo.ModelStore) -> ConstantClassifier:
    return ConstantClassifier(label=int(store.get("label.txt")))


@app.infer
def infer(model: ConstantClassifier, image: zygo.Image) -> Prediction:
    """Ignore the image and always predict the learned label."""
    return Prediction(label=model.label)
