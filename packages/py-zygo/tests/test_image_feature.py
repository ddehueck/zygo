from pathlib import Path
import runpy

from PIL import Image as PILImage
import pyarrow as pa


def test_image_round_trip():
    path = Path(__file__).parents[1] / "src/zygo/dataset/features/image.py"
    Image = runpy.run_path(str(path))["Image"]
    original = PILImage.new("RGB", (1, 1), color=(10, 20, 30))
    table = pa.Table.from_pylist(
        [{"image": Image.encode_value(original)}],
        schema=pa.schema([Image.to_arrow_field("image")]),
    )
    decoded = Image.decode_value("image", table.to_pylist()[0]["image"])
    assert decoded.size == original.size
    assert decoded.tobytes() == original.tobytes()
