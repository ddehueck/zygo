"""Workflow storage with independent job, workflow, and cache scopes."""

from __future__ import annotations

from dataclasses import dataclass
import posixpath
from typing import TYPE_CHECKING, assert_never

from zygo.cli.v0.types import StoreConfig
from zygo.store._internal.base import BaseStore
from zygo.store._internal.util import partition
from zygo.store.types import DataUri

if TYPE_CHECKING:
    from zygo.cli.v0.transport import IpcTransport
    from zygo.store.types import Scope
    from zygo.workflow.types import JobRunContext


@dataclass(frozen=True)
class WorkflowStoreConfig:
    job: StoreConfig
    workflow: StoreConfig
    cache: StoreConfig


class WorkflowStore(BaseStore):
    """A workflow store, isolated to the current job by default."""

    def __init__(
        self,
        *,
        context: JobRunContext,
        config: WorkflowStoreConfig,
        ipc_transport: IpcTransport,
        scope: Scope = "job",
    ) -> None:
        self._context = context
        self._config = config

        scope_config = self._config_for_scope(scope)
        super().__init__(
            root=self._root_for_scope(scope, scope_config),
            ipc_transport=ipc_transport,
            kwargs=scope_config.kwargs,
        )

    def _config_for_scope(self, scope: Scope) -> StoreConfig:
        match scope:
            case "job":
                return self._config.job
            case "workflow":
                return self._config.workflow
            case "cache":
                return self._config.cache
            case _:
                assert_never(scope)

    def _root_for_scope(self, scope: Scope, config: StoreConfig) -> DataUri:
        root = DataUri(config.root_uri)
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
                )
            case "cache":
                path = root.path
            case _:
                assert_never(scope)
        return DataUri(f"{root.protocol}://{path.rstrip('/')}/")

    def scope(self, scope: Scope) -> WorkflowStore:
        """Return a new store for this scope, without changing the current store."""
        return WorkflowStore(
            context=self._context,
            config=self._config,
            ipc_transport=self._ipc_transport,
            scope=scope,
        )
