"""Shared logger for Zygo CLI commands."""

from __future__ import annotations

import logging
import sys

_zygo_logger = logging.getLogger("zygo.cli")


def _prepend_zygo(record: logging.LogRecord) -> bool:
    record.msg = f"[zygo] {record.msg}"
    return True


_zygo_logger.addFilter(_prepend_zygo)


def configure_cli_logging() -> None:
    """Show CLI info logs on stderr without duplicating them on the root logger."""
    if _zygo_logger.handlers:
        return
    handler = logging.StreamHandler(sys.stderr)
    handler.setFormatter(logging.Formatter("%(message)s"))
    _zygo_logger.addHandler(handler)
    _zygo_logger.setLevel(logging.INFO)
    _zygo_logger.propagate = False
