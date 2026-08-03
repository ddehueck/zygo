from __future__ import annotations

from typing import TYPE_CHECKING, assert_never, override

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
    from zygo._internal.meta.dependencies import Dependendable
    from zygo.store import Reference, StoreOptions, StoreProtocol
    from zygo.types import ChannelId, JobRunContext


class RunContainer:
    """
    A simple dependency injection container to resolve dependencies at runtime for a workflow job.
    """

    def __init__(
        self,
        *,
        context: JobRunContext,
        store_options: StoreOptions,
    ) -> None:
        super().__init__()
        self._context = context
        self._store_options = store_options

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
                return PublisherImpl(channel_id=dependency.channel.id)
            case _:
                assert_never(dependency)


class PublisherImpl(Publisher):
    """A class that can publish data to a channel."""

    def __init__(self, channel_id: ChannelId) -> None:
        super().__init__()
        self._channel_id = channel_id

    @override
    def publish(self, data: Reference) -> None:
        del data
        raise RuntimeError(
            f"Publishing to channel {self._channel_id} is unavailable until a runtime "
            + "transport is configured"
        )
