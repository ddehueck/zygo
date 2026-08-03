from __future__ import annotations

import ast
import contextlib
from dataclasses import dataclass
import inspect
from pathlib import Path
import pickle  # noqa: S403
import sys
import textwrap
from types import FunctionType, ModuleType

from zygo._internal.utils.hash import hash_bytes, hash_to_str


@dataclass(frozen=True)
class DepGraphResult:
    files: tuple[Path, ...]  # absolute paths
    repo_root: Path  # inferred root
    hash_str: str


def _norm(p: Path) -> Path:
    try:
        return p.resolve()
    except Exception:  # noqa: BLE001
        return Path(str(p)).absolute()


def _find_repo_root(start: Path) -> Path:
    start = _norm(start)
    if start.is_file():
        start = start.parent

    for d in (start, *start.parents):
        if (d / ".git").exists():
            return d

    markers = ("pyproject.toml", "setup.cfg", "setup.py", "requirements.txt")
    for d in (start, *start.parents):
        if any((d / m).exists() for m in markers):
            return d

    d = start
    while (d / "__init__.py").exists() and (d.parent / "__init__.py").exists():
        d = d.parent
    if (d / "__init__.py").exists() and d.parent != d:
        return d.parent

    return start


def _is_under(path: Path, root: Path) -> bool:
    path = _norm(path)
    root = _norm(root)
    try:
        path.relative_to(root)
        return True
    except Exception:  # noqa: BLE001
        return False


def _get_module_file(fn: FunctionType) -> Path:
    """Get the file path for a function's module."""
    mod = sys.modules.get(fn.__module__)
    if mod is None or not getattr(mod, "__file__", None):
        raise ValueError(
            f"Cannot locate module file for {fn} (module={fn.__module__!r})."
        )

    mod_file: str | None = getattr(mod, "__file__", None)
    if mod_file is None:
        raise ValueError(
            f"Cannot locate module file for {fn} (module={fn.__module__!r})."
        )
    return _norm(Path(mod_file))


def _compute_digest(
    value_hashes: list[tuple[str, bytes]],
) -> str:
    """Compute the SHA256 digest over AST-based value hashes (ignores comments)."""
    to_hash = b""

    # Hash the dependency values (closures, globals, function/class ASTs, etc.)
    for key, value_hash in sorted(value_hashes):
        to_hash += key.encode("utf-8")
        to_hash += value_hash

    return hash_to_str(to_hash)


def _cleanup_syspath(repo_root: Path) -> None:
    """Remove repo_root from sys.path if present."""
    repo_str = str(repo_root)
    if sys.path and sys.path[0] == repo_str:
        sys.path.pop(0)
    else:
        with contextlib.suppress(ValueError):
            sys.path.remove(repo_str)


def _get_function_ast(fn: FunctionType) -> ast.FunctionDef | ast.AsyncFunctionDef:
    """Parse and return the AST node for the function definition."""
    unwrapped = inspect.unwrap(fn)  # pyright: ignore[reportAny]
    source = textwrap.dedent(inspect.getsource(unwrapped))  # pyright: ignore[reportAny]
    tree = ast.parse(source)
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return node
    raise ValueError(f"Could not find function definition AST for {fn}")


def _extract_used_names(node: ast.AST) -> set[str]:
    """Extract all names that are loaded (used) in an AST node."""
    used: set[str] = set()
    for child in ast.walk(node):
        if isinstance(child, ast.Name) and isinstance(child.ctx, ast.Load):
            used.add(child.id)
        # Handle attribute access like module.func - we care about the root name
        elif isinstance(child, ast.Attribute):
            current: ast.expr = child.value
            while isinstance(current, ast.Attribute):
                current = current.value
            if isinstance(current, ast.Name) and isinstance(current.ctx, ast.Load):
                used.add(current.id)
    return used


def _get_closure_values(fn: FunctionType) -> dict[str, object]:
    """Get closure variable names and their values."""
    closure_vars: dict[str, object] = {}
    freevars = fn.__code__.co_freevars
    closure = fn.__closure__
    if closure is not None:
        for name, cell in zip(freevars, closure, strict=True):
            with contextlib.suppress(ValueError):
                # cell_contents is typed as Any in the stdlib
                closure_vars[name] = cell.cell_contents  # pyright: ignore[reportAny]
    return closure_vars


