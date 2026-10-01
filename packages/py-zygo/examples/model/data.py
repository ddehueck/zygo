"""Create a one-row dataset containing one black pixel labeled 0."""

from io import BytesIO

from PIL import Image
import pyarrow as pa
import pyarrow.parquet as pq


if __name__ == "__main__":
    image = BytesIO()
    Image.new("RGB", (1, 1)).save(image, format="PNG")
    table = pa.table({"image": [image.getvalue()], "label": [0]})
    pq.write_table(table, "data.parquet")
