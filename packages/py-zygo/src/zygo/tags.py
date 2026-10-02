from typing import Protocol


class TagsProtocol(Protocol):
    """Manage filterable tags across Zygo contexts.

    A tag is an alpha-numeric string that can only contain the characters
    `a-z`, `A-Z`, `0-9`, `_`, and `-`.
    """

    def add(self, value: str) -> None: ...
