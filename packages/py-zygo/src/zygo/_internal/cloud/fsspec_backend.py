from __future__ import annotations

from io import BytesIO
import json
import logging
from typing import IO, TYPE_CHECKING, cast, override
from urllib.error import HTTPError
from urllib.parse import quote, urlencode, urlsplit
from urllib.request import Request, urlopen

from fsspec.spec import AbstractFileSystem  # type: ignore

if TYPE_CHECKING:
    from types import TracebackType

_NOT_FOUND = 404
_PROTOCOL = "zygo"
_logger = logging.getLogger(__name__)


class _VerboseLogger:
    def __init__(self, *, enabled: bool) -> None:
        super().__init__()
        self._enabled = enabled

    def info(self, message: str, *args: object) -> None:
        if self._enabled:
            _logger.info(message, *args)


class ZygoFileSystem(AbstractFileSystem):
    """
    The zygo cloud storage backend - zygo://<path>

    The zygo cloud storage API is built on top of object store
    and provides presigned urls for get and put operations.

    e.g.
       1. POST https://api.zygo.cloud/v1/store/<path>/presign with GET or PUT method --> {"url": "<presigned-url>"}
       2. Use the presigned URL to retrieve (GET) or upload (PUT) the object

    """

    protocol = _PROTOCOL
    root_marker = ""

    def __init__(self, **kwargs: object) -> None:
        # Must be provided by the caller via the --store-config options via the run CLI
        api_host = kwargs.pop("api_host", None)
        api_bearer_auth = kwargs.pop("api_bearer_auth", None)
        verbose = kwargs.pop("verbose", False)

        if not isinstance(api_host, str) or not api_host:
            raise ValueError("api_host must be provided")
        if not isinstance(api_bearer_auth, str) or not api_bearer_auth:
            raise ValueError("api_bearer_auth must be provided")
        if not isinstance(verbose, bool):
            raise ValueError("verbose must be a boolean")

        super().__init__(**kwargs)  # type: ignore[reportUnknownMemberType]
        self._verbose = verbose
        self._logger = _VerboseLogger(enabled=self._verbose)
        self._api = _ZygoApiClient(api_host, api_bearer_auth)
        self._transfer = _PresignedUrlClient()

    @classmethod
    @override
    def _strip_protocol(cls, path: str) -> str:
        return path.removeprefix(f"{_PROTOCOL}://").lstrip("/")

    # fsspec infers AbstractBufferedFile, but this backend returns other binary streams.
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

        match mode:
            case "rb":
                self._logger.info("Getting presigned zygo object url %s", path)
                url = self._api.presign(path, "GET")
                self._logger.info("Downloading zygo object %s", path)
                return self._transfer.get(url)
            case "wb":
                self._logger.info("Opening zygo object for upload: %s", path)
                return _UploadFile(self._api, self._transfer, path, logger=self._logger)
            case _:
                raise ValueError(f"zygo:// does not support mode {mode!r}")

    @override
    def ls(
        self, path: str, detail: bool = True, **kwargs: object
    ) -> list[dict[str, object]] | list[str]:
        prefix = self._strip_protocol(path)
        entries = self._api.list(prefix)
        self._logger.info("Listed %d zygo objects under %s", len(entries), prefix)
        if detail:
            return entries
        return [str(entry["name"]) for entry in entries]

    @override
    def _rm(self, path: str) -> None:
        path = self._strip_protocol(path)
        self._api.delete(path)
        self._logger.info("Deleted zygo object %s", path)


class _UploadFile(BytesIO):
    def __init__(
        self,
        api: _ZygoApiClient,
        transfer: _PresignedUrlClient,
        path: str,
        *,
        logger: _VerboseLogger,
    ) -> None:
        super().__init__()
        self._api = api
        self._transfer = transfer
        self._path = path
        self._logger = logger
        self._discard = False

    @override
    def close(self) -> None:
        if self.closed:
            return
        try:
            if not self._discard:
                url = self._api.presign(self._path, "PUT")
                self._transfer.put(url, self.getvalue())
                self._logger.info("Uploaded zygo object %s", self._path)
        finally:
            super().close()

    @override
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self._discard = exc_type is not None
        super().__exit__(exc_type, exc_value, traceback)


class _ZygoApiClient:
    def __init__(self, host: str, bearer_auth: str) -> None:
        super().__init__()
        self._host = host.rstrip("/")
        self._bearer_auth = bearer_auth

    def _request(self, method: str, route: str, *, data: bytes | None = None) -> bytes:
        request = Request(  # noqa: S310
            f"{self._host}/v1/{route}",
            data=data,
            headers={
                "Authorization": f"Bearer {self._bearer_auth}",
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
            raise RuntimeError(
                f"Zygo Cloud API {method}:{route} failed (HTTP {error.code})"
            ) from error

    def presign(self, path: str, method: str) -> str:
        encoded_path = quote(path, safe="")
        route = f"store/{encoded_path}/presign"
        data = json.dumps({"method": method}).encode()
        try:
            response = cast(
                "dict[str, object]", json.loads(self._request("POST", route, data=data))
            )
        except json.JSONDecodeError as error:
            raise RuntimeError(
                f"Zygo Cloud API POST /v1/{route} returned non-JSON; check api_host and API route"
            ) from error
        return str(response["url"])

    def list(self, prefix: str) -> list[dict[str, object]]:
        route = f"ls?{urlencode({'prefix': prefix})}"
        response = cast("dict[str, object]", json.loads(self._request("GET", route)))
        return cast("list[dict[str, object]]", response["entries"])

    def delete(self, path: str) -> None:
        self._request("DELETE", quote(path, safe=""))


class _PresignedUrlClient:
    @staticmethod
    def _request(url: str, method: str, data: bytes | None = None) -> IO[bytes]:
        parsed = urlsplit(url)
        is_local_http = parsed.scheme == "http" and parsed.hostname in {
            "localhost",
            "127.0.0.1",
            "::1",
        }
        if parsed.scheme != "https" and not is_local_http:
            raise ValueError("Presigned transfer URL must use HTTPS or loopback HTTP")
        request = Request(url, data=data, method=method)  # noqa: S310
        return cast("IO[bytes]", urlopen(request, timeout=120))  # noqa: S310

    def get(self, url: str) -> IO[bytes]:
        return self._request(url, "GET")

    def put(self, url: str, data: bytes) -> None:
        with self._request(url, "PUT", data):
            pass
