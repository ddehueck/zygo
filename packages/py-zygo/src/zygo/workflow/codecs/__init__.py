# Ready-to-use codec instances are intentionally initialized in this facade.
# ruff: file-ignore[non-empty-init-module]

from zygo.workflow.codecs.base import (
    Codec,
    CodecDecodeError,
    CodecEncodeError,
    CodecError,
    FileExtension,
    FileFormat,
)
from zygo.workflow.codecs.fs import (
    File as _File,
    FileMap as _FileMap,
    Folder as _Folder,
)
from zygo.workflow.codecs.json import Json
from zygo.workflow.codecs.primitives import (
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
    "File",
    "FileExtension",
    "FileFormat",
    "FileMap",
    "Float",
    "Folder",
    "Integer",
    "Json",
    "String",
]
