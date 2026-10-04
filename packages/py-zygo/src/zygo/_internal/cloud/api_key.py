"""
zygo.cloud API key utilities.
"""

import os

_DEFAULT_API_KEY_ENV_VAR = "ZYGO_CLOUD_API_KEY"


def load_api_key() -> str | None:
    """Load the API key from ENV var or .env file."""
    return os.environ.get(_DEFAULT_API_KEY_ENV_VAR, None)
