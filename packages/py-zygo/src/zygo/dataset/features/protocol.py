"""Feature protocol and supported primitive Arrow types."""

from __future__ import annotations

from typing import Protocol, runtime_checkable

import pyarrow as pa


@runtime_checkable
class ZygoFeature(Protocol):
    """Base class for zygo features"""

    @classmethod
    def to_arrow_field(cls, name: str, default: object) -> pa.Field[pa.DataType]:
        raise NotImplementedError

    @classmethod
    def decode_value(cls, name: str, value: object, default: object) -> object:
        raise NotImplementedError


_ALLOWED_PRIMITIVE_FEATURE_TYPES = (str, int, float, bool, bytes)

type _FeatureT = type[ZygoFeature] | type[str | int | float | bool | bytes]


_PRIMITIVE_ARROW_DTYPES: dict[type[str | int | float | bool | bytes], pa.DataType] = {
    str: pa.string(),
    int: pa.int64(),
    float: pa.float64(),
    bool: pa.bool_(),
    bytes: pa.binary(),
}
