"""Errors returned by the Zygo Cloud API."""


class CloudApiError(RuntimeError):
    """An HTTP error returned by the Zygo Cloud API."""

    def __init__(self, status: int, message: str) -> None:
        super().__init__(f"Zygo Cloud API returned HTTP {status}: {message}")
        self.status = status
        self.message = message


class CloudConnectionError(RuntimeError):
    """The Zygo Cloud API host could not be reached."""

    def __init__(self, host: str, reason: str) -> None:
        super().__init__(f"Could not connect to Zygo Cloud API host {host}: {reason}")
        self.host = host
        self.reason = reason
