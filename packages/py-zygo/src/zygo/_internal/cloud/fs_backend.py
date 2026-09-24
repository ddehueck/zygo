"""Provisional fsspec backend backed by Zygo-authorized, direct object transfers.

All Zygo API routes and JSON response shapes are placeholders. Zygo authorizes each
object operation and returns short-lived presigned URLs. Bytes never transit Zygo.
"""

from __future__ import annotations

from io import SEEK_CUR, SEEK_END, BufferedIOBase, BufferedReader, RawIOBase
import json
import os
from typing import IO, TYPE_CHECKING, cast, override
from urllib.error import HTTPError
from urllib.parse import quote, urlencode
from urllib.request import Request, urlopen

from fsspec.spec import AbstractFileSystem  # type: ignore

# Reader/writer helpers collaborate with the filesystem's internal API.
# pyright: reportPrivateUsage=false

if TYPE_CHECKING:
    from http.client import HTTPResponse
    from types import TracebackType

_NOT_FOUND = 404
_PARTIAL_CONTENT = 206
_PART_SIZE = 64 * 1024 * 1024  # 100 GiB fits below S3's 10,000-part limit.


def _transfer(
    url: str,
    method: str,
    *,
    data: bytes | None = None,
    start: int | None = None,
    length: int | None = None,
) -> tuple[bytes, str | None]:
    if not url.startswith("https://"):
        raise ValueError("Presigned transfer URL must use HTTPS")
    headers: dict[str, str] = {}
    if start is not None and length is not None:
        headers["Range"] = f"bytes={start}-{start + length - 1}"
    request = Request(url, data=data, headers=headers, method=method)  # noqa: S310
    # TODO: Retry failed parts and ranges, requesting a fresh presigned URL on expiry.
    try:
        with cast("HTTPResponse", urlopen(request, timeout=120)) as response:  # noqa: S310
            if start is not None and response.status != _PARTIAL_CONTENT:
                raise OSError("Object storage did not honor the requested byte range")
            return response.read(
                length + 1 if length is not None else -1
            ), response.headers.get("ETag")
    except HTTPError as error:
        raise OSError(f"Object storage {method} failed (HTTP {error.code})") from error


class _RangedReader(RawIOBase):
    def __init__(
        self, fs: ZygoFileSystem, path: str, size: int, block_size: int
    ) -> None:
        super().__init__()
        self._fs = fs
        self._path = path
        self._size = size
        self._position = 0
        self._block_size = block_size

    @override
    def readable(self) -> bool:
        return True

    @override
    def seekable(self) -> bool:
        return True

    @override
    def tell(self) -> int:
        return self._position

    @override
    def seek(self, offset: int, whence: int = 0) -> int:
        if whence == SEEK_CUR:
            offset += self._position
        elif whence == SEEK_END:
            offset += self._size
        elif whence != 0:
            raise ValueError("Invalid seek mode")
        if offset < 0:
            raise ValueError("Cannot seek before start of file")
        self._position = offset
        return offset

    @override
    def readinto(self, buffer: bytearray | memoryview) -> int:  # type: ignore[reportIncompatibleMethodOverride]
        if self._position >= self._size:
            return 0
        length = min(len(buffer), self._block_size, self._size - self._position)
        url = self._fs._signed_url(
            "GET", f"{self._fs._object_route(self._path)}/download"
        )
        data, _ = _transfer(url, "GET", start=self._position, length=length)
        if len(data) != length:
            raise OSError("Object storage returned an incomplete byte range")
        buffer[:length] = data
        self._position += length
        return length


class _MultipartWriter(BufferedIOBase):
    def __init__(self, fs: ZygoFileSystem, path: str) -> None:
        super().__init__()
        self._fs = fs
        self._path = path
        self._buffer = bytearray()
        self._upload_id: str | None = None
        self._parts: list[dict[str, str | int]] = []
        self._aborted = False

    @override
    def writable(self) -> bool:
        return True

    @override
    def write(self, data: bytes | bytearray | memoryview) -> int:  # type: ignore[reportIncompatibleMethodOverride]
        if self.closed or self._aborted:
            raise ValueError("Upload is closed")
        source = memoryview(data)
        length = len(source)
        offset = 0
        while offset < length:
            chunk_size = min(_PART_SIZE - len(self._buffer), length - offset)
            self._buffer.extend(source[offset : offset + chunk_size])
            offset += chunk_size
            if len(self._buffer) == _PART_SIZE:
                self._send_part()
        return length

    def _send_part(self) -> None:
        if self._upload_id is None:
            result = self._fs._json_request(
                "POST", f"{self._fs._object_route(self._path)}/uploads"
            )
            self._upload_id = str(result["upload_id"])
        number = len(self._parts) + 1
        route = f"{self._fs._object_route(self._path)}/uploads/{quote(self._upload_id, safe='')}/parts/{number}"
        url = self._fs._signed_url("POST", route)
        _, etag = _transfer(url, "PUT", data=bytes(self._buffer))
        if not etag:
            raise OSError("Object storage did not return a part ETag")
        self._parts.append({"part_number": number, "etag": etag})
        self._buffer.clear()

    def _abort(self) -> None:
        self._aborted = True
        if self._upload_id is not None:
            route = f"{self._fs._object_route(self._path)}/uploads/{quote(self._upload_id, safe='')}"
            self._fs._request("DELETE", route)

    def _complete(self) -> None:
        if self._upload_id is None and not self._buffer:
            url = self._fs._signed_url(
                "POST", f"{self._fs._object_route(self._path)}/upload"
            )
            _transfer(url, "PUT", data=b"")
            return
        if self._buffer:
            self._send_part()
        route = f"{self._fs._object_route(self._path)}/uploads/{quote(self._upload_id or '', safe='')}/complete"
        self._fs._json_request("POST", route, {"parts": self._parts})

    @override
    def close(self) -> None:
        if self.closed:
            return
        try:
            if not self._aborted:
                self._complete()
        except Exception:
            self._abort()
            raise
        finally:
            super().close()

    @override
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        if exc_type is not None:
            self._abort()
        super().__exit__(exc_type, exc_value, traceback)


