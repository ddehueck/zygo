"""Workflow storage with job, workflow, and cache isolation."""

from __future__ import annotations

import posixpath
from typing import TYPE_CHECKING, assert_never, override

from zygo.cli.v0.types import DataReferenceCreated
from zygo.store._internal.base import BaseStore
from zygo.store._internal.util import partition
from zygo.store.types import DataUri

if TYPE_CHECKING:
    from zygo.cli.v0.transport import IpcTransport
    from zygo.store.types import Scope
    from zygo.workflow.types import JobRunContext


class WorkflowStore(BaseStore):
    """A workflow store, isolated to the current job by default."""

    def __init__(
        self,
        *,
        context: JobRunContext,
        root: DataUri,
        ipc_transport: IpcTransport,
        kwargs: dict[str, str | int | float | bool | None] | None = None,
        scope: Scope = "job",
    ) -> None:
        self._context = context
        self._workflow_root = root
        self._ipc_transport = ipc_transport
        super().__init__(root=self._root_for_scope(scope), kwargs=kwargs)

    def _root_for_scope(self, scope: Scope) -> DataUri:
        root = self._workflow_root
        match scope:
            case "job":
                path = posixpath.join(
                    root.path,
                    partition("wr", self._context.workflow_run_id),
                    partition("jr", self._context.job_run_id),
                )
            case "workflow":
                path = posixpath.join(
                    root.path,
                    partition("wr", self._context.workflow_run_id),
                    "shared",
                )
            case "cache":
                path = posixpath.join(root.path, "cache")
            case _:
                assert_never(scope)
        return DataUri(f"{root.protocol}://{path.rstrip('/')}/")

    def scope(self, scope: Scope) -> WorkflowStore:
        """Return a new store for this scope, without changing the current store."""
        return WorkflowStore(
            context=self._context,
            root=self._workflow_root,
            ipc_transport=self._ipc_transport,
            kwargs=self.kwargs,
            scope=scope,
        )

    @override
    def put(self, key: str | DataUri, data: bytes) -> DataUri:
        uri = super().put(key, data)
        self._ipc_transport.emit(
            DataReferenceCreated(type="data_reference_created", data_reference=str(uri))
        )
        return uri
