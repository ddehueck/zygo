"""Tests for function hashing."""

from collections.abc import Callable
from types import FunctionType
from typing import cast

from zygo._internal.python.fn_hash import local_source_dependency_hash


def test_basic_function_hash() -> None:
    """Basic function produces a stable hash."""

    def simple_fn(x: int) -> int:
        return x * 2

    result1 = local_source_dependency_hash(simple_fn)
    result2 = local_source_dependency_hash(simple_fn)

    assert result1.hash_str == result2.hash_str


def test_different_functions_different_hash() -> None:
    """Different functions produce different hashes."""

    def fn_add(x: int) -> int:
        return x + 1

    def fn_sub(x: int) -> int:
        return x - 1

    result_add = local_source_dependency_hash(fn_add)
    result_sub = local_source_dependency_hash(fn_sub)

    assert result_add.hash_str != result_sub.hash_str


def test_closure_different_values_different_hash() -> None:
    """Functions with different closure values produce different hashes."""

    def make_adder(n: int) -> FunctionType:
        def adder(x: int) -> int:
            return x + n

        return adder

    add5 = make_adder(5)
    add10 = make_adder(10)

    result5 = local_source_dependency_hash(add5)
    result10 = local_source_dependency_hash(add10)

    assert result5.hash_str != result10.hash_str


def test_closure_same_value_same_hash() -> None:
    """Functions with same closure values produce same hashes."""

    def make_multiplier(n: int) -> FunctionType:
        def mult(x: int) -> int:
            return x * n

        return mult

    mult_a = make_multiplier(7)
    mult_b = make_multiplier(7)

    result_a = local_source_dependency_hash(mult_a)
    result_b = local_source_dependency_hash(mult_b)

    assert result_a.hash_str == result_b.hash_str


def test_global_value_affects_hash() -> None:
    """Global values referenced by the function are included in the hash."""
    MULTIPLIER = 5  # noqa: N806

    def compute(x: int) -> int:
        return x * MULTIPLIER

    result1 = local_source_dependency_hash(compute)

    # The hash should be stable for same global value
    result2 = local_source_dependency_hash(compute)
    assert result1.hash_str == result2.hash_str


def test_variable_names_affect_hash() -> None:
    """Different variable names produce different hashes (AST-sensitive)."""

    def fn_with_a(x: int) -> int:
        a = x * 2
        a += 0  # Use variable to avoid lint warning
        return a

    def fn_with_b(x: int) -> int:
        b = x * 2
        b += 0  # Use variable to avoid lint warning
        return b

    result_a = local_source_dependency_hash(fn_with_a)
    result_b = local_source_dependency_hash(fn_with_b)

    # Different variable names = different AST = different hash
    assert result_a.hash_str != result_b.hash_str


def test_nested_function_dependency() -> None:
    """Changing a helper function changes the outer function's hash."""

    def helper_v1(x: int) -> int:
        return x * 2

    def helper_v2(x: int) -> int:
        return x * 3  # Different implementation

    def make_outer(helper: Callable[[int], int]) -> Callable[[int], int]:
        def outer(x: int) -> int:
            return helper(x) + 1

        return outer

    outer_v1 = make_outer(helper_v1)
    outer_v2 = make_outer(helper_v2)

    result_v1 = local_source_dependency_hash(cast("FunctionType", outer_v1))
    result_v2 = local_source_dependency_hash(cast("FunctionType", outer_v2))

    # Different helper = different hash for outer
    assert result_v1.hash_str != result_v2.hash_str

    # Same helper = same hash for outer
    outer_v1_equal = make_outer(helper_v1)

    result_v1_equal = local_source_dependency_hash(cast("FunctionType", outer_v1_equal))
    assert result_v1_equal.hash_str == result_v1.hash_str
