import json
import math
from typing import (
    Protocol,
    cast,
    get_args,
    get_origin,
    get_type_hints,
    is_typeddict,
    override,
)

from zygo.codecs.base import (
    Codec,
    CodecDecodeError,
    CodecEncodeError,
    FileExtension,
    FileFormat,
)

_DICT_TYPE_ARGUMENT_COUNT = 2


class _TypedDictType(Protocol):
    __required_keys__: frozenset[str]


class Json[T](Codec[T]):
    """A typed container encoded as canonical UTF-8 JSON.

    Passing ``dict`` as the value type enables arbitrary JSON objects whose
    values are validated recursively as JSON-compatible values.
    """

    def __init__(self, value_type: type[T]) -> None:
        super().__init__()
        _validate_type_shape(value_type)
        self._value_type = value_type

    @property
    @override
    def value_type(self) -> type[T]:
        return self._value_type

    @property
    @override
    def format(self) -> FileFormat:
        return FileFormat(extension=FileExtension(".json"))

    @override
    def encode(self, value: T, /) -> bytes:
        try:
            _validate_value(value, self._value_type, path="$", decoding=False)
            encoded = json.dumps(
                value,
                allow_nan=False,
                ensure_ascii=False,
                separators=(",", ":"),
                sort_keys=True,
            )
        except (TypeError, ValueError) as error:
            raise CodecEncodeError(str(error)) from error
        return encoded.encode("utf-8")

    @override
    def decode(self, payload: bytes, /) -> T:
        try:
            text = payload.decode("utf-8")
            value: object = json.loads(text)  # pyright: ignore[reportAny]
            _validate_value(value, self._value_type, path="$", decoding=True)
        except (
            UnicodeDecodeError,
            json.JSONDecodeError,
            TypeError,
            ValueError,
        ) as error:
            raise CodecDecodeError(str(error)) from error
        return cast("T", value)


def _validate_type_shape(value_type: object) -> None:
    if _is_json_primitive_type(value_type) or value_type is dict:
        return

    if is_typeddict(value_type):
        for field_type in _typed_dict_fields(value_type).values():
            _validate_type_shape(field_type)
        return

    origin = get_origin(value_type)
    args = cast("tuple[object, ...]", get_args(value_type))
    if origin is list and len(args) == 1:
        _validate_type_shape(args[0])
        return

    if origin is dict and len(args) == _DICT_TYPE_ARGUMENT_COUNT and args[0] is str:
        _validate_type_shape(args[1])
        return

    raise TypeError(
        "Json supports only None, bool, int, float, str, dict, list[T], dict[str, T], and TypedDicts with supported field types"
    )


def _validate_value(
    value: object,
    expected: object,
    *,
    path: str,
    decoding: bool,
) -> None:
    if _is_json_primitive_type(expected):
        _validate_primitive(value, expected, path=path)
        return

    if expected is dict:
        _validate_json_object(value, path=path)
        return

    if is_typeddict(expected):
        _validate_typed_dict(
            value,
            expected,
            path=path,
            decoding=decoding,
        )
        return

    origin = get_origin(expected)
    args = cast("tuple[object, ...]", get_args(expected))
    if origin is list:
        if not isinstance(value, list):
            _raise_type_error(path, expected, value)
        item_type = args[0]
        for index, item in enumerate(cast("list[object]", value)):
            _validate_value(
                item,
                item_type,
                path=f"{path}[{index}]",
                decoding=decoding,
            )
        return

    if origin is dict:
        if not isinstance(value, dict):
            _raise_type_error(path, expected, value)
        item_type = args[1]
        for key, item in cast("dict[object, object]", value).items():
            if not isinstance(key, str):
                raise TypeError(
                    f"{path} expected string keys, got {type(key).__name__}"
                )
            _validate_value(
                item,
                item_type,
                path=f"{path}.{key}",
                decoding=decoding,
            )
        return

    action = "decode" if decoding else "encode"
    raise TypeError(f"Unsupported JSON type while attempting to {action}: {expected!r}")


def _validate_json_object(value: object, *, path: str) -> None:
    if not isinstance(value, dict):
        _raise_type_error(path, dict, value)

    for key, item in cast("dict[object, object]", value).items():
        if not isinstance(key, str):
            raise TypeError(f"{path} expected string keys, got {type(key).__name__}")
        _validate_json_value(item, path=f"{path}.{key}")


def _validate_json_value(value: object, *, path: str) -> None:
    if value is None or type(value) is bool or type(value) is int:
        return
    if type(value) is float:
        if not math.isfinite(cast("float", value)):
            raise ValueError(f"{path} does not support NaN or infinity")
        return
    if type(value) is str:
        return
    if isinstance(value, list):
        for index, item in enumerate(cast("list[object]", value)):
            _validate_json_value(item, path=f"{path}[{index}]")
        return
    if isinstance(value, dict):
        _validate_json_object(value, path=path)
        return
    raise TypeError(f"{path} expected a JSON-compatible value, got {type(value).__name__}")


def _typed_dict_fields(value_type: object) -> dict[str, object]:
    return cast(
        "dict[str, object]",
        get_type_hints(cast("type[object]", value_type)),
    )


def _validate_typed_dict(
    value: object,
    expected: object,
    *,
    path: str,
    decoding: bool,
) -> None:
    if not isinstance(value, dict):
        _raise_type_error(path, expected, value)

    untyped_items = cast("dict[object, object]", value)
    for key in untyped_items:
        if not isinstance(key, str):
            raise TypeError(f"{path} expected string keys, got {type(key).__name__}")

    items = cast("dict[str, object]", value)
    field_types = _typed_dict_fields(expected)
    required_keys = cast("_TypedDictType", expected).__required_keys__
    missing_keys = required_keys - items.keys()
    if missing_keys:
        missing_key = min(missing_keys)
        raise TypeError(f"{path} missing required key {missing_key!r}")

    unexpected_keys = items.keys() - field_types.keys()
    if unexpected_keys:
        unexpected_key = min(unexpected_keys)
        raise TypeError(f"{path} got unexpected key {unexpected_key!r}")

    for key, item in items.items():
        _validate_value(
            item,
            field_types[key],
            path=f"{path}.{key}",
            decoding=decoding,
        )


def _is_json_primitive_type(value_type: object) -> bool:
    return (
        value_type is type(None)
        or value_type is bool
        or value_type is int
        or value_type is float
        or value_type is str
    )


def _validate_primitive(value: object, expected: object, *, path: str) -> None:
    if expected is type(None):
        _validate_none(value, expected=expected, path=path)
        return
    if expected is float:
        _validate_float(value, expected=expected, path=path)
        return
    _validate_scalar(value, expected=expected, path=path)


def _validate_none(value: object, *, expected: object, path: str) -> None:
    if value is not None:
        _raise_type_error(path, expected, value)


def _validate_float(value: object, *, expected: object, path: str) -> None:
    if type(value) is not float:
        _raise_type_error(path, expected, value)
    if not math.isfinite(cast("float", value)):
        raise ValueError(f"{path} does not support NaN or infinity")


def _validate_scalar(value: object, *, expected: object, path: str) -> None:
    if expected is bool and type(value) is not bool:
        _raise_type_error(path, expected, value)
    if expected is int and type(value) is not int:
        _raise_type_error(path, expected, value)
    if expected is str and type(value) is not str:
        _raise_type_error(path, expected, value)


def _raise_type_error(path: str, expected: object, value: object) -> None:
    raise TypeError(f"{path} expected {expected!r}, got {type(value).__name__}")
