from dataclasses import dataclass

from pydantic import BaseModel, Field

import zygo

from .dataset import DATASET_PATH, BlackWhiteFeatures


@dataclass
class ConstantClassifier:
    label: int


class Prediction(BaseModel):
    label: int


class TrainParams(zygo.ml.HyperParams):
    epochs: int = Field(default=10, ge=1)
    batch_size: int = Field(default=16, ge=1)
    learning_rate: float = Field(default=1e-3, gt=0)
    seed: int = 42


app = zygo.Model("constant-classifier")


@app.train
def train(
    dataset: zygo.Dataset[BlackWhiteFeatures],
    *,
    params: TrainParams,
    ctx: zygo.TrainingContext,
) -> None:
    label = next(iter(dataset)).label
    print(f"training with label {label} and params {params.model_dump(mode='json')}")
    ctx.store.put("label.txt", str(label).encode())


@app.load
def load(store: zygo.ModelStore) -> ConstantClassifier:
    return ConstantClassifier(label=int(store.get("label.txt")))


@app.infer
def infer(model: ConstantClassifier, image: zygo.Image) -> Prediction:
    """Ignore the image and always predict the learned label."""
    return Prediction(label=model.label)


if __name__ == "__main__":
    store = zygo.train(model=app, dataset=DATASET_PATH, params=TrainParams(epochs=5))
    dataset = zygo.Dataset.open(DATASET_PATH, features=BlackWhiteFeatures)
    prediction = zygo.infer(model=app, store=store, data=next(iter(dataset)).image)
    print(prediction)
