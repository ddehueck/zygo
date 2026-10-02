from __future__ import annotations

from dataclasses import dataclass
import json

import pyarrow as pa


@dataclass(frozen=True, init=False)
class ClassLabel:
    """Ordered class names, with integer labels starting at zero."""

    names: tuple[str, ...]

    def __init__(self, *names: str) -> None:
        if any(not isinstance(name, str) for name in names):
            raise TypeError("ClassLabel names must be strings")
        if not names or any(not name for name in names):
            raise ValueError("ClassLabel requires nonempty class names")
        if len(set(names)) != len(names):
            raise ValueError("ClassLabel names must be unique")
        object.__setattr__(self, "names", names)

    @classmethod
    def to_arrow_field(cls, name: str, default: object) -> pa.Field:
        if not isinstance(default, cls):
            raise TypeError(f"ClassLabel field {name!r} requires a ClassLabel default")
        return pa.field(
            name,
            pa.int32(),
            metadata={b"class_names": json.dumps(default.names).encode("utf-8")},
        )

    @classmethod
    def decode_value(cls, name: str, value: object, default: object) -> int:
        if not isinstance(default, cls):
            raise TypeError(f"ClassLabel field {name!r} requires a ClassLabel default")
        if type(value) is not int:
            raise TypeError(f"Field {name!r} requires int, got {type(value).__name__}")
        if not 0 <= value < len(default.names):
            raise ValueError(
                f"Label field {name!r} must be in range "
                f"0..{len(default.names) - 1}, got {value}"
            )
        return value
