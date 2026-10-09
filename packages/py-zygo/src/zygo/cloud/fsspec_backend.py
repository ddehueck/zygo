from __future__ import annotations

from io import BytesIO
import json
import logging
from typing import IO, TYPE_CHECKING, cast, override
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import Request, urlopen

from fsspec.spec import AbstractFileSystem

if TYPE_CHECKING:
    import builtins
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

    The zygo cloud storage API is built on top of an object store
    and provides presigned urls for get and put operations.

    e.g.
       1. POST https://zygo.cloud/api/v1/store/presign with a body of {"op": "GET", "uri": "zygo://..."} --> {"url": "<presigned-url>"}
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
        self._logger = _VerboseLogger(enabled=verbose)
        self._api = _ZygoApiClient(api_host, api_bearer_auth)
        self._transfer = _PresignedUrlClient()

    @staticmethod
    def _ensure_uri(path: str) -> str:
        if path.startswith(f"{_PROTOCOL}://"):
            return path
        return f"{_PROTOCOL}://{path.lstrip('/')}"

    @staticmethod
    def _fs_path(uri: str) -> str:
        """Strip zygo:// so fsspec/pyarrow paths match url_to_fs base dirs."""
        prefix = f"{_PROTOCOL}://"
        if uri.startswith(prefix):
            return uri[len(prefix) :]
        return uri

    @classmethod
    def _normalize_entry(cls, entry: dict[str, object]) -> dict[str, object]:
        # Cloud store/ls returns Zygo URIs; fsspec paths are protocol-stripped.
        # pyarrow FSSpecHandler also requires `size` on every detail entry.
        return {
            **entry,
            "name": cls._fs_path(str(entry["name"])),
            "size": entry.get("size", 0),
        }

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
        uri = self._ensure_uri(path)

        match mode:
            case "rb":
                self._logger.info("Getting presigned zygo object url %s", uri)
                url = self._api.presign(uri, "GET")
                self._logger.info("Downloading zygo object %s", uri)
                return self._transfer.get(url)
            case "wb":
                self._logger.info("Opening zygo object for upload: %s", uri)
                return _UploadFile(self._api, self._transfer, uri, logger=self._logger)
            case _:
                raise ValueError(f"zygo:// does not support mode {mode!r}")

    @override
    def ls(
        self, path: str, detail: bool = True, **kwargs: object
    ) -> list[dict[str, object]] | list[str]:
        uri = self._ensure_uri(path)
        entries = [self._normalize_entry(entry) for entry in self._api.list(uri)]
        self._logger.info("Listed %d zygo objects under %s", len(entries), uri)
        if detail:
            return entries
        return [str(entry["name"]) for entry in entries]

    @override
    def _rm(self, path: str) -> None:
        uri = self._ensure_uri(path)
        self._api.delete(uri)
        self._logger.info("Deleted zygo object %s", uri)


class _UploadFile(BytesIO):
    def __init__(
        self,
        api: _ZygoApiClient,
        transfer: _PresignedUrlClient,
        uri: str,
        *,
        logger: _VerboseLogger,
    ) -> None:
        super().__init__()
        self._api = api
        self._transfer = transfer
        self._uri = uri
        self._logger = logger
        self._discard = False

    @override
    def close(self) -> None:
        if self.closed:
            return
        try:
            if not self._discard:
                url = self._api.presign(self._uri, "PUT")
                self._transfer.put(url, self.getvalue())
                self._logger.info("Uploaded zygo object %s", self._uri)
        finally:
            super().close()

    @override
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        self._discard = exc_type is not None
        super().__exit__(exc_type, exc_val, exc_tb)


class _ZygoApiClient:
    def __init__(self, host: str, bearer_auth: str) -> None:
        super().__init__()
        self._host = host.rstrip("/")
        self._bearer_auth = bearer_auth

    def _request(self, method: str, route: str, *, body: dict[str, object]) -> bytes:
        request = Request(  # ruff: ignore[suspicious-url-open-usage]
            f"{self._host}/v1/{route}",
            data=json.dumps(body).encode(),
            headers={
                "Authorization": f"Bearer {self._bearer_auth}",
                "Content-Type": "application/json",
            },
            method=method,
        )
        try:
            with cast("IO[bytes]", urlopen(request, timeout=30)) as response:  # ruff: ignore[suspicious-url-open-usage]
                return response.read()
        except HTTPError as error:
            if error.code == _NOT_FOUND:
                raise FileNotFoundError(str(body["uri"])) from error
            raise RuntimeError(
                f"Zygo Cloud API {method}:{route} failed (HTTP {error.code})"
            ) from error

    def _request_json(
        self, method: str, route: str, *, body: dict[str, object]
    ) -> dict[str, object]:
        data = self._request(method, route, body=body)
        try:
            return cast("dict[str, object]", json.loads(data))
        except json.JSONDecodeError as error:
            raise RuntimeError(
                f"Zygo Cloud API {method} /v1/{route} returned non-JSON, check api_host and API route"
            ) from error

    def presign(self, uri: str, method: str) -> str:
        response = self._request_json(
            "POST", "store/presign", body={"op": method, "uri": uri}
        )
        return str(response["url"])

    def list(self, uri: str) -> builtins.list[dict[str, object]]:
        if not uri.endswith("/"):
            uri += "/"
        response = self._request_json("POST", "store/ls", body={"uri": uri})
        return cast("builtins.list[dict[str, object]]", response["entries"])

    def delete(self, uri: str) -> None:
        self._request("DELETE", "store", body={"uri": uri})


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
        request = Request(url, data=data, method=method)  # ruff: ignore[suspicious-url-open-usage]
        return cast("IO[bytes]", urlopen(request, timeout=120))  # ruff: ignore[suspicious-url-open-usage]

    def get(self, url: str) -> IO[bytes]:
        # Buffer the object so readers that seek (pyarrow/parquet) work.
        # HTTPResponse from urlopen is not seekable.
        response = self._request(url, "GET")
        try:
            return BytesIO(response.read())
        finally:
            response.close()

    def put(self, url: str, data: bytes) -> None:
        response = self._request(url, "PUT", data)
        try:
            response.read()
        finally:
            response.close()
