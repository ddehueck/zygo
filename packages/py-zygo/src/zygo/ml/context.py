from __future__ import annotations

from typing import TYPE_CHECKING

from zygo.ml._store import TrainingStore
from zygo.ml.bundle import ModelBundle
from zygo.store._internal.impl import StoreImpl
from zygo.types import JobRunContext, JobRunId, WorkflowRunId

if TYPE_CHECKING:
    from collections.abc import Sequence

    from zygo.cli.v0.types import IpcMessage
    from zygo.store import DataUri, StoreProtocol


class TrainingContext:
    """Runtime-injected storage for one training execution at a time."""

    def __init__(self, *, store: StoreProtocol, root: DataUri | None = None) -> None:
        super().__init__()
        if root is None:
            if not isinstance(store, StoreImpl):
                raise ValueError(
                    "A custom training store requires an explicit root prefix"
                )
            root = store.training_root
        ModelBundle(root)
        self._store = store
        self._root = root

    def store(self) -> TrainingStore:
        """Access the runtime-selected training directory.

        Relative job keys are resolved under the injected root. Other scopes
        and explicit URIs retain the underlying generic store's behavior.
        """
        return TrainingStore(self._store, self._root)

    @classmethod
    def local(cls, root: DataUri) -> TrainingContext:
        """Use exactly this file:// prefix, including artifacts from earlier runs."""

        ModelBundle(root)
        if root.protocol != "file":
            raise ValueError("TrainingContext.local() requires a file:// prefix")
        store = StoreImpl(
            context=JobRunContext(
                workflow_run_id=WorkflowRunId("training"),
                job_run_id=JobRunId("training"),
                input=root,
            ),
            root=root,
            ipc_transport=_LocalTransport(),
        )
        return cls(store=store, root=root)


class _LocalTransport:
    """Local training has no orchestrator to notify about artifact writes."""

    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None:
        pass
