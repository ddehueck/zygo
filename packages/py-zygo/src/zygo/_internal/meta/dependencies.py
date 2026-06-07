"""
Dependecies are used to instruct the framework what constructs a job needs.
This is modeled after FastAPI's dependency injection system.

The framework will automatically inject the dependencies into the job function.
A user simply declares the dependencies they need and the framework will handle the rest.

Example:

```python
@workflow.job
def my_job(store: Store = Depends(Store), input: Input = Input(channel)) -> None:
    ...
```

The most correct typing is to use Annotated[T, Depends(Store)] or Annotated[T, Input(channel)] or Annotated[T, Output(channel)].
e.g.

```python
def my_job(store: Annotated[Store, Depends(Store)], input: Annotated[DataRef, Input(channel)]) -> None:
    ...
```
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from zygo.store import StoreProtocol

if TYPE_CHECKING:
    from zygo.channel import Channel
    from zygo.store import Reference


@dataclass(frozen=True)
class Store(StoreProtocol):
    """Marker class for Store dependency injection.
    Used as a token in Depends(Store) to request a StoreProtocol implementation.
    """


class Publisher:
    """A class that can publish data to a channel."""

    def publish(self, data: Reference) -> None: ...


DependencyToken = type[Store]


@dataclass(frozen=True)
class DependsMarker:
    token: DependencyToken


@dataclass(frozen=True)
class InputMarker:
    channel: Channel


@dataclass(frozen=True)
class OutputMarker:
    channel: Channel


Dependendable = DependsMarker | InputMarker | OutputMarker


def Depends(dependency: DependencyToken) -> DependsMarker:  # noqa: N802
    return DependsMarker(dependency)


def Input(channel: Channel) -> InputMarker:  # noqa: N802
    return InputMarker(channel)


def Output(channel: Channel) -> OutputMarker:  # noqa: N802
    return OutputMarker(channel)
