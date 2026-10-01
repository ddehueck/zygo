"""Optional PyTorch adapter, imported lazily by Dataset.to_torch."""

from collections.abc import Callable
import operator

import pyarrow as pa
from torch.utils.data import Dataset

from zygo.ml.features import Features


class TorchDataset(Dataset[tuple[object, object]]):
    """A map-style Arrow dataset with optional feature decoding."""

    def __init__(
        self,
        table: pa.Table,
        inputs: str,
        target: str,
        transform: Callable[[object], object] | None = None,
        features: type[Features] | None = None,
    ) -> None:
        super().__init__()
        for name in (inputs, target):
            indices = table.schema.get_all_field_indices(name)
            if not indices:
                raise ValueError(f"Arrow table is missing required column {name!r}")
            if len(indices) != 1:
                raise ValueError(f"Arrow table has duplicate column {name!r}")
        if features is not None:
            features.validate_arrow_schema(table.schema)
        self.table = table
        self.inputs = inputs
        self.target = target
        self.transform = transform
        self.features = features

    def __len__(self) -> int:
        return self.table.num_rows

    def __getitem__(self, index: int) -> tuple[object, object]:
        index = operator.index(index)
        if index < 0:
            index += len(self)
        if not 0 <= index < len(self):
            raise IndexError("TorchDataset index out of range")

        row = self.table.slice(index, 1).to_pylist()[0]
        if self.features is not None:
            decoded = self.features.decode(row)
            inputs = getattr(decoded, self.inputs, row[self.inputs])
            target = getattr(decoded, self.target, row[self.target])
        else:
            inputs = row[self.inputs]
            target = row[self.target]
        if self.transform is not None:
            inputs = self.transform(inputs)
        return inputs, target