class ZygoFileSystem(AbstractFileSystem):
    """fsspec ``zygo://`` storage via presigned S3 uploads and ranged downloads.

    The provisional API is specified in protocol/cloud/v1/openapi.json.
    Object keys are percent-encoded as single URL path segments.
    """

    protocol = "zygo"
    root_marker = ""

    def __init__(
        self, *, api_url: str = "https://api.zygo.cloud", **kwargs: object
    ) -> None:
        if not api_url.startswith("https://"):
            raise ValueError("Zygo Cloud API URL must use HTTPS")
        super().__init__(**kwargs)  # type: ignore[reportUnknownMemberType]
        self.api_url = api_url.rstrip("/")

    @classmethod
    @override
    def _strip_protocol(cls, path: str) -> str:
        return path.removeprefix("zygo://").lstrip("/")

    def _request(self, method: str, route: str, *, data: bytes | None = None) -> bytes:
        secret = os.environ.get("ZYGO_API_SECRET")
        if not secret:
            raise RuntimeError("ZYGO_API_SECRET is required to access zygo:// storage")
        request = Request(  # noqa: S310
            f"{self.api_url}/v1/{route}",
            data=data,
            headers={
                "Authorization": f"Bearer {secret}",
                "Content-Type": "application/json",
            },
            method=method,
        )
        try:
            with cast("IO[bytes]", urlopen(request, timeout=30)) as response:  # noqa: S310
                return response.read()
        except HTTPError as error:
            if error.code == _NOT_FOUND:
                raise FileNotFoundError(route) from error
            raise OSError(f"Zygo Cloud {method} failed (HTTP {error.code})") from error

    def _json_request(
        self, method: str, route: str, body: dict[str, object] | None = None
    ) -> dict[str, object]:
        data = json.dumps(body).encode() if body is not None else None
        return cast(
            "dict[str, object]", json.loads(self._request(method, route, data=data))
        )

    def _signed_url(self, method: str, route: str) -> str:
        return str(self._json_request(method, route)["url"])

    def _object_route(self, path: str) -> str:
        return f"objects/{quote(self._strip_protocol(path), safe='')}"

    @override
    def _open(  # type: ignore[reportIncompatibleMethodOverride]
        self,
        path: str,
        mode: str = "rb",
        block_size: int | None = None,
        autocommit: bool = True,
        cache_options: dict[str, object] | None = None,
        **kwargs: object,
    ) -> IO[bytes]:
        path = self._strip_protocol(path)
        if mode == "rb":
            size = int(cast("int", self.info(path)["size"]))
            if block_size is not None and block_size <= 0:
                raise ValueError("block_size must be positive")
            read_size = min(block_size or _PART_SIZE, _PART_SIZE)
            return BufferedReader(
                _RangedReader(self, path, size, read_size),
                buffer_size=read_size,
            )
        if mode == "wb":
            return cast("IO[bytes]", _MultipartWriter(self, path))
        raise NotImplementedError(f"zygo:// does not support mode {mode!r}")

    @override
    def info(self, path: str, **kwargs: object) -> dict[str, object]:
        route = f"metadata/{quote(self._strip_protocol(path), safe='')}"
        return self._json_request("GET", route)

    @override
    def ls(
        self, path: str, detail: bool = True, **kwargs: object
    ) -> list[dict[str, object]] | list[str]:
        route = f"objects?{urlencode({'prefix': self._strip_protocol(path)})}"
        entries = cast(
            "list[dict[str, object]]", self._json_request("GET", route)["entries"]
        )
        if detail:
            return entries
        return [str(entry["name"]) for entry in entries]

    @override
    def _rm(self, path: str) -> None:
        self._request("DELETE", self._object_route(path))
