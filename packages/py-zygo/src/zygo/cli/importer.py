import importlib
from types import ModuleType
from typing import Self, cast


class Importer:
    """Load typed instances from a module or a ``module:attribute.path`` target.

    Types may define ``conventional_names`` as an ordered tuple of module
    attribute names. These take priority over discovery of unnamed instances.
    """

    def __init__(
        self,
        module: ModuleType,
        attribute_path: str | None = None,
    ) -> None:
        super().__init__()
        self.module = module
        self._attribute_path = attribute_path

    @classmethod
    def from_target(cls, target: str) -> Self:
        module_name, separator, attribute_path = target.partition(":")

        if not module_name:
            raise RuntimeError("A Python module is required")
        if separator and not attribute_path:
            raise RuntimeError(f"Expected {module_name}:<instance-name>")

        try:
            module = importlib.import_module(module_name)
        except Exception as exc:
            raise RuntimeError(
                f"Could not import module {module_name!r}: {exc}"
            ) from exc

        return cls(module, attribute_path if separator else None)

    def has_instance[T](self, instance_type: type[T]) -> bool:
        """Check for a matching instance, even if discovery is ambiguous.

        Explicit targets only check the selected attribute. Missing attributes
        and mismatched types return False, while import failures still raise.
        """
        if self._attribute_path is not None:
            try:
                value = self._resolve_attribute()
            except AttributeError:
                return False
            return isinstance(value, instance_type)

        return any(
            isinstance(value, instance_type)
            for value in cast("dict[str, object]", vars(self.module)).values()
        )

    def load[T](self, instance_type: type[T]) -> T:
        """Load an instance, rejecting missing, mismatched, or ambiguous targets."""
        if self._attribute_path is not None:
            try:
                value = self._resolve_attribute()
            except AttributeError as exc:
                raise RuntimeError(
                    f"{self.module.__name__!r} has no attribute {self._attribute_path!r}"
                ) from exc

            if not isinstance(value, instance_type):
                target = f"{self.module.__name__}:{self._attribute_path}"
                raise RuntimeError(
                    f"{target!r} resolved to {type(value).__name__}, not a {instance_type.__module__}.{instance_type.__qualname__}"
                )
            return value

        return self._discover(instance_type)

    def _resolve_attribute(self) -> object:
        value: object = self.module
        for part in (self._attribute_path or "").split("."):
            value = cast("object", getattr(value, part))
        return value

    def _discover[T](self, instance_type: type[T]) -> T:
        namespace = cast("dict[str, object]", vars(self.module))
        conventional_names = cast(
            "tuple[str, ...]", getattr(instance_type, "conventional_names", ())
        )
        for name in conventional_names:
            value = namespace.get(name)
            if isinstance(value, instance_type):
                return value

        matches = [
            (name, value)
            for name, value in namespace.items()
            if isinstance(value, instance_type)
        ]
        if not matches:
            raise RuntimeError(
                f"No {instance_type.__name__} instance found in {self.module.__name__!r}"
            )
        if len(matches) > 1:
            names = ", ".join(name for name, _ in matches)
            raise RuntimeError(
                f"Multiple {instance_type.__name__} instances found in {self.module.__name__!r}: {names}. Select one with {self.module.__name__}:<name>."
            )

        return matches[0][1]
