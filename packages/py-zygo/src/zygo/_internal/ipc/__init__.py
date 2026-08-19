"""
This module provides IPC utilities for the zygo workflow engine.

There are two main components required by the workflow engine:
    - Retrieving workflow metadata
    - Executing workflow jobs

While the main components should stay consistent the objects may
need to be updated as the engine evolves.
In anticipation of this evolution we version the IPC module interface.

The workflow engine will call out to this module via a command line interface
defined in the ipc.vXYZ.__main__ module.

This interface follows the form of:
`python -m zygo._internal.ipc.version <command> <users_workflow_module>`

For example:
`python -m zygo._internal.ipc.v0 meta examples.main:workflow

python -m zygo._internal.ipc.v0 run examples.main:workflow \
  --args '{...}'
`
"""

from zygo._internal.ipc.importer import load_workflow

__all__ = ["load_workflow"]
