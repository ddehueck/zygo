"""
Minimal typed features backed by Arrow rows.

Example::

    @features
    class Sample:
        image: Image
        label: ClassLabel = ClassLabel("clear", "crystal", "precipitate")
"""

from __future__ import annotations

from collections.abc import Callable, Iterator, Mapping
from dataclasses import MISSING, dataclass, fields
from typing import (
    ClassVar,
    Self,
    cast,
    dataclass_transform,
    get_type_hints,
)

import pyarrow as pa

from .class_label import ClassLabel
from .image import Image
from .protocol import (
    _ALLOWED_PRIMITIVE_FEATURE_TYPES,
    _PRIMITIVE_ARROW_DTYPES,
    ZygoFeature,
    _FeatureT,
)

__all__ = ["ClassLabel", "Features", "Image", "features"]


def _get_feature_fields(cls: type[Features]) -> Iterator[tuple[str, _FeatureT, object]]:
    """Yield each feature field's name, annotated class, and default configuration."""
    hints = get_type_hints(cls)
    for field in fields(cls):
        annotation = hints[field.name]
        if isinstance(annotation, type) and issubclass(annotation, ZygoFeature):
            yield field.name, annotation, field.default
        elif annotation in _ALLOWED_PRIMITIVE_FEATURE_TYPES:
            yield field.name, cast("_FeatureT", annotation), field.default
        else:
            raise TypeError(
                f"Unsupported feature type for field {field.name!r}: {annotation!r}"
            )


class Features:
    """Base class for feature schemas declared with ``@features``."""

    _feature_fields: ClassVar[tuple[tuple[str, _FeatureT, object], ...]]

    @classmethod
    def to_schema(cls) -> pa.Schema:
        """Convert the feature schema to an Arrow Schema."""
        return pa.schema(
            _feature_to_pyarrow_field(name, annotation, default)
            for name, annotation, default in cls._feature_fields
        )

    @classmethod
    def validate_arrow_schema(cls, schema: pa.Schema) -> None:
        """
        Check required fields, exact dtypes, and expected metadata when present.
        Arrow lets you check the schema without loading the data so this check is fast.
        """
        for expected in cls.to_schema():
            name = expected.name
            indices = schema.get_all_field_indices(name)
            if not indices:
                raise ValueError(f"Arrow schema is missing required field {name!r}")
            if len(indices) != 1:
                raise ValueError(f"Arrow schema has duplicate field {name!r}")
            actual = schema.field(indices[0])
            if not actual.type.equals(expected.type):
                raise TypeError(
                    f"Arrow field {name!r} must be {expected.type}, got {actual.type}"
                )
            metadata = actual.metadata or {}
            for key, value in (expected.metadata or {}).items():
                if key in metadata and metadata[key] != value:
                    raise ValueError(
                        f"Arrow field {name!r} metadata {key!r} must match "
                        f"{value!r}, got {metadata[key]!r}"
                    )

    @classmethod
    def decode(cls, row: Mapping[str, object]) -> Self:
        """Decode a row, rejecting missing, null, or incorrectly typed values."""
        values: dict[str, object] = {}
        for name, annotation, default in cls._feature_fields:
            if name not in row:
                raise ValueError(f"Row is missing required field {name!r}")
            value = row[name]
            if value is None:
                raise ValueError(f"Required field {name!r} cannot be null")
            values[name] = _decode_feature(name, annotation, value, default)
        construct = cast("Callable[..., Self]", cls)
        return construct(**values)


@dataclass_transform(kw_only_default=True)
def features[T: Features](cls: type[T]) -> type[T]:
    """Turn a Features subclass into a keyword-only dataclass schema."""
    if not issubclass(cls, Features):
        raise TypeError("@features requires a Features subclass")
    decorate = cast("Callable[..., type[T]]", dataclass)
    cls = decorate(cls, kw_only=True)
    cls._feature_fields = tuple(_get_feature_fields(cls))
    return cls


def _feature_to_pyarrow_field(
    name: str,
    typeclass: _FeatureT,
    default: object = MISSING,
) -> pa.Field:
    if issubclass(typeclass, ZygoFeature):
        return typeclass.to_arrow_field(name, default)
    try:
        dtype = _PRIMITIVE_ARROW_DTYPES[typeclass]
    except KeyError:
        raise TypeError(f"Unsupported feature type: {typeclass!r}") from None
    return pa.field(name, dtype)


def _decode_feature(
    name: str,
    typeclass: _FeatureT,
    value: object,
    default: object = MISSING,
) -> object:
    if issubclass(typeclass, ZygoFeature):
        return typeclass.decode_value(name, value, default)
    if typeclass is float:
        if type(value) not in (int, float):
            raise TypeError(f"Field {name!r} requires a numeric value")
        return float(cast("int | float", value))
    if typeclass in (int, bool):
        valid = type(value) is typeclass
    else:
        valid = isinstance(value, typeclass)
    if not valid:
        raise TypeError(
            f"Field {name!r} requires {typeclass.__name__}, got {type(value).__name__}"
        )
    return value
