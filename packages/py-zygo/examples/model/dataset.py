"""Create a one-row dataset containing one black pixel labeled 0."""

from PIL import Image

import zygo

DATASET_PATH = "./data/"


@zygo.features
class BlackWhiteFeatures(zygo.Features):
    image: zygo.Image
    label: zygo.ClassLabel = zygo.ClassLabel("black", "white")


if __name__ == "__main__":
    with zygo.Dataset.builder(
        DATASET_PATH, schema=BlackWhiteFeatures, overwrite=True
    ) as builder:
        builder.add({"image": Image.new("RGB", (1, 1)), "label": 0})

    ds = zygo.Dataset.open(DATASET_PATH, features=BlackWhiteFeatures)
    print(f"Arrow schema:\n-----\n{ds.arrow_schema}\n-----\n")  # ruff: ignore[print]
    print(f"Features: {ds.features}")  # ruff: ignore[print]
    print(f"Dataset length: {len(ds)}")  # ruff: ignore[print]
    for row in ds:
        print(row)  # ruff: ignore[print]
