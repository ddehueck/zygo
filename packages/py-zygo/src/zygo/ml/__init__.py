"""
Zygo expose machine learning functionality to train and run inference for your models.

```
from zygo.ml import Model, Dataset, DatasetSchema, Context, ModelBundle

my_model = Model(name="my_model_for_xyz")

@my_model.train
def train(dataset: Dataset, ctx: Context) -> ModelBundle:
    # Use torch, jax, or any other ML framework
    ...
    return ModelBundle(path="results/)


@my_model.load
def load(bundle: ModelBundle) -> MyCustomModelInstance:
    # Load the trained model from the bundle
    ...

@my_model.infer
def infer(input: Input, model: MyCustomModelInstance) -> Dataset:
    # Use trained model to make predictions
    ...
    return dataset
```
"""
