from zygo.codecs.base import (
    Codec,
    CodecDecodeError,
    CodecEncodeError,
    CodecError,
    FileExtension,
    FileFormat,
)
from zygo.codecs.json import Json
from zygo.codecs.fs import FileMap as _FileMap, Folder as _Folder, File as _File
from zygo.codecs.primitives import (
    Boolean as _Boolean,
    Bytes as _Bytes,
    Float as _Float,
    Integer as _Integer,
    String as _String,
)

Boolean = _Boolean()
Bytes = _Bytes()
Float = _Float()
Integer = _Integer()
String = _String()
FileMap = _FileMap()
Folder = _Folder()
File = _File()


__all__ = [
    "Boolean",
    "Bytes",
    "Codec",
    "CodecDecodeError",
    "CodecEncodeError",
    "CodecError",
    "FileExtension",
    "FileFormat",
    "FileMap",
    "Folder",
    "File",
    "Float",
    "Integer",
    "Json",
    "String",
]
