from __future__ import annotations

from typing import TYPE_CHECKING
from uuid import uuid4

from zygo.ml._store import ArtifactStore
from zygo.ml.bundle import ModelBundle
from zygo.store import DataUri, StoreProtocol

if TYPE_CHECKING:
    from collections.abc import Sequence

    from zygo.cli.v0.types import IpcMessage


class TrainingContext:
    """Training-specific access to the existing scoped store.

    Only successful artifact writes made through this context's store can
    back a newly trained bundle. Reads and failed writes do not qualify.
    A context is intended for one training execution at a time.
    """

    def __init__(self, *, store: StoreProtocol) -> None:
        super().__init__()
        self._store = ArtifactStore(store)

    @property
    def store(self) -> StoreProtocol:
        return self._store

    def bundle(self, prefix: DataUri) -> ModelBundle:
        """Create a bundle backed by a successful, still-existing artifact write."""
        bundle = ModelBundle(prefix)
        self.validate_bundle(bundle)
        return bundle

    def begin_training(self) -> None:
        """Reset artifact tracking when the runtime starts a training execution."""
        self._store.writes.clear()

    def validate_bundle(self, bundle: ModelBundle) -> None:
        """Verify that this execution wrote an existing artifact under the prefix."""
        prefix = str(bundle.uri)
        if not any(
            str(uri).startswith(prefix)
            and str(uri) != prefix
            and self.store.exists(str(uri))
            for uri in self._store.writes
        ):
            raise ValueError(
                "A training bundle must contain an artifact successfully written through ctx.store"
            )

    @classmethod
    def local(cls, root: DataUri) -> TrainingContext:
        """Create an isolated local training store without workflow IPC.

        The root must be a file:// prefix. Each context uses a new run ID,
        preserving the underlying workflow store's per-run isolation.
        """
        from zygo.store._internal.impl import StoreImpl
        from zygo.types import JobRunContext, JobRunId, WorkflowRunId

        ModelBundle(root)
        if root.protocol != "file":
            raise ValueError("TrainingContext.local() requires a file:// prefix")
        store = StoreImpl(
            context=JobRunContext(
                workflow_run_id=WorkflowRunId("training"),
                job_run_id=JobRunId(uuid4().hex),
                input=root,
            ),
            root=root,
            ipc_transport=_LocalTransport(),
        )
        return cls(store=store)


class _LocalTransport:
    """Local training has no orchestrator to notify about artifact writes."""

    def emit(self, messages: IpcMessage | Sequence[IpcMessage]) -> None:
        pass
