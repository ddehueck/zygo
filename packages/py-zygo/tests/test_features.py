import importlib.util
from pathlib import Path
import sys

import pyarrow as pa


path = Path(__file__).parents[1] / "src/zygo/ml/features/__init__.py"
spec = importlib.util.spec_from_file_location("_test_zygo_features", path)
assert spec is not None and spec.loader is not None
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)


@module.features
class Sample(module.Features):
    label: int


def test_features_schema_and_decode():
    schema = pa.schema([("label", pa.int64())])
    assert "decode" not in Sample.__dict__
    assert Sample.to_schema().equals(schema)
    Sample.validate_arrow_schema(schema)
    assert Sample.decode({"label": 2}) == Sample(label=2)


def test_features_subclass():
    @module.features
    class Extended(Sample):
        name: str

    assert Extended.to_schema().names == ["label", "name"]
    assert Sample.to_schema().names == ["label"]
    assert Extended.decode({"label": 2, "name": "example"}) == Extended(
        label=2, name="example"
    )
