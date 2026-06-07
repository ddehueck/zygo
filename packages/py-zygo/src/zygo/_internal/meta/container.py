from __future__ import annotations

from typing import TYPE_CHECKING, assert_never, override

from zygo._internal import grpc
from zygo._internal.meta.dependencies import (
    DependsMarker,
    InputMarker,
    OutputMarker,
    Publisher,
    Store,
)
from zygo._internal.meta.errors import DIError
from zygo.store._internal.impl import StoreImpl

if TYPE_CHECKING:
    from zygo._internal.grpc.client import OrchestratorClient
    from zygo._internal.meta.dependencies import Dependendable
    from zygo.store import Reference, StoreOptions, StoreProtocol
    from zygo.types import ChannelId, ChannelName, JobRunContext, RunEventContext


class RunContainer:
    """
    A simple dependency injection container to resolve dependencies at runtime for a workflow job.
    """

    def __init__(
        self,
        *,
        context: JobRunContext,
        store_options: StoreOptions,
        client: OrchestratorClient,
        event_context: RunEventContext,
        channel_ids_by_name: dict[ChannelName, ChannelId],
    ) -> None:
        super().__init__()
        self._context = context
        self._store_options = store_options
        self._client = client
        self._event_context = event_context
        self._channel_ids_by_name = channel_ids_by_name

    def resolve(
        self, dependency: Dependendable
    ) -> StoreProtocol | Reference | Publisher:
        """
        Resolve a dependency by its token.

        Args:
            token: Either the Store type or a Channel instance.

        Returns:
            The registered dependency for the given token.

        Raises:
            DIError: If the token is not registered in the container.
        """
        match dependency:
            case DependsMarker():
                if dependency.token is Store:
                    return StoreImpl(context=self._context, options=self._store_options)
                raise DIError(f"Unknown dependency token: {dependency!r}")
            case InputMarker():
                return self._context.data_ref
            case OutputMarker():
                channel_id = self._channel_ids_by_name[dependency.channel.name]
                return PublisherImpl(
                    client=self._client,
                    event_context=self._event_context,
                    channel_id=channel_id,
                )
            case _:
                assert_never(dependency)


class PublisherImpl(Publisher):
    """A class that can publish data to a channel."""

    def __init__(
        self,
        client: OrchestratorClient,
        event_context: RunEventContext,
        channel_id: str,
    ) -> None:
        super().__init__()
        self._client = client
        self._event_context = event_context
        self._channel_id = channel_id

    @override
    def publish(self, data: Reference) -> None:
        self._client.emit(
            context=self._event_context,
            event=grpc.ChannelItemInserted(
                channel_id=self._channel_id,
                data_reference=grpc.DataReference.from_store_ref(data),
            ),
        )
