from __future__ import annotations

from collections.abc import Callable
from inspect import Parameter, Signature, signature
from typing import TYPE_CHECKING, cast, get_args, get_origin, get_type_hints

from zygo.dataset.dataset import Dataset
from zygo.dataset.features import Features
from zygo.ml.context import TrainingContext
from zygo.ml.hyperparams import HyperParams
from zygo.ml.store import ModelStore

if TYPE_CHECKING:
    from collections.abc import Mapping


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
        self._load: Callable[[ModelStore], object] | None = None
        self._infer: Callable[..., object] | None = None
        self._features: type[Features] | None = None
        self._params_type: type[HyperParams] | None = None
        self._model_type: type[object] | None = None

    def train[F: Callable[..., None]](self, fn: F) -> F:
        """Register a dataset, optional HyperParams, and TrainingContext hook.

        Declare hyperparameters as ``*, params: MyParams`` where MyParams
        subclasses HyperParams. Omitted execution inputs use field defaults.
        The dataset location and training store are supplied at execution.
        Persist artifacts through ctx.store without returning a bundle.
        """
        if self._train is not None:
            raise ValueError("A training function is already registered")
        parameters, hints = _annotations(fn)
        if len(parameters) not in {2, 3}:
            raise TypeError(
                "Training requires a dataset, keyword-only TrainingContext, and optionally keyword-only HyperParams"
            )
        dataset_parameter = parameters[0]
        _require_positional(dataset_parameter, role="Training dataset")
        annotation = hints.get(dataset_parameter.name)
        features = _dataset_features(annotation)
        if hints.get("return") is not type(None):
            raise TypeError("Training must return None")
        keyword_parameters = {parameter.name: parameter for parameter in parameters[1:]}
        context_parameter = keyword_parameters.get("ctx")
        if context_parameter is None:
            raise TypeError(
                "Training context must be declared as '*, ctx: TrainingContext'"
            )
        _require_context(context_parameter, hints)
        params_type = None
        if keyword_parameters.keys() != {"ctx"}:
            params_type = _hyperparams_type(keyword_parameters.get("params"), hints)
        self._params_type = params_type
        self._features = features
        self._train = fn
        return fn

    def run_train[T](
        self,
        dataset: Dataset[T],
        *,
        ctx: TrainingContext,
        params: HyperParams | Mapping[str, object] | None = None,
    ) -> None:
        """Validate training inputs and invoke the registered training hook.

        Parameters may be a declared HyperParams instance or a mapping.
        When omitted, the declared type is constructed from field defaults.
        """
        if self._train is None:
            raise ValueError("No training function is registered")
        training_params = self._validate_params(params)
        training_dataset = (
            dataset.with_features(self._features)
            if self._features is not None
            else dataset
        )
        kwargs: dict[str, object] = {"ctx": ctx}
        if training_params is not None:
            kwargs["params"] = training_params
        result = cast("Callable[..., object]", self._train)(training_dataset, **kwargs)
        if result is not None:
            raise TypeError("Training returned a value that is not None")

    def _validate_params(
        self, params: HyperParams | Mapping[str, object] | None
    ) -> HyperParams | None:
        if self._params_type is None:
            if params is not None:
                raise TypeError("The training hook does not declare hyperparameters")
            return None
        if params is None:
            return self._params_type()
        if isinstance(params, HyperParams) and not isinstance(
            params, self._params_type
        ):
            raise TypeError(
                f"Training parameters must be an instance of {self._params_type.__name__} or a mapping"
            )
        return self._params_type.model_validate(params)

    def load[F: Callable[..., object]](self, fn: F) -> F:
        """Register ``(store: ModelStore)`` returning a live model for inference.

        The runtime selects the model store. Read artifacts through get()
        or open() without needing to enter the store itself as a context.
        """
        if self._load is not None:
            raise ValueError("A load function is already registered")
        parameters, hints = _annotations(fn)
        if len(parameters) != 1:
            raise TypeError("Loading requires exactly one ModelStore parameter")
        parameter = parameters[0]
        _require_positional(parameter, role="Load store")
        if hints.get(parameter.name) is not ModelStore:
            raise TypeError("The load parameter must be annotated as ModelStore")
        model_type = hints.get("return")
        if not isinstance(model_type, type):
            raise TypeError("Loading must declare a concrete model return type")
        self._model_type = cast("type[object]", model_type)
        self._load = fn
        return fn

    def run_load(self, store: ModelStore) -> object:
        """Load a live model from its artifact store."""
        if self._load is None:
            raise ValueError("No loading function is registered")
        model = self._load(store)
        if self._model_type is not None and not isinstance(model, self._model_type):
            raise TypeError(
                "Loading returned a model that does not match its annotation"
            )
        return model

    def infer[F: Callable[..., object]](self, fn: F) -> F:
        """Register inference and check its model annotation against load's return.

        Register load first. The annotations must match exactly, and an
        incompatible annotation raises TypeError during registration.
        """
        if self._infer is not None:
            raise ValueError("An inference function is already registered")
        if self._load is None:
            raise ValueError(
                "Register load before infer so its model type can be checked"
            )
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

    def run_infer(self, model: object, data: object) -> object:
        """Invoke the registered inference hook with a loaded model and input."""
        if self._infer is None:
            raise ValueError("No inference function is registered")
        return self._infer(model, data)


def _annotations(
    fn: Callable[..., object],
) -> tuple[tuple[Parameter, ...], dict[str, object]]:
    try:
        hints = cast("dict[str, object]", get_type_hints(fn))
    except (NameError, TypeError) as error:
        raise TypeError(
            f"Could not resolve ML function annotations: {error}"
        ) from error
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
        raise TypeError(
            "The training input must be annotated as Dataset or Dataset[Features]"
        )
    arguments = cast("tuple[object, ...]", get_args(annotation))
    if len(arguments) != 1:
        raise TypeError("Dataset requires one feature type")
    features = arguments[0]
    if not isinstance(features, type) or not issubclass(features, Features):
        raise TypeError(
            "The training dataset's type argument must be a Features subclass"
        )
    return features


def _hyperparams_type(
    parameter: Parameter | None, hints: dict[str, object]
) -> type[HyperParams]:
    if (
        parameter is None
        or parameter.kind is not Parameter.KEYWORD_ONLY
        or cast("object", parameter.default) is not Signature.empty
    ):
        raise TypeError(
            "Training hyperparameters must be declared as '*, params: HyperParamsSubclass'"
        )
    annotation = hints.get("params")
    if not isinstance(annotation, type) or not issubclass(annotation, HyperParams):
        raise TypeError("Training params must be annotated as a HyperParams subclass")
    return annotation


def _require_context(parameter: Parameter, hints: dict[str, object]) -> None:
    if (
        parameter.name != "ctx"
        or parameter.kind is not Parameter.KEYWORD_ONLY
        or cast("object", parameter.default) is not Signature.empty
        or hints.get("ctx") is not TrainingContext
    ):
        raise TypeError(
            "Training context must be declared as '*, ctx: TrainingContext'"
        )
