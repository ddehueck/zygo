from __future__ import annotations

from typing import Any, override

from zygo._internal.fsspec import FsspecUri
from zygo.backends.protocol import (
    Backend,
)
from zygo.store import StoreOptions


class DefaultBackend(Backend):
    """
    The default backend for local development and single-machine workflows.

    Store:
        Accepts any fsspec-compatible URI as ``store_uri``, including local
        paths (``file://``, ``memory://``) and remote object stores
        (``s3://``, ``gs://``, etc.).  Local filesystem stores are permitted
        (``allow_local_store = True``).

    Execution:
        All jobs run on the **local machine**.  ``deploy()`` returns a
        ``LocalEntrypoint`` for every job — no remote infrastructure is
        provisioned. The working directory is the workflow's project root.
        The exec command is resolved from each job's ``Environment`` using
        either its ``uv_lock`` or package list.

    The module path (absolute path to the workflow's Python module) is
    supplied by the CLI at ``deploy()`` time. The project root
    (used as cwd for local execution) is derived as ``module_path.parent``.

    Example:
        ```python
        workflow = Workflow(id="dev")
        ...

        backend = DefaultBackend(store_uri="file://data/")
        workflow.run(backend=backend)
        ```
    """

    def __init__(
        self,
        *,
        store_uri: str,
        store_options: dict[str, Any] | None = None,  # pyright: ignore[reportExplicitAny]
    ) -> None:
        self._store_uri = FsspecUri(store_uri)
        self._store_options = store_options or {}
        super().__init__()

    @property
    @override
    def store_options(self) -> StoreOptions:
        return StoreOptions(root_uri=self._store_uri, kwargs=dict(self._store_options))

    @property
    @override
    def allow_local_store(self) -> bool:
        return True
