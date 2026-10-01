from __future__ import annotations

from collections.abc import Callable, Mapping
from inspect import Parameter, Signature, signature
from typing import cast, get_args, get_origin, get_type_hints

from zygo.ml._store import TrainingStore
from zygo.ml.context import TrainingContext
from zygo.ml.dataset import Dataset
from zygo.ml.features import Features
from zygo.store import DataUri


class Model:
    """Register framework-independent training, loading, and inference hooks.

    Use ``Model(name)`` without declaring a model type. When inference is
    registered, Zygo checks that its first parameter annotation matches the
    load return annotation. This linkage is validated at runtime, not by type
    checkers. Decorators retain the functions' individual static signatures.
    Execution methods are entry points for a local caller or future runtime,
    not an orchestrator, artifact publisher, or framework-specific trainer.
    """

    def __init__(self, name: str) -> None:
        super().__init__()
        if not name:
            raise ValueError("Model name must not be empty")
        self.name = name
        self._train: Callable[..., None] | None = None
        self._load: Callable[[TrainingStore], object] | None = None
        self._infer: Callable[..., object] | None = None
        self._features: type[Features] | None = None
        self._model_type: type[object] | None = None


    def train[F: Callable[..., None]](self, fn: F) -> F:
        """Register ``(dataset: Dataset[Features], *, ctx: TrainingContext) -> None``.

        The dataset location and training store are supplied at execution,
        rather than captured in the model definition. Persist artifacts through
        ctx.store() without returning a bundle.
        """
        if self._train is not None:
            raise ValueError("A training function is already registered")
        parameters, hints = _annotations(fn)
        if len(parameters) != 2:
            raise TypeError("Training requires a dataset and keyword-only TrainingContext")
        dataset_parameter = parameters[0]
        _require_positional(dataset_parameter, role="Training dataset")
        annotation = hints.get(dataset_parameter.name)
        features = _dataset_features(annotation)
        if hints.get("return") is not type(None):
            raise TypeError("Training must return None")
        _require_context(parameters[1], hints)
        self._features = features
        self._train = fn
        return fn

    def load[F: Callable[..., object]](self, fn: F) -> F:
        """Register ``(store: TrainingStore)`` returning a live model for inference.

        The runtime selects the artifact store. Read artifacts through get()
        or open() without needing to enter the store itself as a context.
        """
        if self._load is not None:
            raise ValueError("A load function is already registered")
        parameters, hints = _annotations(fn)
        if len(parameters) != 1:
            raise TypeError("Loading requires exactly one TrainingStore parameter")
        parameter = parameters[0]
        _require_positional(parameter, role="Load store")
        if hints.get(parameter.name) is not TrainingStore:
            raise TypeError("The load parameter must be annotated as TrainingStore")
        model_type = hints.get("return")
        if not isinstance(model_type, type):
            raise TypeError("Loading must declare a concrete model return type")
        self._model_type = cast("type[object]", model_type)
        self._load = fn
        return fn

    def infer[F: Callable[..., object]](self, fn: F) -> F:
        """Register inference and check its model annotation against load's return.

        Register load first. The annotations must match exactly, and an
        incompatible annotation raises TypeError during registration.
        """
        if self._infer is not None:
            raise ValueError("An inference function is already registered")
        if self._load is None:
            raise ValueError("Register load before infer so its model type can be checked")
        parameters, hints = _annotations(fn)
        if not parameters:
            raise TypeError("Inference requires a model as its first parameter")
        parameter = parameters[0]
        _require_positional(parameter, role="Inference model")
        if hints.get(parameter.name) is not self._model_type:
            raise TypeError(
                f"The first inference parameter must match the load return type {self._model_type!r}"
            )
        self._infer = fn
        return fn

    def run_train(
        self,
        dataset: str | DataUri | Dataset[object],
        *,
        ctx: TrainingContext,
        storage_options: Mapping[str, object] | None = None,
    ) -> None:
        """Inject a caller-selected dataset and a training-specific store context.

        Store-produced DataUri references and direct local/cloud locations are
        accepted. The hook persists artifacts through ctx.store() and must
        return None. No bundle result is required or validated.
        """
        if self._train is None:
            raise ValueError("No training function is registered")
        if not isinstance(cast("object", ctx), TrainingContext):
            raise TypeError("Training requires a TrainingContext")
        training_dataset: Dataset[object]
        if isinstance(dataset, Dataset):
            training_dataset = dataset
        else:
            training_dataset = Dataset.open(dataset, storage_options=storage_options)
        if self._features is not None:
            training_dataset = training_dataset.with_features(self._features)

        result = self._train(training_dataset, ctx=ctx)
        if cast("object", result) is not None:
            raise TypeError("Training returned a value that is not None")

    def run_load(self, store: TrainingStore) -> object:
        """Inject a runtime-selected store and runtime-check the loaded model.

        The caller may construct the store with ctx.store(). Reads through
        get() and open() do not require entering the store context. Call the
        decorated load function directly for its concrete static return type.
        This runtime entry point returns object.
        """
        if self._load is None or self._model_type is None:
            raise ValueError("No load function is registered")
        if not isinstance(cast("object", store), TrainingStore):
            raise TypeError("Loading requires a TrainingStore")
        model = self._load(store)
        if not isinstance(model, self._model_type):
            raise TypeError("Loading returned a value incompatible with its return type")
        return model

    def run_infer(self, model: object, /, **inputs: object) -> object:
        """Inject a previously loaded model into the registered inference hook.

        This dynamic runtime boundary accepts named inputs. Calling the
        decorated inference function directly preserves its full static types.
        """
        if self._infer is None or self._model_type is None:
            raise ValueError("No inference function is registered")
        if not isinstance(model, self._model_type):
            raise TypeError("Inference requires a model compatible with load's return type")
        arguments = signature(self._infer).bind(model, **inputs)
        return self._infer(*arguments.args, **arguments.kwargs)


def _annotations(
    fn: Callable[..., object],
) -> tuple[tuple[Parameter, ...], dict[str, object]]:
    try:
        hints = cast("dict[str, object]", get_type_hints(fn))
    except (NameError, TypeError) as error:
        raise TypeError(f"Could not resolve ML function annotations: {error}") from error
    return tuple(signature(fn).parameters.values()), hints


def _require_positional(parameter: Parameter, *, role: str) -> None:
    if parameter.kind not in {
        Parameter.POSITIONAL_ONLY,
        Parameter.POSITIONAL_OR_KEYWORD,
    }:
        raise TypeError(f"{role} must be a positional parameter")
    if cast("object", parameter.default) is not Parameter.empty:
        raise TypeError(f"{role} must not have a default value")


def _dataset_features(annotation: object) -> type[Features] | None:
    if annotation is Dataset:
        return None
    if get_origin(annotation) is not Dataset:
        raise TypeError("The training input must be annotated as Dataset or Dataset[Features]")
    arguments = cast("tuple[object, ...]", get_args(annotation))
    if len(arguments) != 1:
        raise TypeError("Dataset requires one feature type")
    features = arguments[0]
    if not isinstance(features, type) or not issubclass(features, Features):
        raise TypeError("The training dataset's type argument must be a Features subclass")
    return features


def _require_context(parameter: Parameter, hints: dict[str, object]) -> None:
    if (
        parameter.name != "ctx"
        or parameter.kind is not Parameter.KEYWORD_ONLY
        or cast("object", parameter.default) is not Signature.empty
        or hints.get("ctx") is not TrainingContext
    ):
        raise TypeError("Training context must be declared as '*, ctx: TrainingContext'")
