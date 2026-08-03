from dataclasses import dataclass
from pathlib import Path
from typing import override

from fsspec import filesystem  # type: ignore
from fsspec.core import split_protocol  # type: ignore


@dataclass(frozen=True)
class FsspecUri:
    uri: str

    def __post_init__(self) -> None:
        """Basic validation of fsspec URI"""
        try:
            # Check if protocol can be parsed
            protocol, path = split_protocol(self.uri)
            # Check if protocol is supported
            filesystem(protocol)
            # Check if path is not empty (for most protocols)
            if not path and protocol not in {"memory"}:
                raise ValueError(f"Empty path for protocol: {protocol}")

        except Exception as e:
            raise ValueError(f"Invalid fsspec URI: {self.uri} ({e})") from e

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

    def to_absolute(self, root: str | None = None) -> "FsspecUri":
        """Convert the fsspec URI to an absolute file URI."""
        absolute_path = Path(root) / self.path if root else Path(self.path).resolve()
        return FsspecUri(f"{self.protocol}://{absolute_path}")

    def is_absolute(self) -> bool:
        """Check if the fsspec URI is absolute."""
        # TODO: Maybe split out the absolute/relative stuff into a seperate
        # local only subset of FsspecUri?
        return self.protocol in {"file", "memory"} and not self.path.startswith("/")

    def is_local(self) -> bool:
        """Check if the fsspec URI is a local filesystem."""
        return self.protocol in {"file", "memory"}

    @override
    def __str__(self) -> str:
        return self.uri
