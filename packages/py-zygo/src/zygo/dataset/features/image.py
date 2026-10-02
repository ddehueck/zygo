"""Pillow images stored as encoded bytes in Arrow binary columns."""

from __future__ import annotations

from dataclasses import MISSING
from io import BytesIO
from typing import TYPE_CHECKING

import pyarrow as pa

if TYPE_CHECKING:
    from PIL import Image as PILImage


def _load_pillow():
    try:
        from PIL import Image as PILImage
    except ModuleNotFoundError as exc:
        if exc.name != "PIL":
            raise
        raise ImportError(
            "Image encoding and decoding require Pillow. Install it with `pip install pillow`."
        ) from exc
    return PILImage


class Image:
    """An image feature decoded to a Pillow image, with PNG encoding by default."""

    @classmethod
    def to_arrow_field(cls, name: str, default: object = MISSING) -> pa.Field:
        return pa.field(name, pa.binary())

    @classmethod
    def decode_value(
        cls,
        name: str,
        value: object,
        default: object = MISSING,
    ) -> PILImage.Image:
        if not isinstance(value, bytes):
            raise TypeError(
                f"Image field {name!r} requires bytes, got {type(value).__name__}"
            )
        pillow = _load_pillow()
        try:
            with BytesIO(value) as buffer, pillow.open(buffer) as image:
                image.load()
                return image.copy()
        except (OSError, ValueError) as exc:
            raise ValueError(
                f"Image field {name!r} contains invalid image bytes"
            ) from exc

    @classmethod
    def encode_value(cls, value: PILImage.Image, *, format: str = "PNG") -> bytes:
        """Encode a Pillow image for insertion into an Arrow binary column."""
        pillow = _load_pillow()
        if not isinstance(value, pillow.Image):
            raise TypeError(f"Expected a Pillow image, got {type(value).__name__}")
        with BytesIO() as buffer:
            value.save(buffer, format=format)
            return buffer.getvalue()