def _is_stable_value(value: object) -> bool:
    """Check if a value has a stable/deterministic representation."""
    # Primitive types are always stable
    if isinstance(value, (type(None), bool, int, float, str, bytes)):
        return True
    # Container types are stable if all their elements are stable
    if isinstance(value, (tuple, list, set, frozenset)):
        return all(
            _is_stable_value(item)  # pyright: ignore[reportUnknownArgumentType]
            for item in value  # pyright: ignore[reportUnknownVariableType]
        )
    # Dicts are stable if keys and values are stable
    if isinstance(value, dict):
        return all(
            _is_stable_value(k) and _is_stable_value(v)  # pyright: ignore[reportUnknownArgumentType]
            for k, v in value.items()  # pyright: ignore[reportUnknownVariableType]
        )
    # Other objects (custom classes, etc.) are not considered stable
    # as they may contain object IDs, timestamps, etc.
    return False


def _hash_value(value: object) -> bytes | None:
    """
    Hash a Python value for inclusion in the digest.
    Returns None if the value cannot be stably hashed.
    """
    # Only hash values that have stable representations
    if not _is_stable_value(value):
        return None
    try:
        # Pickle should be stable for these simple types
        pickled = pickle.dumps(value, protocol=pickle.HIGHEST_PROTOCOL)
        return hash_bytes(pickled)
    except Exception:  # noqa: BLE001
        return None


def _pyc_to_py(path: Path) -> Path:
    """Convert a .pyc path to its corresponding .py path."""
    # Handle __pycache__/*.pyc files
    if path.suffix == ".pyc" and path.parent.name == "__pycache__":
        # e.g., __pycache__/foo.cpython-314.pyc -> foo.py
        stem = path.stem
        # Remove the .cpython-XXX suffix
        if ".cpython-" in stem:
            stem = stem.split(".cpython-")[0]
        return path.parent.parent / f"{stem}.py"
    # Handle legacy .pyc files (same directory as .py)
    if path.suffix == ".pyc":
        return path.with_suffix(".py")
    return path


def _source_to_ast_hash_bytes(source: str) -> bytes:
    """Hash source code by its AST, ignoring comments and formatting."""
    tree = ast.parse(source)
    # ast.dump without attributes strips line numbers and formatting
    dumped = ast.dump(tree, include_attributes=False)
    return hash_bytes(dumped.encode("utf-8"))


def _get_object_source_hash_bytes(
    obj: FunctionType | type, repo_root: Path
) -> tuple[Path | None, bytes | None]:
    """
    Get the source file and hash for an object (function, class, etc).
    Returns (file_path, source_hash) if it's a local object, (None, None) otherwise.
    The hash is based on the AST, ignoring comments and formatting.
    """
    try:
        if isinstance(obj, FunctionType):
            # inspect.unwrap returns Any in the stdlib
            unwrapped = inspect.unwrap(obj)  # pyright: ignore[reportAny]
            source = textwrap.dedent(inspect.getsource(unwrapped))  # pyright: ignore[reportAny]
            file_path = Path(inspect.getfile(unwrapped))  # pyright: ignore[reportAny]
        else:
            source = textwrap.dedent(inspect.getsource(obj))
            file_path = Path(inspect.getfile(obj))

        # Convert .pyc to .py to ensure stable hashing
        file_path = _pyc_to_py(file_path)
        file_path = _norm(file_path)

        # Only include .py files
        if file_path.suffix != ".py":
            return None, None

        if not _is_under(file_path, repo_root):
            return None, None

        return file_path, _source_to_ast_hash_bytes(source)
    except (TypeError, OSError, SyntaxError):
        return None, None


def _resolve_name_value(
    name: str,
    closure_values: dict[str, object],
    fn_globals: dict[str, object],
) -> tuple[object, str] | None:
    """Resolve a name to its value and source (closure or global)."""
    if name in closure_values:
        return closure_values[name], "closure"
    if name in fn_globals:
        return fn_globals[name], "global"
    return None


