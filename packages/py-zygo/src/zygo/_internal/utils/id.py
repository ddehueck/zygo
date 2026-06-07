import secrets
import time
from typing import TYPE_CHECKING, cast
import uuid as uuid_module
from uuid import UUID

if TYPE_CHECKING:
    from collections.abc import Callable

UUID7_VERSION = 7


def uuid7() -> UUID:
    stdlib_uuid7 = getattr(uuid_module, "uuid7", None)
    if callable(stdlib_uuid7):
        uuid7_fn = cast("Callable[[], UUID]", stdlib_uuid7)
        return uuid7_fn()

    # Python < 3.14 compatibility: synthesize UUIDv7 following RFC 9562 layout.
    timestamp_ms = int(time.time() * 1000) & ((1 << 48) - 1)
    rand_b = secrets.randbits(12)
    rand_c = secrets.randbits(62)

    uuid_int = timestamp_ms << 80
    uuid_int |= UUID7_VERSION << 76
    uuid_int |= rand_b << 64
    uuid_int |= 0b10 << 62
    uuid_int |= rand_c

    return UUID(int=uuid_int)


class UUID7(UUID):
    def __init__(self, id: str | UUID | int) -> None:
        # Handle both string and UUID object inputs
        if isinstance(id, UUID):
            uuid = id
            id_str = str(id)
        elif isinstance(id, int):
            uuid = UUID(int=id)
            id_str = str(id)
        else:
            try:
                uuid = UUID(id)
                id_str = id
            except ValueError as e:
                raise ValueError(f"Invalid UUID7: {id}") from e

        if uuid.version != UUID7_VERSION:
            raise ValueError(f"Invalid UUID7: {id_str}, version must be 7")
        super().__init__(id_str)

    @staticmethod
    def generate() -> "UUID7":
        return UUID7(uuid7())
