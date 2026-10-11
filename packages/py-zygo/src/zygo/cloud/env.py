"""Environment variables Zygo Cloud reads."""

import os

API_HOST_ENV_VAR = "ZYGO_CLOUD_API_HOST"
API_KEY_ENV_VAR = "ZYGO_CLOUD_API_KEY"

_DEFAULT_API_HOST = "https://zygo.sh"


def api_host() -> str:
    """Return the Zygo Cloud API host, with no trailing slash."""
    return os.environ.get(API_HOST_ENV_VAR, _DEFAULT_API_HOST).rstrip("/")


def api_key() -> str | None:
    """Return the Zygo Cloud API key."""
    return os.environ.get(API_KEY_ENV_VAR)
