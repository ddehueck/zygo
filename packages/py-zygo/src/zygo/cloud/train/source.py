"""Choose and zip the files uploaded as the Docker build context.

Selection is a list of exclude rules. The defaults skip tool directories and
honor ``.gitignore``. Add gitignore patterns with ``excludes``, or pass
``filters`` to replace the defaults with your own rules.
"""

from __future__ import annotations

from io import BytesIO
from typing import TYPE_CHECKING, Protocol
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

from pathspec import GitIgnoreSpec

if TYPE_CHECKING:
    from pathlib import Path

_MAX_ARCHIVE_BYTES = 50_000_000
_ZIP_DATE = (1980, 1, 1, 0, 0, 0)
_ZIP_FILE_MODE = 0o100644 << 16
_DEFAULT_EXCLUDED_DIRECTORIES = frozenset({
    ".git",
    ".hg",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".venv",
    "__pycache__",
    "node_modules",
    "venv",
})


class SourceExclude(Protocol):
    """One reason a path might stay out of the archive."""

    def excludes(self, relative: str) -> bool:
        """Return True when ``relative`` should not be uploaded.

        Directories are passed with a trailing slash, such as ``build/``.
        """


class DirectoryExclude:
    """Skip directories with these names anywhere in the tree."""

    def __init__(self, names: frozenset[str]) -> None:
        super().__init__()
        self._names = names

    def excludes(self, relative: str) -> bool:
        return bool(self._names.intersection(_parts(relative)))


class GitignoreExclude:
    """Apply ``.gitignore`` files from the source root downward."""

    def __init__(self, root: Path) -> None:
        super().__init__()
        self._root = root
        self._specs: dict[str, GitIgnoreSpec | None] = {}

    def excludes(self, relative: str) -> bool:
        directory = relative.endswith("/")
        parts = _parts(relative)
        ignored = False
        for index in range(len(parts)):
            spec = self._spec("/".join(parts[:index]))
            if spec is None:
                continue
            rest = "/".join(parts[index:])
            if directory:
                rest = f"{rest}/"
            decision = spec.check_file(rest).include
            if decision is not None:
                ignored = decision
        return ignored

    def _spec(self, directory: str) -> GitIgnoreSpec | None:
        if directory in self._specs:
            return self._specs[directory]
        base = (
            self._root if not directory else self._root.joinpath(*directory.split("/"))
        )
        path = base / ".gitignore"
        if not path.is_file() or path.is_symlink():
            self._specs[directory] = None
            return None
        spec = GitIgnoreSpec.from_lines(
            path.read_text(encoding="utf-8", errors="replace").splitlines()
        )
        self._specs[directory] = spec
        return spec


class PatternExclude:
    """Extra gitignore-style patterns, relative to the source root."""

    def __init__(self, patterns: tuple[str, ...]) -> None:
        super().__init__()
        self._spec = GitIgnoreSpec.from_lines(patterns)

    def excludes(self, relative: str) -> bool:
        return self._spec.check_file(relative).include is True


class SourcePackage:
    """The files from a source directory that will be uploaded."""

    def __init__(
        self,
        root: Path,
        *,
        excludes: tuple[str, ...] = (),
        filters: tuple[SourceExclude, ...] | None = None,
    ) -> None:
        super().__init__()
        if not root.is_dir():
            raise ValueError(f"Source directory is not a directory: {root}")
        self.root = root
        chosen = list(default_filters(root) if filters is None else filters)
        if excludes:
            chosen.append(PatternExclude(excludes))
        self._filters = tuple(chosen)
        self._selected: tuple[list[Path], list[str]] | None = None

    def files(self) -> list[Path]:
        """Return the selected files in archive order."""
        selected, _excluded = self._selection()
        return list(selected)

    def excluded(self) -> list[str]:
        """Return paths left out of the archive, in display order.

        An excluded directory is one path, such as ``.git/``. Files inside it
        are not listed again.
        """
        _selected, excluded = self._selection()
        return list(excluded)

    def _selection(self) -> tuple[list[Path], list[str]]:
        if self._selected is None:
            selected, excluded = _collect(self.root, "", self._filters)
            selected.sort(key=lambda path: path.relative_to(self.root).as_posix())
            excluded.sort()
            self._selected = (selected, excluded)
        return self._selected

    def archive(self) -> bytes:
        """Zip the selected files."""
        buffer = BytesIO()
        with ZipFile(buffer, "w", compression=ZIP_DEFLATED, compresslevel=6) as archive:
            for path in self.files():
                info = ZipInfo(path.relative_to(self.root).as_posix(), _ZIP_DATE)
                info.compress_type = ZIP_DEFLATED
                info.create_system = 3
                info.external_attr = _ZIP_FILE_MODE
                archive.writestr(info, path.read_bytes())
        packed = buffer.getvalue()
        if len(packed) > _MAX_ARCHIVE_BYTES:
            raise ValueError(
                f"Source archive is {len(packed)} bytes, "
                f"above the {_MAX_ARCHIVE_BYTES} byte limit"
            )
        return packed


def default_filters(root: Path) -> tuple[SourceExclude, ...]:
    """Rules used when the caller does not supply their own."""
    return (DirectoryExclude(_DEFAULT_EXCLUDED_DIRECTORIES), GitignoreExclude(root))


def _collect(
    directory: Path,
    prefix: str,
    filters: tuple[SourceExclude, ...],
) -> tuple[list[Path], list[str]]:
    selected: list[Path] = []
    excluded: list[str] = []
    children = sorted(directory.iterdir(), key=lambda path: path.name)
    for child in children:
        relative = child.name if not prefix else f"{prefix}/{child.name}"
        if child.is_symlink():
            excluded.append(relative)
            continue
        if child.is_dir():
            nested, nested_excluded = _collect_directory(child, relative, filters)
            selected.extend(nested)
            excluded.extend(nested_excluded)
            continue
        if child.is_file() and _excluded(filters, relative):
            excluded.append(relative)
            continue
        if child.is_file():
            selected.append(child)
    return selected, excluded


def _collect_directory(
    directory: Path,
    relative: str,
    filters: tuple[SourceExclude, ...],
) -> tuple[list[Path], list[str]]:
    if _excluded(filters, f"{relative}/"):
        return [], [f"{relative}/"]
    return _collect(directory, relative, filters)


def _excluded(filters: tuple[SourceExclude, ...], relative: str) -> bool:
    return any(rule.excludes(relative) for rule in filters)


def _parts(relative: str) -> list[str]:
    return [part for part in relative.split("/") if part]
