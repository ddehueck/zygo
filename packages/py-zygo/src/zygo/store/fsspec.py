from dataclasses import dataclass
from pathlib import Path
from typing import override

from fsspec import filesystem  # type: ignore
from fsspec.core import split_protocol  # type: ignore


@dataclass(frozen=True)
class FsspecUri:
    uri: str

    def __post_init__(self) -> None:
        """Validate and normalize the fsspec URI."""
        try:
            object.__setattr__(self, "uri", self._parse(self.uri))
        except Exception as e:
            raise ValueError(f"Invalid fsspec URI: {self.uri} ({e})") from e

    @staticmethod
    def _parse(uri: str) -> str:
        """Validate an fsspec URI and normalize local paths to absolute URIs."""
        protocol, path = split_protocol(uri)
        filesystem(protocol) # raise on invalid protocol

        if not path and protocol not in {"memory"}:
            raise ValueError(f"Empty path for protocol: {protocol}")

        # default to file protocol if none specified and ensure we have absolute paths locally
        effective_protocol = protocol or "file"
        if effective_protocol in {"file", "memory"} and not path.startswith("/"):
            path = str(Path(path).resolve())
            return f"{effective_protocol}://{path}"

        return uri

    @classmethod
    def try_parse(cls, uri: str) -> "FsspecUri | None":
        """Parse a URI, returning None instead of raising when it is invalid."""
        try:
            return cls(uri)
        except ValueError:
            return None

    @classmethod
    def is_valid(cls, uri: str) -> bool:
        """Return whether the input is a valid fsspec URI."""
        return cls.try_parse(uri) is not None

    @property
    def protocol(self) -> str:
        """Get the protocol of the fsspec URI."""
        protocol = split_protocol(self.uri)[0]
        if protocol is None:
            return "file"
        return protocol

    @property
    def path(self) -> str:
        """Get the path of the fsspec URI."""
        return split_protocol(self.uri)[1]

    @property
    def key(self) -> str:
        """Get the key of the fsspec URI."""
        return Path(self.path).name

    def to_absolute(self) -> "FsspecUri":
        """Convert the fsspec URI to an absolute file URI."""
        absolute_path = Path(self.path).resolve()
        return FsspecUri(f"{self.protocol}://{absolute_path}")

    def is_absolute(self) -> bool:
        """Check if the fsspec URI is absolute."""
        # TODO: Maybe split out the absolute/relative stuff into a seperate
        # local only subset of FsspecUri?
        return self.protocol in {"file", "memory"} and self.path.startswith("/")

    def is_local(self) -> bool:
        """Check if the fsspec URI is a local filesystem."""
        return self.protocol in {"file", "memory"}

    @override
    def __str__(self) -> str:
        return self.uri

