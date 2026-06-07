import base64
from collections.abc import Sequence
import hashlib

hash_arg = bytes | str | Sequence[bytes | str]


def _normalize_hash_arg(val: hash_arg) -> bytes:
    if isinstance(val, str):
        return val.encode("utf-8")
    if isinstance(val, bytes):
        return val
    return b"".join(v.encode("utf-8") if isinstance(v, str) else v for v in val)


def hash_bytes(val: hash_arg, nbytes: int = 12) -> bytes:
    """Hash bytes or a list of bytes, returning base64-encoded result."""
    val = _normalize_hash_arg(val)
    h = hashlib.blake2s(val, digest_size=nbytes).digest()
    return base64.urlsafe_b64encode(h)


def hash_to_str(val: hash_arg, nbytes: int = 12) -> str:
    """Hash bytes/strings and return as ASCII string."""
    return hash_bytes(val, nbytes).decode("ascii")
