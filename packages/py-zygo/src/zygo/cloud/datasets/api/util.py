"""Small URL helpers for the data repository API client."""

from __future__ import annotations

from urllib.parse import urlsplit

__all__ = ["is_local_http"]


def is_local_http(url: str) -> bool:
    """Return whether ``url`` uses HTTP on a loopback host."""

    parsed = urlsplit(url)
    return parsed.scheme == "http" and parsed.hostname in {
        "localhost",
        "127.0.0.1",
        "::1",
    }
