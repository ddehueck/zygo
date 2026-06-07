from pathlib import Path
import sys


def caller_module_path(*, stack_offset: int = 1) -> Path:
    """Return the resolved file path of the calling module.

    *stack_offset* controls how many frames to skip beyond the direct
    caller.  The default (1) gives the file of whoever called the
    function that called ``caller_module_path``.
    """
    # +1 because _getframe(0) is *this* function.
    frame = sys._getframe(stack_offset + 1)
    return Path(frame.f_code.co_filename).resolve()