def _process_module_value(
    value: ModuleType,
    value_key: str,
    repo_root: Path,
    included_files: set[Path],
    value_hashes: list[tuple[str, bytes]],
) -> None:
    """Process a module value, adding its file if local."""
    mod_file: str | None = getattr(value, "__file__", None)
    if mod_file is None:
        return
    mod_path = _norm(Path(mod_file))
    if _is_under(mod_path, repo_root) and mod_path.suffix == ".py":
        included_files.add(mod_path)
        value_hashes.append((f"{value_key}:module", hash_bytes(mod_path.read_bytes())))


class _DepCollector:
    """Helper class to collect function dependencies."""

    def __init__(self, repo_root: Path, max_depth: int) -> None:
        super().__init__()
        self.repo_root = repo_root
        self.max_depth = max_depth
        self.visited: set[int] = set()
        self.included_files: set[Path] = set()
        self.value_hashes: list[tuple[str, bytes]] = []

    def _add_source(self, obj: FunctionType | type, prefix: str) -> None:
        file_path, source_hash = _get_object_source_hash_bytes(obj, self.repo_root)
        if file_path is not None:
            self.included_files.add(file_path)
        if source_hash is not None:
            self.value_hashes.append((f"{prefix}:source", source_hash))

    def _process_value(self, value: object, value_key: str, depth: int) -> None:
        if isinstance(value, (FunctionType, type)):
            self.process(value, value_key, depth + 1)
        elif isinstance(value, ModuleType):
            _process_module_value(
                value, value_key, self.repo_root, self.included_files, self.value_hashes
            )
        elif not callable(value):
            value_hash = _hash_value(value)
            if value_hash is not None:
                self.value_hashes.append((value_key, value_hash))

    def process(self, obj: FunctionType | type, prefix: str, depth: int = 0) -> None:
        if depth > self.max_depth or id(obj) in self.visited:
            return
        self.visited.add(id(obj))
        self._add_source(obj, prefix)

        if not isinstance(obj, FunctionType):
            return

        try:
            func_ast = _get_function_ast(obj)
        except Exception:  # noqa: BLE001
            return

        for name in sorted(_extract_used_names(func_ast)):
            resolved = _resolve_name_value(
                name, _get_closure_values(obj), obj.__globals__
            )
            if resolved is None or resolved[0] is None:
                continue
            value, source = resolved
            self._process_value(value, f"{prefix}.{source}.{name}", depth)


def _collect_function_dependencies(
    fn: FunctionType,
    repo_root: Path,
    max_depth: int = 100,
) -> tuple[set[Path], list[tuple[str, bytes]]]:
    """
    Collect dependencies for a specific function by analyzing its AST.

    Returns:
        - Set of local file paths that contain used code
        - List of (key, hash) tuples for values that should be included in digest
    """
    collector = _DepCollector(repo_root, max_depth)
    collector.process(fn, f"{fn.__module__}.{fn.__qualname__}")
    return collector.included_files, collector.value_hashes


def local_source_dependency_hash(
    fn: FunctionType,
    *,
    max_files: int = 10_000,
    max_depth: int = 100,
) -> DepGraphResult:
    """
    Compute a hash of a function based on its actual dependencies.

    This analyzes the function's AST to determine which names it uses,
    then traces those through closures and globals to find:
    - Local source files containing used functions/classes (hashed by AST, ignoring comments)
    - Values of closure variables
    - Values of global variables used

    Args:
        fn: The function to hash.
        max_files: Maximum number of files to include (safety limit).
        max_depth: Maximum recursion depth for dependency analysis.

    Returns:
        DepGraphResult with files included and the computed digest.
    """
    start_file = _get_module_file(fn)
    repo_root = _find_repo_root(start_file)

    added_to_syspath = str(repo_root) not in sys.path
    if added_to_syspath:
        sys.path.insert(0, str(repo_root))

    try:
        # Collect dependencies specific to this function
        included, value_hashes = _collect_function_dependencies(
            fn, repo_root, max_depth=max_depth
        )

        if len(included) > max_files:
            raise RuntimeError(
                f"Exceeded max_files={max_files}; possible dependency explosion."
            )

        rels = sorted(
            str(p.relative_to(repo_root)).replace("\\", "/") for p in included
        )
        digest = _compute_digest(value_hashes)

        return DepGraphResult(
            files=tuple(repo_root / r for r in rels),
            repo_root=repo_root,
            hash_str=digest,
        )
    finally:
        if added_to_syspath:
            _cleanup_syspath(repo_root)
